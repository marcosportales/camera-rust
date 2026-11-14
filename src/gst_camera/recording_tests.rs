use super::*;
use crate::app_core::state::{AppState, CaptureOutcome, Event, Mode, VideoCapture};
use crate::gst_recording::{AudioSource, H264AacMp4, VideoFormat};
use futures::channel::mpsc;
use std::time::Duration;

/// Script availability, loss while opening, and loss after capture starts without real hardware.
enum FakeAudio {
    Available,
    Unavailable,
    FailsToOpen,
    FailsImmediately,
    Disconnects,
}

impl AudioSource for FakeAudio {
    /// Report whether a usable input exists at the pre-recording check.
    fn is_available(&self) -> bool {
        !matches!(self, Self::Unavailable)
    }

    /// Supply synthetic audio or a failure at the scripted stage.
    fn source(&self) -> &str {
        match self {
            Self::Available => "audiotestsrc is-live=true",
            Self::Unavailable => panic!("an unavailable input must never be opened"),
            Self::FailsToOpen => "missing-audio-source",
            Self::FailsImmediately => "audiotestsrc is-live=true ! identity error-after=1",
            Self::Disconnects => "audiotestsrc is-live=true ! identity error-after=20",
        }
    }
}

/// The real Video format with a fake available microphone.
pub(super) fn test_recorder() -> Recorder {
    Recorder::new(H264AacMp4, FakeAudio::Available)
}

/// The core starts and saves normally with an unavailable microphone; the MP4 has only video.
#[test]
fn unavailable_audio_saves_exactly_one_video_stream() {
    let (streams, _) = record_video(FakeAudio::Unavailable, Duration::from_millis(500));
    assert_eq!(streams, ["video/x-h264"]);
}

/// Available audio is recorded and core feedback appears only after the Video is saved.
#[test]
fn available_audio_is_recorded_and_saves_normally() {
    gst::init().unwrap();
    let required = ["openh264enc", "avenc_aac", "mp4mux"];
    if let Some(missing) = required
        .iter()
        .find(|name| gst::ElementFactory::find(name).is_none())
    {
        eprintln!("skipped: GStreamer element {missing} is not installed");
        return;
    }
    let (streams, duration) = record_video(FakeAudio::Available, Duration::from_secs(2));
    assert_eq!(streams, ["audio/mpeg", "video/x-h264"]);
    assert!(
        (Duration::from_millis(1500)..Duration::from_secs(3)).contains(&duration),
        "duration {duration:?}"
    );
}

/// Audio failure must not interrupt subsequent video frames or produce failure feedback.
#[test]
fn disconnected_audio_keeps_recording_video_and_saves_normally() {
    let (streams, duration) = record_video(FakeAudio::Disconnects, Duration::from_secs(1));
    assert_eq!(streams, ["audio/mpeg", "video/x-h264"]);
    assert!(
        duration >= Duration::from_millis(800),
        "video stopped with audio at {duration:?}"
    );
}

/// A device lost between the availability check and opening is treated as unavailable.
#[test]
fn audio_lost_while_opening_saves_video_only() {
    let (streams, _) = record_video(FakeAudio::FailsToOpen, Duration::from_millis(500));
    assert_eq!(streams, ["video/x-h264"]);
}

/// An asynchronous failure before the first audio sample must not stall the muxer.
#[test]
fn audio_failing_before_the_first_sample_saves_video_only() {
    let (streams, _) = record_video(FakeAudio::FailsImmediately, Duration::from_millis(500));
    assert_eq!(streams, ["video/x-h264"]);
}

/// A camera supplying MJPG frames without needing webcam hardware.
fn test_camera(recorder: Recorder) -> GstCamera {
    gst::init().unwrap();
    let (frames, _) = mpsc::channel(0);
    let mut camera = GstCamera::new(frames, recorder);
    camera.source = Some("videotestsrc is-live=true ! jpegenc".into());
    camera
        .start_viewfinder(Resolution {
            width: 640,
            height: 360,
        })
        .unwrap();
    camera
}

