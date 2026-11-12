//! Visuals of taking a Photo: the Shutter, the flash and the toast.

use std::time::Duration;

use gpui::{
    Animation, AnimationExt, BoxShadow, Div, FontWeight, IntoElement, Rgba, SharedString, Stateful,
    div, point, prelude::*, px, rgba, svg,
};

use crate::app_core::state::PhotoOutcome;
use crate::photo_store::display_folder;
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

/// The pill at the top center telling the user how the last Photo went. A new `id` restarts its fade.
pub fn toast(id: u64, outcome: &PhotoOutcome) -> impl IntoElement {
    let (icon, text, folder) = match outcome {
        PhotoOutcome::Saved(path) => (
            "icons/check.svg",
            "Photo saved",
            path.parent().map(display_folder),
        ),
        PhotoOutcome::Failed => ("icons/x.svg", "Couldn't save photo", None),
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

/// How the Shutter fill looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutterFill {
    Rest,
    Hover,
    Pressed,
}

impl ShutterFill {
    /// The design token for this look.
    fn color(self) -> u32 {
        match self {
            Self::Rest => color::SHUTTER_FILL,
            Self::Hover => color::SHUTTER_FILL_HOVER,
            Self::Pressed => color::SHUTTER_FILL_PRESSED,
        }
    }
}

/// A fade of the Shutter fill from one look to another; a new `epoch` restarts the animation.
#[derive(Debug, Clone, Copy)]
pub struct FillTransition {
    from: ShutterFill,
    to: ShutterFill,
    epoch: usize,
}

impl FillTransition {
    /// A Shutter at rest.
    pub fn new() -> Self {
        Self {
            from: ShutterFill::Rest,
            to: ShutterFill::Rest,
            epoch: 0,
        }
    }

    /// Start fading to `target`; returns whether anything changed.
    pub fn retarget(&mut self, target: ShutterFill) -> bool {
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
}

/// Mix two `0xRRGGBBAA` colors; `t` is 0 for `from` and 1 for `to`.
fn mix(from: u32, to: u32, t: f32) -> Rgba {
    let (from, to) = (rgba(from), rgba(to));
    let lerp = |a: f32, b: f32| a + (b - a) * t;
    Rgba {
        r: lerp(from.r, to.r),
        g: lerp(from.g, to.g),
        b: lerp(from.b, to.b),
        a: lerp(from.a, to.a),
    }
}

/// The Shutter button: a ring around a fill that fades between its looks over `motion::HOVER`.
pub fn shutter(fill: FillTransition) -> Stateful<Div> {
    let FillTransition { from, to, epoch } = fill;
    let center = div()
        .size(px(size::SHUTTER_FILL))
        .rounded_full()
        .with_animation(
            ("shutter-fill", epoch),
            Animation::new(motion::HOVER),
            move |center, progress| center.bg(mix(from.color(), to.color(), progress)),
        );
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
        assert!(!fill.retarget(ShutterFill::Rest));
        assert!(fill.retarget(ShutterFill::Hover));
        assert_eq!(
            (fill.from, fill.to, fill.epoch),
            (ShutterFill::Rest, ShutterFill::Hover, 1)
        );
        assert!(!fill.retarget(ShutterFill::Hover));
    }
}
