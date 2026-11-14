use anyhow::{Context, anyhow};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;

/// Where the Video's sound comes from, as a `gst-launch` fragment producing raw audio.
pub trait AudioSource: Send {
    /// Whether a usable default input exists before recording starts.
    fn is_available(&self) -> bool;

    /// The fragment producing raw audio; used only when an input is available.
    fn source(&self) -> &str;
}

/// The system default microphone, through PulseAudio (or PipeWire's Pulse server).
pub struct DefaultMicrophone;

impl AudioSource for DefaultMicrophone {
    /// Open the default input to check access and availability, then release it.
    fn is_available(&self) -> bool {
        let Ok(probe) = gst::parse::launch("pulsesrc ! fakesink async=false") else {
            return false;
        };
        let available = probe.set_state(gst::State::Paused).is_ok()
            && matches!(
                probe.state(gst::ClockTime::from_seconds(1)),
                (Ok(_), gst::State::Paused, _)
            );
        let _ = probe.set_state(gst::State::Null);
        available
    }

    /// Capture from the system default input.
    fn source(&self) -> &str {
        "pulsesrc"
    }
}

/// Isolates microphone errors from the pipeline that encodes and saves the Video.
pub(super) struct AudioCapture {
    pipeline: gst::Pipeline,
    sink: gst_app::AppSink,
    first: gst::Sample,
}

impl AudioCapture {
    /// Open the input and require a real sample before reserving an audio track in the MP4.
    pub(super) fn start(source: &str) -> anyhow::Result<Self> {
        let pipeline = gst::parse::launch(&format!(
            "{source} ! appsink name=captured max-buffers=1 drop=true"
        ))?
        .downcast::<gst::Pipeline>()
        .map_err(|_| anyhow!("not an audio pipeline"))?;
        let sink = pipeline
            .by_name("captured")
            .and_then(|element| element.downcast::<gst_app::AppSink>().ok())
            .context("no audio appsink")?;
        let first = pipeline
            .set_state(gst::State::Playing)
            .ok()
            .and_then(|_| sink.try_pull_sample(gst::ClockTime::from_seconds(1)))
            .ok_or_else(|| {
                let _ = pipeline.set_state(gst::State::Null);
                anyhow!("audio input did not supply a sample")
            })?;
        Ok(Self {
            pipeline,
            sink,
            first,
        })
    }

    /// Forward audio to the writer; source errors end only this input, preserving earlier audio.
    pub(super) fn forward_to(&self, writer: &gst::Pipeline) -> anyhow::Result<()> {
        let input = writer
            .by_name("audio")
            .and_then(|element| element.downcast::<gst_app::AppSrc>().ok())
            .context("no audio appsrc")?;
        let bus = self.pipeline.bus().context("no audio bus")?;
        // Queue a known-good sample before forwarding any failure, avoiding an empty MP4 track.
        super::push_sample(&input, &self.first);
        self.sink.set_callbacks(callbacks(&input));
        let ended = input.clone();
        bus.set_sync_handler(move |_, message| {
            if matches!(message.view(), gst::MessageView::Error(_)) {
                let _ = input.end_of_stream();
            }
            gst::BusSyncReply::Drop
        });
        // A device may have failed after the first sample but before the handler was installed.
        if bus
            .pop_filtered(&[gst::MessageType::Error, gst::MessageType::Eos])
            .is_some()
        {
            let _ = ended.end_of_stream();
        }
        Ok(())
    }
}

impl Drop for AudioCapture {
    /// Release the input device even if starting it failed.
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}

/// Bridge captured audio into the writer's clock domain and end it when the source ends.
fn callbacks(input: &gst_app::AppSrc) -> gst_app::AppSinkCallbacks {
    let samples = input.clone();
    let ended = input.clone();
    gst_app::AppSinkCallbacks::builder()
        .new_sample(move |sink| {
            let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
            super::push_sample(&samples, &sample);
            Ok(gst::FlowSuccess::Ok)
        })
        .eos(move |_| {
            let _ = ended.end_of_stream();
        })
        .build()
}
