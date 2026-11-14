//! Shortcut descriptions selected from the current capture and menu state.
use super::state::{AppState, Mode, VideoCapture};

/// A control with a discoverable action, including the future Last-Capture Thumbnail.
#[derive(Clone, Copy, Debug)]
pub enum Control {
    Shutter,
    Photo,
    Video,
    Timer,
    CycleTimer,
    Mirror,
    Resolution,
    Thumbnail,
}

/// Tooltip copy and an optional keyboard shortcut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hint {
    pub label: &'static str,
    pub key: Option<&'static str>,
}

impl AppState {
    /// Describe available controls; locked settings retain their existing lock explanation.
    pub fn shortcut_hint(&self, control: Control) -> Option<Hint> {
        if matches!(control, Control::CycleTimer) {
            return self.timer_menu_open.then_some(Hint {
                label: "Cycle timer",
                key: Some("T"),
            });
        }
        let covered_by_menu = match control {
            Control::Timer => self.timer_menu_open,
            Control::Resolution => self.resolution_menu_open,
            // The menu's full-window dismissal layer shields the bottom controls.
            _ => self.timer_menu_open || self.resolution_menu_open,
        };
        if covered_by_menu {
            return None;
        }
        if self.control_lock().is_some()
            && matches!(
                control,
                Control::Photo
                    | Control::Video
                    | Control::Timer
                    | Control::Mirror
                    | Control::Resolution
            )
        {
            return None;
        }
        self.available_hint(control)
    }

    /// Select copy for an unlocked control and the current Shutter action.
    fn available_hint(&self, control: Control) -> Option<Hint> {
        let (label, key) = match control {
            Control::Shutter if self.countdown.is_some() => ("Cancel", Some("Esc")),
            Control::Shutter => match self.video {
                Some(VideoCapture::Recording(_)) => ("Stop recording", Some("Space")),
                Some(VideoCapture::Saving) => return None,
                None if self.mode == Mode::Video => ("Record video", Some("Space")),
                None => ("Take photo", Some("Space")),
            },
            Control::Photo => ("Photo mode", Some("P")),
            Control::Video => ("Video mode", Some("V")),
            Control::Timer | Control::CycleTimer => ("Timer", Some("T")),
            Control::Mirror => ("Mirror", Some("M")),
            Control::Resolution => ("Resolution", None),
            Control::Thumbnail => ("Open folder", None),
        };
        Some(Hint { label, key })
    }
}

#[cfg(test)]
mod tests;
