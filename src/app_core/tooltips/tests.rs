use super::{Control, Hint};
use crate::app_core::state::{AppState, Event, Mode, ShutterAction};
use std::time::Duration;

/// Idle controls expose the exact names and shortcuts from ticket 16.
#[test]
fn shortcut_tooltips_describe_idle_controls() {
    let state = AppState::new();
    for (control, label, key) in [
        (Control::Shutter, "Take photo", Some("Space")),
        (Control::Photo, "Photo mode", Some("P")),
        (Control::Video, "Video mode", Some("V")),
        (Control::Timer, "Timer", Some("T")),
        (Control::Mirror, "Mirror", Some("M")),
        (Control::Resolution, "Resolution", None),
        (Control::Thumbnail, "Open folder", None),
    ] {
        assert_eq!(state.shortcut_hint(control), Some(Hint { label, key }));
    }
    assert_eq!(state.shortcut_hint(Control::CycleTimer), None);
}

/// The Shutter describes the action available now, including cancellation and finalizing.
#[test]
fn shortcut_tooltips_follow_shutter_capture_states() {
    let mut state = AppState::new();
    state.handle(Event::SetMode(Mode::Video));
    assert_eq!(
        state.shortcut_hint(Control::Shutter),
        Some(Hint {
            label: "Record video",
            key: Some("Space"),
        })
    );
    for action in [ShutterAction::TakePhoto, ShutterAction::StartRecording] {
        state.handle(Event::StartCountdown {
            action,
            now: Duration::ZERO,
            seconds: 3,
        });
        assert_eq!(
            state.shortcut_hint(Control::Shutter),
            Some(Hint {
                label: "Cancel",
                key: Some("Esc"),
            })
        );
        state.handle(Event::CancelCountdown);
    }
    state.handle(Event::RecordingStarted);
    assert_eq!(
        state.shortcut_hint(Control::Shutter),
        Some(Hint {
            label: "Stop recording",
            key: Some("Space"),
        })
    );
    state.handle(Event::RecordingStopped);
    assert_eq!(state.shortcut_hint(Control::Shutter), None);
    state.handle(Event::VideoSaved("video.mp4".into()));
    assert_eq!(
        state.shortcut_hint(Control::Shutter),
        Some(Hint {
            label: "Record video",
            key: Some("Space"),
        })
    );
}

/// Locked settings reserve their hover for ticket 12's explanation and recover afterward.
#[test]
fn shortcut_tooltips_yield_to_capture_locks() {
    let mut state = AppState::new();
    for (event, message) in [
        (
            Event::StartCountdown {
                action: ShutterAction::TakePhoto,
                now: Duration::ZERO,
                seconds: 3,
            },
            "Unavailable during countdown",
        ),
        (Event::RecordingStarted, "Unavailable while recording"),
    ] {
        state.handle(event);
        assert_eq!(state.control_lock().unwrap().message(), message);
        for control in [
            Control::Photo,
            Control::Video,
            Control::Timer,
            Control::Mirror,
            Control::Resolution,
        ] {
            assert_eq!(state.shortcut_hint(control), None);
        }
        state.handle(Event::CancelCountdown);
    }
    state.handle(Event::RecordingStopped);
    for control in [
        Control::Photo,
        Control::Video,
        Control::Timer,
        Control::Mirror,
        Control::Resolution,
    ] {
        assert!(state.shortcut_hint(control).is_some());
    }
}

/// Menus suppress covered controls, retaining the exposed top-row button and cycle hint.
#[test]
fn shortcut_tooltips_respect_menu_coverage() {
    let mut state = AppState::new();
    for event in [Event::ToggleTimerMenu, Event::ToggleResolutionMenu] {
        state.handle(event);
        for control in [
            Control::Shutter,
            Control::Photo,
            Control::Video,
            Control::Mirror,
            Control::Thumbnail,
        ] {
            assert_eq!(state.shortcut_hint(control), None);
        }
        if state.timer_menu_open {
            assert_eq!(state.shortcut_hint(Control::Timer), None);
            assert_eq!(
                state.shortcut_hint(Control::Resolution),
                Some(Hint {
                    label: "Resolution",
                    key: None
                })
            );
            assert_eq!(
                state.shortcut_hint(Control::CycleTimer),
                Some(Hint {
                    label: "Cycle timer",
                    key: Some("T")
                })
            );
        } else {
            assert_eq!(state.shortcut_hint(Control::Resolution), None);
            assert_eq!(
                state.shortcut_hint(Control::Timer),
                Some(Hint {
                    label: "Timer",
                    key: Some("T")
                })
            );
            assert_eq!(state.shortcut_hint(Control::CycleTimer), None);
        }
    }
    state.handle(Event::CloseResolutionMenu);
    assert_eq!(
        state.shortcut_hint(Control::Shutter),
        Some(Hint {
            label: "Take photo",
            key: Some("Space")
        })
    );
}
