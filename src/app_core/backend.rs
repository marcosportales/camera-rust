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

/// What the core needs from a camera; GStreamer implements it, tests use a fake.
pub trait CameraBackend {
    /// Find the first capture-capable camera.
    fn detect_cameras(&mut self) -> Result<Camera, CameraError>;

    /// Start the Viewfinder at the given Resolution.
    fn start_viewfinder(&mut self, resolution: Resolution) -> Result<(), CameraError>;
}
