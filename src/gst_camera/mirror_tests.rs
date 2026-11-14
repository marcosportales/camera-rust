use super::*;
use crate::app_core::state::{AppState, CaptureOutcome, Event, Mode, ShutterAction, Timer};
use futures::channel::mpsc;
use std::time::Duration;

/// Core capture requests persist Mirror in actual MP4 pixels, independently of subsequent toggles.
#[test]
fn saved_video_follows_mirror_setting() {
    gst::init().unwrap();
    for (mirror, timer) in [
        (true, Timer::Off),
        (false, Timer::Off),
        (true, Timer::ThreeSeconds),
        (false, Timer::TenSeconds),
    ] {
        let (frames, _) = mpsc::channel(0);
        let mut camera = GstCamera::new(frames, super::recording_tests::silent_recorder());
        let mut state = AppState::new();
        state.handle(Event::CameraReady(Camera {
            name: "Test camera".into(),
            resolutions: vec![],
        }));
        state.handle(Event::SetMode(Mode::Video));
        if !mirror {
            state.handle(Event::ToggleMirror);
        }
        let folder = tempfile::tempdir().unwrap();
        let action = capture_action(&mut state, timer);
        assert_eq!(action, Some(ShutterAction::StartRecording));
        state.start_recording(&mut camera, folder.path());
        assert!(state.recording().is_some());
        state.handle(Event::ToggleMirror);
        assert_eq!(state.mirror, mirror, "Mirror stays locked during recording");
        let input = lock(&camera.jpeg).recording.clone().unwrap();
        let sample = asymmetric_sample();
        for _ in 0..6 {
            gst_recording::push_sample(&input, &sample);
            std::thread::sleep(std::time::Duration::from_millis(35));
        }
        let saved =
            futures::executor::block_on(state.stop_recording(&mut camera).unwrap()).unwrap();
        let path = saved.as_ref().unwrap().clone();
        state.handle(saved.into());
        assert_eq!(
            state.last_capture,
            Some(CaptureOutcome::VideoSaved(path.clone()))
        );
        assert_video_orientation(&path, mirror);
    }
}

/// Encode a known asymmetric grayscale frame with distinct upper and lower rows.
fn asymmetric_sample() -> gst::Sample {
    let pixels = image::RgbImage::from_fn(32, 16, |x, y| {
        image::Rgb([if (x < 16) == (y < 8) { 20 } else { 230 }; 3])
    });
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 95)
        .encode_image(&pixels)
        .unwrap();
    gst::Sample::builder()
        .buffer(&gst::Buffer::from_slice(jpeg))
        .caps(
            &gst::Caps::builder("image/jpeg")
                .field("width", 32i32)
                .field("height", 16i32)
                .field("framerate", gst::Fraction::new(30, 1))
                .build(),
        )
        .build()
}

/// Decode an actual saved MP4 and verify horizontal orientation, dimensions and row order.
fn assert_video_orientation(path: &Path, mirror: bool) {
    let pipeline = gst::parse::launch(
        "filesrc name=file ! qtdemux ! decodebin ! videoconvert \
         ! video/x-raw,format=RGB ! appsink name=pixels sync=false",
    )
    .unwrap()
    .downcast::<gst::Pipeline>()
    .unwrap();
    pipeline
        .by_name("file")
        .unwrap()
        .set_property("location", path);
    let sink = pipeline
        .by_name("pixels")
        .unwrap()
        .downcast::<gst_app::AppSink>()
        .unwrap();
    pipeline.set_state(gst::State::Playing).unwrap();
    let sample = sink.try_pull_sample(gst::ClockTime::from_seconds(5));
    pipeline.set_state(gst::State::Null).unwrap();
    let sample = sample.expect("saved Video should decode");
    assert_eq!(
        fixed_size(sample.caps().unwrap().structure(0).unwrap()),
        Some(Resolution {
            width: 32,
            height: 16
        })
    );
    let pixels = sample.buffer().unwrap().map_readable().unwrap();
    for (x, y, original) in [(4, 3, 20i16), (27, 3, 230), (4, 12, 230), (27, 12, 20)] {
        let expected = if mirror { 250 - original } else { original };
        assert!((i16::from(pixels[(y * 32 + x) * 3]) - expected).abs() < 20);
    }
}

/// Bad input and an unwritable destination produce Photo failure feedback without a partial file.
#[test]
fn mirrored_photo_failure_leaves_no_capture() {
    gst::init().unwrap();
    let (frames, _) = mpsc::channel(0);
    let mut camera = GstCamera::new(frames, super::recording_tests::silent_recorder());
    let folder = tempfile::tempdir().unwrap();
    lock(&camera.jpeg).latest = Some(gst::Buffer::from_slice(b"invalid JPEG".to_vec()));
    let mut state = AppState::new();
    state.take_photo(&mut camera, folder.path());
    assert_eq!(state.last_capture, Some(CaptureOutcome::PhotoFailed));
    assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 0);
    lock(&camera.jpeg).latest = asymmetric_sample().buffer_owned();
    let file = tempfile::NamedTempFile::new().unwrap();
    state.take_photo(&mut camera, &file.path().join("Camera"));
    assert_eq!(state.last_capture, Some(CaptureOutcome::PhotoFailed));
    assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 0);
}

/// Saved Photos follow Mirror on/off/on without altering the source or changing row order.
#[test]
fn saved_photo_follows_mirror_setting() {
    gst::init().unwrap();
    let sample = asymmetric_sample();
    let source = sample.buffer().unwrap().map_readable().unwrap().to_vec();
    let (frames_tx, _frames_rx) = mpsc::channel(0);
    let mut camera = GstCamera::new(frames_tx, super::recording_tests::silent_recorder());
    lock(&camera.jpeg).latest = Some(gst::Buffer::from_slice(source.clone()));
    let folder = tempfile::tempdir().unwrap();
    let mut state = AppState::new();
    state.handle(Event::CameraReady(Camera {
        name: "Test camera".into(),
        resolutions: vec![],
    }));
    for (mirror, timer) in [
        (true, Timer::Off),
        (false, Timer::ThreeSeconds),
        (true, Timer::TenSeconds),
    ] {
        let action = capture_action(&mut state, timer);
        assert_eq!(action, Some(ShutterAction::TakePhoto));
        assert_eq!(state.mirror, mirror);
        state.take_photo(&mut camera, folder.path());
        let Some(CaptureOutcome::PhotoSaved(path)) = &state.last_capture else {
            panic!("Photo was not saved");
        };
        let saved = fs::read(path).unwrap();
        let decoded = image::load_from_memory(&saved).unwrap().to_rgb8();
        assert_eq!(decoded.dimensions(), (32, 16));
        for (x, y, original) in [(4, 3, 20i16), (27, 3, 230), (4, 12, 230), (27, 12, 20)] {
            let expected = if mirror { 250 - original } else { original };
            assert!((i16::from(decoded.get_pixel(x, y)[0]) - expected).abs() < 10);
        }
        if !mirror {
            assert_eq!(saved, source);
        }
        state.handle(Event::ToggleMirror);
    }
}

/// Press the Shutter and advance an enabled Timer to its deadline, without wall-clock waiting.
fn capture_action(state: &mut AppState, timer: Timer) -> Option<ShutterAction> {
    state.handle(Event::SetTimer(timer));
    let action = state.press_shutter(Duration::ZERO);
    if timer == Timer::Off {
        return action;
    }
    assert_eq!(action, None);
    state.advance_countdown(Duration::from_secs(10))
}
