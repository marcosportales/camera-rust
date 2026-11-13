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

mod audio;
use audio::AudioCapture;
pub use audio::{AudioSource, DefaultMicrophone};

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
        let audio = self
            .audio
            .is_available()
            .then(|| AudioCapture::start(self.audio.source()).ok())
            .flatten();
        let pipeline = self.pipeline(audio.is_some())?;
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
        if let Some(audio) = &audio
            && let Err(error) = audio.forward_to(&pipeline)
        {
            let _ = pipeline.set_state(gst::State::Null);
            return Err(error);
        }
        Ok(VideoWriter {
            pipeline,
            input,
            path: path.to_path_buf(),
            audio,
        })
    }

    /// Build the writer with a video input and, when available, an independent audio input.
    fn pipeline(&self, with_audio: bool) -> anyhow::Result<gst::Pipeline> {
        let (video, audio_encoder, muxer) = (
            self.format.video_encoder(),
            self.format.audio_encoder(),
            self.format.muxer(),
        );
        let audio = if with_audio {
            format!(
                "appsrc name=audio is-live=true do-timestamp=true format=time \
                ! queue ! audioconvert ! audioresample ! {audio_encoder} ! queue ! mux."
            )
        } else {
            String::new()
        };
        // The appsrc stamps each frame with the pipeline clock on arrival, so video and audio share a timeline.
        let description = format!(
            "appsrc name=video is-live=true do-timestamp=true format=time \
             ! queue ! jpegdec ! videoconvert ! {video} ! queue ! mux. \
             {audio} \
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
    audio: Option<AudioCapture>,
}

impl VideoWriter {
    /// Where the camera's MJPG frames go; feed it with [`push_sample`].
    pub fn input(&self) -> &gst_app::AppSrc {
        &self.input
    }

    /// Send an end-of-stream through the pipeline so the muxer finishes the file; a thread waits for
    /// it, closes the file, and resolves the returned [`VideoSaving`] with the outcome.
    pub fn stop(self) -> VideoSaving {
        let Self {
            pipeline,
            path,
            audio,
            ..
        } = self;
        let (finished, saving) = oneshot::channel();
        thread::spawn(move || {
            // Stop the microphone before ending both writer inputs, so it cannot enqueue more audio.
            drop(audio);
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

/// Push a captured audio or MJPG sample into the writer, using its shared timeline.
pub fn push_sample(input: &gst_app::AppSrc, sample: &gst::Sample) {
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
