//! Real camera backend: V4L2 capture through GStreamer (ADR 0002).

use futures::channel::mpsc::Sender;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;

use crate::app_core::backend::{Camera, CameraBackend, CameraError};
use crate::app_core::resolution::Resolution;

/// One decoded Viewfinder frame: tightly packed BGRA rows.
pub struct Frame {
    pub size: Resolution,
    pub bgra: Vec<u8>,
}

impl Frame {
    /// Copy a BGRA appsink sample into a frame, or `None` if its caps or size do not add up.
    fn from_sample(sample: &gst::Sample) -> Option<Self> {
        let size = fixed_size(sample.caps()?.structure(0)?)?;
        // BGRA rows are 4-byte aligned already, so GStreamer adds no row padding.
        let bgra = sample.buffer()?.map_readable().ok()?.to_vec();
        let expected_len = usize::try_from(size.pixels() * 4).ok()?;
        (bgra.len() == expected_len).then_some(Self { size, bgra })
    }
}

/// Camera backend that captures MJPG from a V4L2 device and sends decoded frames to the UI.
pub struct GstCamera {
    /// Launch description of the element(s) producing `image/jpeg`; set by detection.
    source: Option<String>,
    frames: Sender<Frame>,
    pipeline: Option<gst::Pipeline>,
}

impl GstCamera {
    /// A backend that sends Viewfinder frames to `frames` once started.
    pub fn new(frames: Sender<Frame>) -> Self {
        Self {
            source: None,
            frames,
            pipeline: None,
        }
    }
}

impl CameraBackend for GstCamera {
    /// Find the first capture-capable V4L2 camera and its MJPG Resolutions.
    fn detect_cameras(&mut self) -> Result<Camera, CameraError> {
        gst::init().map_err(|_| CameraError::NoCamera)?;
        // The V4L2 provider lists only video-capture nodes, so metadata nodes are already skipped.
        let provider = gst::DeviceProviderFactory::by_name("v4l2deviceprovider")
            .ok_or(CameraError::NoCamera)?;
        let (path, device) = provider
            .devices()
            .into_iter()
            .filter_map(|device| {
                let path = device_path(&device)?;
                Some((video_node_number(&path)?, path, device))
            })
            .min_by_key(|(node, _, _)| *node)
            .map(|(_, path, device)| (path, device))
            .ok_or(CameraError::NoCamera)?;
        let resolutions = device
            .caps()
            .map_or_else(Vec::new, |caps| mjpg_resolutions(&caps));
        // Without an MJPG mode the Viewfinder cannot start, so the camera is unusable.
        if resolutions.is_empty() {
            return Err(CameraError::NoCamera);
        }
        let camera = Camera {
            name: card_name(&device).ok_or(CameraError::NoCamera)?,
            resolutions,
        };
        self.source = Some(format!("v4l2src device={path}"));
        Ok(camera)
    }

    /// Start the pipeline at `resolution`, decoding frames into the UI channel.
    fn start_viewfinder(&mut self, resolution: Resolution) -> Result<(), CameraError> {
        let source = self.source.as_deref().ok_or(CameraError::NoCamera)?;
        let pipeline = viewfinder_pipeline(source, resolution)?;
        send_frames_to(&pipeline, self.frames.clone())?;
        play(&pipeline)?;
        self.pipeline = Some(pipeline);
        Ok(())
    }
}

impl Drop for GstCamera {
    /// Stop the pipeline so the device is released.
    fn drop(&mut self) {
        if let Some(pipeline) = &self.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
    }
}

/// The `/dev/video*` path of a V4L2 device.
fn device_path(device: &gst::Device) -> Option<String> {
    device.properties()?.get("device.path").ok()
}

/// The device name V4L2 reports (its "card").
fn card_name(device: &gst::Device) -> Option<String> {
    device.properties()?.get("v4l2.device.card").ok()
}

/// The `N` in `/dev/videoN`, so `/dev/video2` sorts before `/dev/video10`.
fn video_node_number(path: &str) -> Option<u32> {
    path.strip_prefix("/dev/video")?.parse().ok()
}

/// Build `source` → MJPG at `resolution` → decode → BGRA appsink. No frame rate is forced.
fn viewfinder_pipeline(source: &str, resolution: Resolution) -> Result<gst::Pipeline, CameraError> {
    let Resolution { width, height } = resolution;
    let description = format!(
        "{source} ! image/jpeg,width={width},height={height} ! jpegdec ! videoconvert \
         ! video/x-raw,format=BGRA ! appsink name=frames max-buffers=1 drop=true sync=false"
    );
    // A parse failure means a GStreamer plugin is missing: no usable camera.
    gst::parse::launch(&description)
        .ok()
        .and_then(|element| element.downcast().ok())
        .ok_or(CameraError::NoCamera)
}

