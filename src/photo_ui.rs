//! Visuals of taking a Photo: the Shutter, the flash and the toast.

use std::time::Duration;

use gpui::{
    Animation, AnimationExt, BoxShadow, Div, FontWeight, IntoElement, Rgba, SharedString, Stateful,
    div, point, prelude::*, px, rgba, svg,
};

use crate::app_core::state::{CaptureOutcome, Mode};
use crate::capture_store::display_folder;
use crate::theme::{color, font, motion, radius, size, space};

/// Distance of the toast from the top edge, in logical px (the design's value; no spacing token).
const TOAST_TOP: f32 = 20.0;

/// How long the flash plays in total.
pub const FLASH_DURATION: Duration = Duration::from_millis(
    motion::FLASH_IN.as_millis() as u64 + motion::FLASH_OUT.as_millis() as u64,
);

/// How long the toast is on screen in total: fade in, hold, fade out.
pub const TOAST_DURATION: Duration = Duration::from_millis(
    motion::CONTROLS_IN.as_millis() as u64
        + motion::TOAST_HOLD.as_millis() as u64
        + motion::CONTROLS_OUT.as_millis() as u64,
);

/// Opacity (0 to 1) at `progress` (0 to 1) of an effect that fades in, holds at its peak, then fades out.
fn envelope(progress: f32, fade_in: Duration, hold: Duration, fade_out: Duration) -> f32 {
    let total = (fade_in + hold + fade_out).as_secs_f32();
    let elapsed = progress.clamp(0.0, 1.0) * total;
    let (fade_in, hold) = (fade_in.as_secs_f32(), hold.as_secs_f32());
    if elapsed < fade_in {
        elapsed / fade_in
    } else if elapsed < fade_in + hold {
        1.0
    } else {
        1.0 - (elapsed - fade_in - hold) / (total - fade_in - hold)
    }
}

/// A full-window `color::FLASH` overlay that peaks after `FLASH_IN` and is gone after `FLASH_OUT` more.
/// A new `id` restarts the animation.
pub fn flash(id: u64) -> impl IntoElement {
    div()
        .absolute()
        .size_full()
        .bg(rgba(color::FLASH))
        .with_animation(
            ("flash", id),
            Animation::new(FLASH_DURATION),
            |overlay, progress| {
                overlay.opacity(envelope(
                    progress,
                    motion::FLASH_IN,
                    Duration::ZERO,
                    motion::FLASH_OUT,
                ))
            },
        )
}

/// The pill at the top center telling the user how the last Capture went. A new `id` restarts its fade.
pub fn toast(id: u64, outcome: &CaptureOutcome) -> impl IntoElement {
    const SAVED: &str = "icons/check.svg";
    const FAILED: &str = "icons/x.svg";
    let (icon, text, folder) = match outcome {
        CaptureOutcome::PhotoSaved(path) => {
            (SAVED, "Photo saved", path.parent().map(display_folder))
        }
        CaptureOutcome::VideoSaved(path) => {
            (SAVED, "Video saved", path.parent().map(display_folder))
        }
        CaptureOutcome::PhotoFailed => (FAILED, "Couldn't save photo", None),
        CaptureOutcome::VideoFailed => (FAILED, "Couldn't save video", None),
    };
    let pill = div()
        .h(px(size::STATUS))
        .pl(px(10.0))
        .pr(px(space::S12))
        .flex()
        .items_center()
        .gap(px(space::S8))
        .rounded(px(radius::FULL))
        .bg(rgba(color::OVERLAY))
        .border_1()
        .border_color(rgba(color::BORDER_SUBTLE))
        .shadow(vec![BoxShadow {
            color: rgba(color::SHADOW_OVERLAY).into(),
            offset: point(px(0.0), px(8.0)),
            blur_radius: px(24.0),
            spread_radius: px(0.0),
        }])
        .text_size(px(font::SMALL))
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgba(color::TEXT_PRIMARY))
        .child(
            svg()
                .path(icon)
                .size(px(size::ICON_MD))
                .text_color(rgba(color::TEXT_PRIMARY)),
        )
        .child(text)
        .children(folder.map(|folder| {
            div()
                .text_color(rgba(color::TEXT_TERTIARY))
                .child(SharedString::from(folder))
        }))
        .with_animation(
            ("toast", id),
            Animation::new(TOAST_DURATION),
            |pill, progress| {
                pill.opacity(envelope(
                    progress,
                    motion::CONTROLS_IN,
                    motion::TOAST_HOLD,
                    motion::CONTROLS_OUT,
                ))
            },
        );
    div()
        .absolute()
        .top(px(TOAST_TOP))
        .left_0()
        .right_0()
        .flex()
        .justify_center()
        .child(pill)
}

/// How the Shutter fill reacts to the pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutterFill {
    Rest,
    Hover,
    Pressed,
}

/// The Shutter fill for a Mode and pointer state: white for Photo, red for Video.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShutterLook {
    pub mode: Mode,
    pub fill: ShutterFill,
}

