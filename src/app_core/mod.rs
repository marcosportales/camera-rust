//! Pure app core: state, events and the camera backend seam. No GStreamer, no GPUI.

pub mod auto_hide;
pub mod backend;
pub mod controls;
pub mod countdown;
#[cfg(test)]
mod fake;
pub mod lifecycle;
pub mod recording;
pub mod resolution;
pub mod state;
pub mod tooltips;

pub mod retry;
