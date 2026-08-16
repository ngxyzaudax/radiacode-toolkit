use egui::{Context, ViewportCommand, ViewportId};

use crate::icon::app_icon;

pub struct StartupChrome {
    pub theme_ready: bool,
    pub startup_scan_sent: bool,
    pub icon_sent: bool,
    pub window_size_frames: u8,
}

impl StartupChrome {
    pub fn new() -> Self {
        Self {
            theme_ready: false,
            startup_scan_sent: false,
            icon_sent: false,
            window_size_frames: 0,
        }
    }

    pub fn ensure_window_icon(&mut self, ctx: &Context) {
        if self.icon_sent {
            return;
        }
        ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Icon(Some(app_icon())));
        self.icon_sent = true;
    }

    pub fn ensure_startup_window_size(&mut self, ctx: &Context) {
        let frames = crate::window::startup_resize_frames();
        if self.window_size_frames >= frames {
            return;
        }
        ctx.send_viewport_cmd_to(
            ViewportId::ROOT,
            ViewportCommand::InnerSize(crate::window::startup_inner_vec()),
        );
        self.window_size_frames += 1;
    }
}
