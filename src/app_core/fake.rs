use super::backend::{Camera, CameraBackend, CameraError};
use super::resolution::Resolution;

/// Scriptable stand-in for the real camera.
pub struct FakeCamera {
    detection: Result<Camera, CameraError>,
    /// Resolutions the Viewfinder was started at, in order.
    pub started_at: Vec<Resolution>,
}

impl FakeCamera {
    /// A camera that detects as ready with the given name and Resolutions.
    pub fn ready(name: &str, resolutions: Vec<Resolution>) -> Self {
        Self::scripted(Ok(Camera {
            name: name.to_owned(),
            resolutions,
        }))
    }

    /// A camera whose detection fails with the given error.
    pub fn failing(error: CameraError) -> Self {
        Self::scripted(Err(error))
    }

    /// Build a fake that answers detection with the scripted result.
    fn scripted(detection: Result<Camera, CameraError>) -> Self {
        Self {
            detection,
            started_at: Vec::new(),
        }
    }
}

impl CameraBackend for FakeCamera {
    /// Return the scripted detection result.
    fn detect_cameras(&mut self) -> Result<Camera, CameraError> {
        self.detection.clone()
    }

    /// Record the Resolution; fail like detection would if the camera is scripted as failing.
    fn start_viewfinder(&mut self, resolution: Resolution) -> Result<(), CameraError> {
        self.detection.as_ref().map_err(|error| *error)?;
        self.started_at.push(resolution);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HD: Resolution = Resolution {
        width: 1280,
        height: 720,
    };

    #[test]
    fn viewfinder_start_is_recorded_for_a_ready_camera() {
        let mut camera = FakeCamera::ready("Cam", vec![HD]);
        assert_eq!(camera.start_viewfinder(HD), Ok(()));
        assert_eq!(camera.started_at, [HD]);
    }

    #[test]
    fn viewfinder_start_fails_like_the_scripted_error() {
        let mut camera = FakeCamera::failing(CameraError::InUse);
        assert_eq!(camera.start_viewfinder(HD), Err(CameraError::InUse));
        assert!(camera.started_at.is_empty());
    }
}
