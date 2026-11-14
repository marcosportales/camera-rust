//! Pure app core: state, events and the camera backend seam. No GStreamer, no GPUI.

pub mod backend;
pub mod countdown;
#[cfg(test)]
mod fake;
pub mod recording;
pub mod resolution;
pub mod state;
