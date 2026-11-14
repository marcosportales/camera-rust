//! Window-close interception and camera-loss delivery.

use super::CameraWindow;
use gpui::{Context, Window};
use std::time::Duration;

impl CameraWindow {
    /// Keep the window alive during EOS and periodically consume camera failures.
    pub(super) fn watch_lifecycle(window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.entity().downgrade();
        window.on_window_should_close(cx, move |_, cx| {
            view.update(cx, |view, cx| {
                let saving = view.state.request_close(&mut view.camera);
                view.await_saving(saving, cx);
                view.state.can_exit()
            })
            .unwrap_or(true)
        });
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                if this
                    .update(cx, |view, cx| {
                        if let Some(error) = view.camera.poll_camera_error() {
                            let saving = view.state.camera_lost(&mut view.camera, error);
                            view.await_saving(saving, cx);
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }
}
