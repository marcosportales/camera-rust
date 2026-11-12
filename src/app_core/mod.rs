//! Pure app core: state, events and the camera backend seam. No GStreamer, no GPUI.

pub mod backend;
#[cfg(test)]
mod fake;
pub mod resolution;
pub mod state;
