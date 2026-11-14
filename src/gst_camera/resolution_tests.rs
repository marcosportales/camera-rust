//! Real-pipeline checks for switching the Viewfinder's Resolution.

use super::recording_tests::test_recorder;
use super::*;
use futures::channel::mpsc;
use std::time::{Duration, Instant};

/// Wait for the first frame at the requested size, allowing an already queued old frame.
fn wait_for_size(frames: &mut mpsc::Receiver<Frame>, size: Resolution) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(frame) = frames.try_recv()
            && frame.size == size
        {
            return;
        }
        assert!(Instant::now() < deadline, "no frame at {size:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Restarting releases the old pipeline and delivers frames and Photos at the new size.
#[test]
fn resolution_restart_releases_old_pipeline_and_changes_frames() {
    gst::init().unwrap();
    let (tx, mut frames) = mpsc::channel(0);
    let mut camera = GstCamera::new(tx, test_recorder());
    camera.source = Some("videotestsrc is-live=true ! jpegenc".into());
    let hd = Resolution {
        width: 1280,
        height: 720,
    };
    let vga = Resolution {
        width: 640,
        height: 480,
    };
    camera.start_viewfinder(hd).unwrap();
    wait_for_size(&mut frames, hd);
    let old = camera.pipeline.clone().unwrap();
    camera.start_viewfinder(vga).unwrap();
    assert_eq!(old.current_state(), gst::State::Null);
    wait_for_size(&mut frames, vga);
    let folder = tempfile::tempdir().unwrap();
    let path = camera.take_photo(folder.path(), false).unwrap();
    let photo = image::open(path).unwrap();
    assert_eq!((photo.width(), photo.height()), (640, 480));
}
