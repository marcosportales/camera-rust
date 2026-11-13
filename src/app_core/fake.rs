use std::path::{Path, PathBuf};

use super::backend::{Camera, CameraBackend, CameraError, PhotoError, VideoError};
use super::resolution::Resolution;

/// Scriptable stand-in for the real camera.
pub struct FakeCamera {
    detection: Result<Camera, CameraError>,
    /// Resolutions the Viewfinder was started at, in order.
    pub started_at: Vec<Resolution>,
    photo: Result<PathBuf, PhotoError>,
    /// Folders Photos were requested for, in order.
    pub photo_folders: Vec<PathBuf>,
    video: Result<PathBuf, VideoError>,
    /// Folders Videos were started in, in order.
    pub video_folders: Vec<PathBuf>,
    /// How many times a Video was stopped.
    pub videos_stopped: usize,
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

    /// Script every Photo to be saved at `path`.
    pub fn saving_photo_as(mut self, path: &Path) -> Self {
        self.photo = Ok(path.to_path_buf());
        self
    }

    /// Script every Photo to fail.
    pub fn failing_photo(mut self) -> Self {
        self.photo = Err(PhotoError);
        self
    }

    /// Script every Video to fail to start.
    pub fn failing_video(mut self) -> Self {
        self.video = Err(VideoError);
        self
    }

    /// Build a fake that answers detection with the scripted result.
    fn scripted(detection: Result<Camera, CameraError>) -> Self {
        Self {
            detection,
            started_at: Vec::new(),
            photo: Ok(PathBuf::from("photo.jpg")),
            photo_folders: Vec::new(),
            video: Ok(PathBuf::from("video.mp4")),
            video_folders: Vec::new(),
            videos_stopped: 0,
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

    /// Record the folder and return the scripted Photo result.
    fn take_photo(&mut self, folder: &Path) -> Result<PathBuf, PhotoError> {
        self.photo_folders.push(folder.to_path_buf());
        self.photo.clone()
    }

    /// Record the folder and return the scripted Video result.
    fn start_video(&mut self, folder: &Path) -> Result<PathBuf, VideoError> {
        self.video_folders.push(folder.to_path_buf());
        self.video.clone()
    }

    /// Count the stop.
    fn stop_video(&mut self) {
        self.videos_stopped += 1;
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