/// Forward every decoded frame from the pipeline's appsink to `frames`.
fn send_frames_to(pipeline: &gst::Pipeline, mut frames: Sender<Frame>) -> Result<(), CameraError> {
    let sink = pipeline
        .by_name("frames")
        .and_then(|element| element.downcast::<gst_app::AppSink>().ok())
        .ok_or(CameraError::NoCamera)?;
    let callbacks = gst_app::AppSinkCallbacks::builder()
        .new_sample(move |sink| {
            let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
            if let Some(frame) = Frame::from_sample(&sample) {
                // A full channel means the UI has not drawn the last frame yet: dropping this one keeps latency low.
                let _ = frames.try_send(frame);
            }
            Ok(gst::FlowSuccess::Ok)
        })
        .build();
    sink.set_callbacks(callbacks);
    Ok(())
}

/// Set the pipeline playing and wait until it is, mapping a failure to its camera error.
fn play(pipeline: &gst::Pipeline) -> Result<(), CameraError> {
    let started = pipeline.set_state(gst::State::Playing).is_ok()
        && pipeline.state(gst::ClockTime::from_seconds(5)).0.is_ok();
    if started {
        return Ok(());
    }
    let error = pipeline
        .bus()
        .and_then(|bus| bus.pop_filtered(&[gst::MessageType::Error]))
        .map_or(CameraError::NoCamera, |message| camera_error(&message));
    let _ = pipeline.set_state(gst::State::Null);
    Err(error)
}

/// Map a GStreamer error message to the camera error it means.
fn camera_error(message: &gst::Message) -> CameraError {
    let gst::MessageView::Error(error) = message.view() else {
        return CameraError::NoCamera;
    };
    match error.error().kind::<gst::ResourceError>() {
        Some(gst::ResourceError::Busy) => CameraError::InUse,
        Some(gst::ResourceError::NotAuthorized) => CameraError::AccessDenied,
        _ => CameraError::NoCamera,
    }
}

/// The Resolutions of the `image/jpeg` (MJPG) modes in `caps`, without duplicates; other formats are ignored.
fn mjpg_resolutions(caps: &gst::CapsRef) -> Vec<Resolution> {
    let mut resolutions = Vec::new();
    // Modes with a range instead of a fixed size are skipped: the Resolution menu lists discrete sizes.
    for resolution in caps.iter().filter_map(fixed_mjpg_size) {
        if !resolutions.contains(&resolution) {
            resolutions.push(resolution);
        }
    }
    resolutions
}

/// The fixed size of an `image/jpeg` caps structure, or `None` for other formats or ranges.
fn fixed_mjpg_size(structure: &gst::StructureRef) -> Option<Resolution> {
    if structure.name() != "image/jpeg" {
        return None;
    }
    fixed_size(structure)
}

/// The fixed `width` × `height` of a caps structure, or `None` if either is missing or a range.
fn fixed_size(structure: &gst::StructureRef) -> Option<Resolution> {
    let dimension = |field| u32::try_from(structure.get::<i32>(field).ok()?).ok();
    Some(Resolution {
        width: dimension("width")?,
        height: dimension("height")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::channel::mpsc;
    use std::time::{Duration, Instant};

    /// Shorthand for building a Resolution in tests.
    fn res(width: u32, height: u32) -> Resolution {
        Resolution { width, height }
    }

    /// MJPG modes become Resolutions once each; YUYV modes are ignored.
    #[test]
    fn only_mjpg_modes_are_supported_resolutions() {
        gst::init().unwrap();
        let caps: gst::Caps = "image/jpeg, width=1280, height=720, framerate=30/1; \
             image/jpeg, width=1280, height=720, framerate=15/1; \
             video/x-raw, format=YUY2, width=320, height=240; \
             image/jpeg, width=640, height=360"
            .parse()
            .unwrap();
        assert_eq!(mjpg_resolutions(&caps), [res(1280, 720), res(640, 360)]);
    }

    /// The backend, fed by `videotestsrc` instead of a webcam, delivers BGRA frames at the requested Resolution.
    #[test]
    fn test_source_delivers_frames_at_the_requested_resolution() {
        gst::init().unwrap();
        let (frames_tx, mut frames_rx) = mpsc::channel(0);
        let mut camera = GstCamera::new(frames_tx);
        // Stand-in for `v4l2src`: a live source producing MJPG like a webcam does.
        camera.source = Some("videotestsrc is-live=true ! jpegenc".into());

        camera.start_viewfinder(res(640, 360)).unwrap();

        let deadline = Instant::now() + Duration::from_secs(5);
        let frame = loop {
            if let Ok(frame) = frames_rx.try_recv() {
                break frame;
            }
            assert!(Instant::now() < deadline, "no frame within 5 s");
            std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(frame.size, res(640, 360));
        assert_eq!(frame.bgra.len(), 640 * 360 * 4);
    }
}
