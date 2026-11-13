use std::path::{Path, PathBuf};

use futures::channel::oneshot;

use super::resolution::Resolution;

/// A capture-capable camera and the MJPG Resolutions it supports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Camera {
    pub name: String,
    pub resolutions: Vec<Resolution>,
}

/// Why the camera could not be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraError {
    /// Another application is using the camera.
    InUse,
    /// The system denied access to the camera.
    AccessDenied,
    /// No capture-capable camera was found.
    NoCamera,
}

/// The Photo could not be saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhotoError;

/// The Video could not be started or its file could not be finalized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoError;

/// Resolves once a stopped Video's file is finalized (end-of-stream reached, file closed), with its
/// path, or with the failure. A dropped sender means the finalizing side died: treat it as a failure.
pub type VideoSaving = oneshot::Receiver<Result<PathBuf, VideoError>>;

/// What the core needs from a camera; GStreamer implements it, tests use a fake.
pub trait CameraBackend {
    /// Find the first capture-capable camera.
    fn detect_cameras(&mut self) -> Result<Camera, CameraError>;

    /// Start the Viewfinder at the given Resolution.
    fn start_viewfinder(&mut self, resolution: Resolution) -> Result<(), CameraError>;

    /// Save the current Viewfinder frame as a JPEG in `folder`, returning the file's path.
    fn take_photo(&mut self, folder: &Path) -> Result<PathBuf, PhotoError>;

    /// Start recording a Video at the Viewfinder's Resolution into a new file in `folder`, returning its path.
    fn start_video(&mut self, folder: &Path) -> Result<PathBuf, VideoError>;

    /// Stop the Video being recorded and close its file with an end-of-stream, in the background.
    fn stop_video(&mut self) -> VideoSaving;
}
