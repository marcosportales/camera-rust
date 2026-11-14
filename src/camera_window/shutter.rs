//! The Shutter of the Camera window: what pressing it does and how it looks.

use std::path::PathBuf;
use std::time::Duration;

use gpui::{Context, IntoElement, MouseButton, div, prelude::*, px};

use super::{CameraWindow, after};
use crate::app_core::backend::VideoError;
use crate::app_core::state::{CaptureOutcome, Event, ShutterAction, VideoCapture};
use crate::capture_store::{photos_folder, videos_folder};
use crate::photo_ui::{self, FLASH_DURATION, ShutterFill, ShutterLook, TOAST_DURATION};
use crate::recording_ui;
use crate::theme::space;

/// How often the elapsed time of a Video advances.
const TICK: Duration = Duration::from_secs(1);

impl CameraWindow {
    /// Take a Photo, or start or stop a Video, depending on the Mode and whether a Video is recording.
    pub(super) fn press_shutter(&mut self, cx: &mut Context<Self>) {
        match self.state.press_shutter(self.countdown_clock.now()) {
            Some(ShutterAction::TakePhoto) => self.take_photo(cx),
            Some(ShutterAction::StartRecording) => self.start_recording(cx),
            Some(ShutterAction::StopRecording) => self.stop_recording(cx),
            None => self.sync_countdown(cx),
        }
    }

    /// Take a Photo; once it is written, flash and toast the outcome. A failure only toasts.
    pub(super) fn take_photo(&mut self, cx: &mut Context<Self>) {
        self.state.take_photo(&mut self.camera, &photos_folder());
        let saved = matches!(self.state.last_capture, Some(CaptureOutcome::PhotoSaved(_)));
        let id = self.show_feedback(saved, cx);
        if saved {
            after(cx, FLASH_DURATION, move |view, cx| view.end_flash(id, cx));
        }
        self.update_shutter_fill(cx);
    }

    /// Toast the last Capture's outcome, with a flash if `flash`, and remove the toast once it has
    /// played; returns the feedback's id.
    fn show_feedback(&mut self, flash: bool, cx: &mut Context<Self>) -> u64 {
        let id = self.feedback.begin(flash);
        after(cx, TOAST_DURATION, move |view, cx| {
            if view.feedback.end_toast(id) {
                cx.notify();
            }
        });
        id
    }

    /// Start a Video and advance its elapsed time every second until it stops; a failed start toasts.
    fn start_recording(&mut self, cx: &mut Context<Self>) {
        self.state
            .start_recording(&mut self.camera, &videos_folder());
        if self.state.recording().is_none() {
            self.show_feedback(false, cx);
        } else {
            self.recording_clock = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor().timer(TICK).await;
                    let ticked = this.update(cx, |view, cx| {
                        view.state.handle(Event::RecordingTick);
                        cx.notify();
                    });
                    if ticked.is_err() {
                        break;
                    }
                }
            }));
        }
        cx.notify();
    }

    /// Stop the Video and its ticking, then wait in the background until its file is finalized.
    fn stop_recording(&mut self, cx: &mut Context<Self>) {
        let saving = self.state.stop_recording(&mut self.camera);
        self.recording_clock = None;
        if let Some(saving) = saving {
            cx.spawn(async move |this, cx| {
                // A dropped sender means the finalizing thread died: the Video is not saved.
                let outcome = saving.await.unwrap_or(Err(VideoError));
                let _ = this.update(cx, |view, cx| view.finish_saving(outcome, cx));
            })
            .detach();
        }
        cx.notify();
    }

    /// The stopped Video is finalized (or failed): toast it; the Shutter returns to its Video Mode look.
    fn finish_saving(&mut self, outcome: Result<PathBuf, VideoError>, cx: &mut Context<Self>) {
        self.state.handle(outcome.into());
        self.show_feedback(false, cx);
        cx.notify();
    }

    /// Remove flash `id`, unless a newer Photo has replaced it.
    pub(super) fn end_flash(&mut self, id: u64, cx: &mut Context<Self>) {
        if self.feedback.end_flash(id) {
            self.update_shutter_fill(cx);
        }
    }

    /// The mouse button is no longer held on the Shutter.
    fn release_shutter(&mut self, cx: &mut Context<Self>) {
        self.shutter_pressed = false;
        self.update_shutter_fill(cx);
    }

    /// Fade the Shutter fill to the look that matches the Mode and the flash, press and hover state.
    pub(super) fn update_shutter_fill(&mut self, cx: &mut Context<Self>) {
        let fill = if self.feedback.flash.is_some() || self.shutter_pressed {
            ShutterFill::Pressed
        } else if self.shutter_hovered {
            ShutterFill::Hover
        } else {
            ShutterFill::Rest
        };
        let mode = self.state.mode;
        // Also redraws the stop square, whose pressed color is not part of the fill transition.
        self.shutter_fill.retarget(ShutterLook { mode, fill });
        cx.notify();
    }

    /// The Shutter at the bottom center: a stop square while recording, greyed out while saving, else
    /// the Mode's fill.
    pub(super) fn shutter(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let button = match self.state.video {
            _ if self.state.countdown_numeral().is_some() => {
                photo_ui::shutter(crate::countdown_ui::cancel_fill())
            }
            Some(VideoCapture::Recording(_)) => {
                photo_ui::shutter(recording_ui::stop_square(self.shutter_pressed))
            }
            Some(VideoCapture::Saving) => photo_ui::disabled_shutter(),
            None => photo_ui::shutter(photo_ui::shutter_fill(self.shutter_fill)),
        };
        let button = button
            .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                this.shutter_hovered = *hovered;
                this.update_shutter_fill(cx);
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.shutter_pressed = true;
                    this.update_shutter_fill(cx);
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.release_shutter(cx)),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.release_shutter(cx)),
            )
            .on_click(cx.listener(|this, _, _, cx| this.press_shutter(cx)));
        div()
            .absolute()
            .bottom(px(space::S24))
            .left_0()
            .right_0()
            .flex()
            .justify_center()
            .child(button)
    }
}
