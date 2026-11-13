//! Recording a Video: the encoder + container and the audio source behind traits, and the
//! pipeline that turns the camera's MJPG frames plus audio into a file.

use std::path::{Path, PathBuf};
use std::thread;

use anyhow::{Context, anyhow};
use futures::channel::oneshot;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;

use crate::app_core::backend::{VideoError, VideoSaving};

/// How long a stopped Video may take to finish writing its file.
const FINALIZE_TIMEOUT: gst::ClockTime = gst::ClockTime::from_seconds(10);

/// Encodes raw video and audio and muxes them into a file. Each method returns a `gst-launch` fragment.
pub trait VideoFormat: Send {
    /// Raw video in, encoded video ready for the muxer out.
    fn video_encoder(&self) -> &str;
    /// Raw audio in, encoded audio ready for the muxer out.
    fn audio_encoder(&self) -> &str;
    /// The muxer writing the container.
    fn muxer(&self) -> &str;
}

/// H.264 (`openh264enc`) and AAC (`avenc_aac`) in MP4. `x264enc` is not installed on the target machine.
pub struct H264AacMp4;

impl VideoFormat for H264AacMp4 {
    /// OpenH264, parsed into the stream format MP4 needs.
    fn video_encoder(&self) -> &str {
        "openh264enc ! h264parse"
    }

    /// FFmpeg's AAC encoder.
    fn audio_encoder(&self) -> &str {
        "avenc_aac"
    }

    /// The MP4 muxer.
    fn muxer(&self) -> &str {
        "mp4mux"
    }
}

/// Where the Video's sound comes from, as a `gst-launch` fragment producing raw audio.
pub trait AudioSource: Send {
    /// The fragment producing raw audio.
    fn source(&self) -> &str;
}

/// The system default microphone, through PulseAudio (or PipeWire's Pulse server).
pub struct DefaultMicrophone;

impl AudioSource for DefaultMicrophone {
    /// `pulsesrc` on its default device.
    fn source(&self) -> &str {
        "pulsesrc"
    }
}

/// Starts Videos with a given format and audio source.
pub struct Recorder {
    format: Box<dyn VideoFormat>,
    audio: Box<dyn AudioSource>,
}

impl Recorder {
    /// A recorder writing `format` with sound from `audio`.
    pub fn new(format: impl VideoFormat + 'static, audio: impl AudioSource + 'static) -> Self {
        Self {
            format: Box::new(format),
            audio: Box::new(audio),
        }
    }

    /// Start a Video written to `path`; its frames are pushed into [`VideoWriter::input`].
    pub fn start(&self, path: &Path) -> anyhow::Result<VideoWriter> {
        let pipeline = self.pipeline()?;
        pipeline
            .by_name("file")
            .context("no filesink")?
            .set_property("location", path);
        let input = pipeline
            .by_name("video")
            .and_then(|element| element.downcast::<gst_app::AppSrc>().ok())
            .context("no appsrc")?;
        if let Err(error) = pipeline.set_state(gst::State::Playing) {
            let _ = pipeline.set_state(gst::State::Null);
            return Err(error.into());
        }
        Ok(VideoWriter {
            pipeline,
            input,
            path: path.to_path_buf(),
        })
    }

    /// Build the pipeline: MJPG frames from an appsrc → encoder, audio → encoder, both into the muxer → file.
    fn pipeline(&self) -> anyhow::Result<gst::Pipeline> {
        let (video, audio_encoder, muxer) = (
            self.format.video_encoder(),
            self.format.audio_encoder(),
            self.format.muxer(),
        );
        let audio = self.audio.source();
        // The appsrc stamps each frame with the pipeline clock on arrival, so video and audio share a timeline.
        let description = format!(
            "appsrc name=video is-live=true do-timestamp=true format=time \
             ! queue ! jpegdec ! videoconvert ! {video} ! queue ! mux. \
             {audio} ! queue ! audioconvert ! audioresample ! {audio_encoder} ! queue ! mux. \
             {muxer} name=mux ! filesink name=file"
        );
        gst::parse::launch(&description)?
            .downcast()
            .map_err(|_| anyhow!("not a pipeline"))
    }
}

/// The pipeline writing a Video being recorded.
pub struct VideoWriter {
    pipeline: gst::Pipeline,
    input: gst_app::AppSrc,
    path: PathBuf,
}

impl VideoWriter {
    /// Where the camera's MJPG frames go; feed it with [`push_frame`].
    pub fn input(&self) -> &gst_app::AppSrc {
        &self.input
    }

    /// Send an end-of-stream through the pipeline so the muxer finishes the file; a thread waits for
    /// it, closes the file, and resolves the returned [`VideoSaving`] with the outcome.
    pub fn stop(self) -> VideoSaving {
        let Self { pipeline, path, .. } = self;
        let (finished, saving) = oneshot::channel();
        thread::spawn(move || {
            // Sending EOS can wait on source locks; keep it off the UI thread too.
            // A pipeline forwards EOS to every source, so both video and audio end.
            let eos = if pipeline.send_event(gst::event::Eos::new()) {
                wait_for_eos(&pipeline)
            } else {
                Err(anyhow!("pipeline refused end-of-stream"))
            };
            // Close even after failure, and announce success only once the file is closed.
            let finalized = eos.and(close(&pipeline));
            let outcome = finalized.map(|()| path).map_err(|error| {
                eprintln!("Could not finish the video: {error:#}");
                VideoError
            });
            // The receiver is gone only if the window closed; nobody is left to tell.
            let _ = finished.send(outcome);
        });
        saving
    }
}

/// Block until `pipeline` reports end-of-stream; an error (also one from while recording, e.g. a full
/// disk) or the finalize timeout fails.
fn wait_for_eos(pipeline: &gst::Pipeline) -> anyhow::Result<()> {
    let bus = pipeline.bus().context("no bus")?;
    let message = bus.timed_pop_filtered(
        FINALIZE_TIMEOUT,
        &[gst::MessageType::Eos, gst::MessageType::Error],
    );
    match message.as_ref().map(|message| message.view()) {
        Some(gst::MessageView::Eos(_)) => Ok(()),
        Some(gst::MessageView::Error(error)) => Err(error.error().into()),
        _ => Err(anyhow!("no end-of-stream within {FINALIZE_TIMEOUT}")),
    }
}

/// Stop `pipeline`, which closes the file; the Video is complete only after this.
fn close(pipeline: &gst::Pipeline) -> anyhow::Result<()> {
    pipeline
        .set_state(gst::State::Null)
        .map(drop)
        .context("could not close the file")
}

/// Push one MJPG frame from the camera into a VideoWriter's `input`.
pub fn push_frame(input: &gst_app::AppSrc, sample: &gst::Sample) {
    let Some(buffer) = sample.buffer() else {
        return;
    };
    // Shares the memory; the camera's timestamps are cleared so the appsrc stamps the frame itself.
    let mut frame = buffer.copy();
    let frame_mut = frame.make_mut();
    frame_mut.set_pts(None);
    frame_mut.set_dts(None);
    let caps = sample.caps_owned();
    let restamped = gst::Sample::builder()
        .buffer(&frame)
        .caps_if_some(caps.as_ref())
        .build();
    // A refused frame means the Video is ending; dropping it is harmless.
    let _ = input.push_sample(&restamped);
}
