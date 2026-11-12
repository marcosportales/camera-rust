//! Bookkeeping of the transient feedback of the Camera window: the Photo flash and toast, and the Mode switch.

use crate::app_core::state::Mode;

/// Which flash and toast are on screen. Ids grow with every Photo, so each one restarts its animation.
#[derive(Debug, Default)]
pub struct CaptureFeedback {
    pub flash: Option<u64>,
    /// The toast's text comes from `AppState::photo`.
    pub toast: Option<u64>,
    photos_taken: u64,
}

impl CaptureFeedback {
    /// Show the toast (and the flash, if the Photo was `saved`) of a new Photo; returns its id.
    pub fn begin(&mut self, saved: bool) -> u64 {
        self.photos_taken += 1;
        let id = self.photos_taken;
        if saved {
            self.flash = Some(id);
        }
        self.toast = Some(id);
        id
    }

    /// Remove flash `id`, unless a newer Photo has replaced it; returns whether it was removed.
    pub fn end_flash(&mut self, id: u64) -> bool {
        Self::end(&mut self.flash, id)
    }

    /// Remove toast `id`, unless a newer Photo has replaced it; returns whether it was removed.
    pub fn end_toast(&mut self, id: u64) -> bool {
        Self::end(&mut self.toast, id)
    }

    /// Clear `slot` if it still holds `id`.
    fn end(slot: &mut Option<u64>, id: u64) -> bool {
        let current = *slot == Some(id);
        if current {
            *slot = None;
        }
        current
    }
}

/// The Mode before the current one, and how many switches happened, so the pill can fade between the two.
#[derive(Debug)]
pub struct ModeTransition {
    pub previous: Mode,
    /// Each switch restarts the pill and Shutter fades.
    pub switches: usize,
}

impl ModeTransition {
    /// No switch yet; the previous Mode is the startup one.
    pub fn new() -> Self {
        Self {
            previous: Mode::Photo,
            switches: 0,
        }
    }

    /// Record a switch away from `from`.
    pub fn switched_from(&mut self, from: Mode) {
        self.previous = from;
        self.switches += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_newer_photo_keeps_its_flash_when_the_older_one_ends() {
        let mut feedback = CaptureFeedback::default();
        let first = feedback.begin(true);
        let second = feedback.begin(true);
        assert!(!feedback.end_flash(first));
        assert_eq!(feedback.flash, Some(second));
        assert!(feedback.end_flash(second));
        assert_eq!(feedback.flash, None);
    }

    #[test]
    fn a_failed_photo_toasts_without_a_flash() {
        let mut feedback = CaptureFeedback::default();
        let id = feedback.begin(false);
        assert_eq!((feedback.flash, feedback.toast), (None, Some(id)));
    }

    #[test]
    fn a_switch_records_the_previous_mode_and_counts() {
        let mut transition = ModeTransition::new();
        transition.switched_from(Mode::Photo);
        assert_eq!((transition.previous, transition.switches), (Mode::Photo, 1));
    }
}