/// Exercise core start/stop/save feedback with fake audio and inspect the actual saved MP4.
fn record_video(audio: FakeAudio, duration: Duration) -> (Vec<String>, Duration) {
    let mut camera = test_camera(Recorder::new(H264AacMp4, audio));
    let mut state = AppState::new();
    state.handle(Event::CameraReady(Camera {
        name: "Test camera".into(),
        resolutions: vec![Resolution {
            width: 640,
            height: 360,
        }],
    }));
    state.handle(Event::SetMode(Mode::Video));
    let folder = tempfile::tempdir().unwrap();
    state.start_recording(&mut camera, folder.path());
    assert!(state.recording().is_some());
    assert_eq!(state.last_capture, None);
    std::thread::sleep(duration);
    let saving = state.stop_recording(&mut camera).unwrap();
    assert_eq!(state.video, Some(VideoCapture::Saving));
    assert_eq!(state.last_capture, None);
    let saved = futures::executor::block_on(saving).unwrap();
    let path = saved.as_ref().unwrap().clone();
    state.handle(saved.into());
    assert_eq!(state.video, None);
    assert_eq!(
        state.last_capture,
        Some(CaptureOutcome::VideoSaved(path.clone()))
    );
    assert_eq!(path.extension().unwrap(), "mp4");
    inspect_mp4(&path)
}

/// Simulate an encoder failure in the video writer rather than a recoverable microphone error.
struct FailingVideo;

impl VideoFormat for FailingVideo {
    /// Fail as soon as a video frame reaches the encoder.
    fn video_encoder(&self) -> &str {
        "identity error-after=1 ! openh264enc ! h264parse"
    }

    /// Encode audio normally so it cannot cause the failure.
    fn audio_encoder(&self) -> &str {
        "avenc_aac"
    }

    /// Mux the capture into MP4.
    fn muxer(&self) -> &str {
        "mp4mux"
    }
}

/// A real video pipeline error must still be reported as a failed save.
#[test]
fn test_source_video_reports_pipeline_failure_on_stop() {
    let mut camera = test_camera(Recorder::new(FailingVideo, FakeAudio::Available));
    let folder = tempfile::tempdir().unwrap();
    camera.start_video(folder.path()).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(
        futures::executor::block_on(camera.stop_video()).unwrap(),
        Err(VideoError)
    );
}

/// Demux the entire saved file and return every stream's media type and the duration.
fn inspect_mp4(path: &Path) -> (Vec<String>, Duration) {
    let pipeline = gst::parse::launch("filesrc name=file ! qtdemux name=demux")
        .unwrap()
        .downcast::<gst::Pipeline>()
        .unwrap();
    pipeline
        .by_name("file")
        .unwrap()
        .set_property("location", path);
    let streams = Arc::new(Mutex::new(Vec::new()));
    let observed = Arc::clone(&streams);
    let weak_pipeline = pipeline.downgrade();
    pipeline
        .by_name("demux")
        .unwrap()
        .connect_pad_added(move |_, pad| {
            let pipeline = weak_pipeline.upgrade().unwrap();
            let caps = pad.current_caps().unwrap();
            observed
                .lock()
                .unwrap()
                .push(caps.structure(0).unwrap().name().to_string());
            let sink = gst::ElementFactory::make("fakesink")
                .property("sync", false)
                .property("async", false)
                .build()
                .unwrap();
            pipeline.add(&sink).unwrap();
            sink.sync_state_with_parent().unwrap();
            pad.link(&sink.static_pad("sink").unwrap()).unwrap();
        });
    pipeline.set_state(gst::State::Playing).unwrap();
    let message = pipeline.bus().unwrap().timed_pop_filtered(
        gst::ClockTime::from_seconds(5),
        &[gst::MessageType::Eos, gst::MessageType::Error],
    );
    let duration = pipeline.query_duration::<gst::ClockTime>();
    pipeline.set_state(gst::State::Null).unwrap();
    assert!(
        matches!(
            message.as_ref().map(|m| m.view()),
            Some(gst::MessageView::Eos(_))
        ),
        "demux failed: {message:?}"
    );
    let mut result = streams.lock().unwrap().clone();
    result.sort();
    (result, Duration::from_nanos(duration.unwrap().nseconds()))
}