impl ShutterLook {
    /// The design token for this look.
    fn color(self) -> u32 {
        match (self.mode, self.fill) {
            (Mode::Photo, ShutterFill::Rest) => color::SHUTTER_FILL,
            (Mode::Photo, ShutterFill::Hover) => color::SHUTTER_FILL_HOVER,
            (Mode::Photo, ShutterFill::Pressed) => color::SHUTTER_FILL_PRESSED,
            (Mode::Video, ShutterFill::Rest) => color::RECORD,
            (Mode::Video, ShutterFill::Hover) => color::RECORD_HOVER,
            (Mode::Video, ShutterFill::Pressed) => color::RECORD_PRESSED,
        }
    }
}

/// A fade of the Shutter fill from one look to another; a new `epoch` restarts the animation.
#[derive(Debug, Clone, Copy)]
pub struct FillTransition {
    from: ShutterLook,
    to: ShutterLook,
    epoch: usize,
}

impl FillTransition {
    /// A Shutter at rest in Photo Mode.
    pub fn new() -> Self {
        let rest = ShutterLook {
            mode: Mode::Photo,
            fill: ShutterFill::Rest,
        };
        Self {
            from: rest,
            to: rest,
            epoch: 0,
        }
    }

    /// Start fading to `target`; returns whether anything changed.
    pub fn retarget(&mut self, target: ShutterLook) -> bool {
        let changed = target != self.to;
        if changed {
            *self = Self {
                from: self.to,
                to: target,
                epoch: self.epoch + 1,
            };
        }
        changed
    }

    /// A Mode switch fades over `motion::MODE_SWITCH`, a hover or press over `motion::HOVER`.
    fn duration(&self) -> Duration {
        if self.from.mode == self.to.mode {
            motion::HOVER
        } else {
            motion::MODE_SWITCH
        }
    }
}

/// Mix two `0xRRGGBBAA` colors; `t` is 0 for `from` and 1 for `to`.
pub fn mix(from: u32, to: u32, t: f32) -> Rgba {
    let (from, to) = (rgba(from), rgba(to));
    let lerp = |a: f32, b: f32| a + (b - a) * t;
    Rgba {
        r: lerp(from.r, to.r),
        g: lerp(from.g, to.g),
        b: lerp(from.b, to.b),
        a: lerp(from.a, to.a),
    }
}

/// The round Shutter fill, fading between its looks.
pub fn shutter_fill(fill: FillTransition) -> impl IntoElement {
    let (from, to, epoch, duration) = (fill.from, fill.to, fill.epoch, fill.duration());
    div()
        .size(px(size::SHUTTER_FILL))
        .rounded_full()
        .with_animation(
            ("shutter-fill", epoch),
            Animation::new(duration),
            move |center, progress| center.bg(mix(from.color(), to.color(), progress)),
        )
}

/// The Shutter button: a ring around `center`.
pub fn shutter(center: impl IntoElement) -> Stateful<Div> {
    div()
        .id("shutter")
        .size(px(size::SHUTTER))
        .rounded_full()
        .border(px(size::SHUTTER_RING))
        .border_color(rgba(color::SHUTTER_RING))
        .flex()
        .items_center()
        .justify_center()
        .child(center)
}

/// The Shutter greyed out with `color::TEXT_DISABLED`, while it cannot be used.
pub fn disabled_shutter() -> Stateful<Div> {
    let fill = div()
        .size(px(size::SHUTTER_FILL))
        .rounded_full()
        .bg(rgba(color::TEXT_DISABLED));
    shutter(fill).border_color(rgba(color::TEXT_DISABLED))
}

#[cfg(test)]
mod tests {
    use super::*;

    const IN: Duration = Duration::from_millis(100);
    const HOLD: Duration = Duration::from_millis(200);
    const OUT: Duration = Duration::from_millis(100);

    #[test]
    fn envelope_fades_in_holds_then_fades_out() {
        let at = |progress| envelope(progress, IN, HOLD, OUT);
        assert_eq!(at(0.0), 0.0);
        assert!((at(0.125) - 0.5).abs() < 1e-5);
        assert_eq!(at(0.5), 1.0);
        assert!((at(0.875) - 0.5).abs() < 1e-5);
        assert_eq!(at(1.0), 0.0);
    }

    #[test]
    fn envelope_without_a_hold_peaks_at_the_end_of_the_fade_in() {
        assert_eq!(envelope(0.1, IN, Duration::ZERO, OUT * 9), 1.0);
    }

    #[test]
    fn retarget_restarts_the_fade_only_when_the_look_changes() {
        let mut fill = FillTransition::new();
        let look = |mode, fill| ShutterLook { mode, fill };
        let rest = look(Mode::Photo, ShutterFill::Rest);
        let hover = look(Mode::Photo, ShutterFill::Hover);
        assert!(!fill.retarget(rest));
        assert!(fill.retarget(hover));
        assert_eq!((fill.from, fill.to, fill.epoch), (rest, hover, 1));
        assert!(!fill.retarget(hover));
    }

    #[test]
    fn a_mode_switch_fades_slower_than_a_hover() {
        let mut fill = FillTransition::new();
        fill.retarget(ShutterLook {
            mode: Mode::Photo,
            fill: ShutterFill::Hover,
        });
        assert_eq!(fill.duration(), motion::HOVER);
        fill.retarget(ShutterLook {
            mode: Mode::Video,
            fill: ShutterFill::Hover,
        });
        assert_eq!(fill.duration(), motion::MODE_SWITCH);
    }
}
