//! Visuals of recording a Video: the timecode pill, the Shutter's stop square and the saving label.

use std::f32::consts::TAU;
use std::sync::Arc;

use gpui::{
    Animation, AnimationExt, Font, FontFeatures, FontStyle, FontWeight, IntoElement, SharedString,
    div, prelude::*, px, rgba,
};

use crate::app_core::state::Recording;
use crate::overlays::shadow_control;
use crate::theme::{color, font, motion, radius, size, space};

/// Distance of the timecode pill from the top edge, in logical px (the design's value; no spacing token).
const TIMECODE_TOP: f32 = 20.0;
/// Opacity of the recording dot at the dim end of its pulse.
const REC_DOT_DIM: f32 = 0.35;

/// Opacity of the recording dot at `progress` (0 to 1) through one pulse: 1 → `REC_DOT_DIM` → 1.
fn dot_opacity(progress: f32) -> f32 {
    // A cosine eases in and out at both ends of the pulse.
    let closeness_to_full = (1.0 + (TAU * progress).cos()) / 2.0;
    REC_DOT_DIM + (1.0 - REC_DOT_DIM) * closeness_to_full
}

/// `type.timecode`: Inter 13 / 600 with tabular figures, so the digits do not shift as they change.
fn timecode_font() -> Font {
    Font {
        family: font::FAMILY.into(),
        features: FontFeatures(Arc::new(vec![("tnum".into(), 1)])),
        fallbacks: None,
        weight: FontWeight::SEMIBOLD,
        style: FontStyle::Normal,
    }
}

/// The pill at the top center with a pulsing red dot and the elapsed time as `MM:SS`.
pub fn timecode_pill(recording: Recording) -> impl IntoElement {
    let dot = div()
        .size(px(size::REC_DOT))
        .rounded(px(radius::FULL))
        .bg(rgba(color::RECORD))
        .with_animation(
            "rec-dot",
            Animation::new(motion::REC_PULSE).repeat(),
            |dot, progress| dot.opacity(dot_opacity(progress)),
        );
    let pill = div()
        .h(px(size::STATUS))
        .px(px(space::S12))
        .flex()
        .items_center()
        .gap(px(space::S8))
        .rounded(px(radius::FULL))
        .bg(rgba(color::CONTROL))
        .shadow(shadow_control())
        .font(timecode_font())
        .text_size(px(font::LABEL))
        .text_color(rgba(color::TEXT_PRIMARY))
        .child(dot)
        .child(SharedString::from(recording.timecode()));
    div()
        .absolute()
        .top(px(TIMECODE_TOP))
        .left_0()
        .right_0()
        .flex()
        .justify_center()
        .child(pill)
}

/// The Shutter's center while recording: a red stop square, darker while `pressed`.
pub fn stop_square(pressed: bool) -> impl IntoElement {
    let fill = if pressed {
        color::RECORD_PRESSED
    } else {
        color::RECORD
    };
    div()
        .size(px(size::STOP))
        .rounded(px(radius::STOP))
        .bg(rgba(fill))
}

/// "Saving…" in `type.small`, centered `space.16` above the Shutter while a stopped Video is finalized.
pub fn saving_label() -> impl IntoElement {
    div()
        .absolute()
        .bottom(px(space::S24 + size::SHUTTER + space::S16))
        .left_0()
        .right_0()
        .flex()
        .justify_center()
        .text_size(px(font::SMALL))
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgba(color::TEXT_SECONDARY))
        .child("Saving…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dot_pulses_from_full_to_dim_and_back_over_one_cycle() {
        assert_eq!(dot_opacity(0.0), 1.0);
        assert!((dot_opacity(0.5) - REC_DOT_DIM).abs() < 1e-5);
        assert!((dot_opacity(1.0) - 1.0).abs() < 1e-5);
        assert!(dot_opacity(0.25) < 1.0 && dot_opacity(0.25) > REC_DOT_DIM);
    }
}
