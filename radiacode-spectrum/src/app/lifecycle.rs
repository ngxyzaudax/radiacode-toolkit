use egui::{Context, ViewportCommand};
use tracing::debug;

use super::SpectrumApp;
use crate::spectrum::{CloseAction, quit_hotkey_pressed};
use crate::view_tab::ViewTab;

impl SpectrumApp {
    pub(super) fn handle_close_request(&mut self, ctx: &Context) {
        let window_close = ctx.input(|input| input.viewport().close_requested());
        let hotkey_quit = quit_hotkey_pressed(ctx);
        if !window_close && !hotkey_quit {
            return;
        }
        let action = self
            .shutdown
            .on_close_request(true, self.device_link_active());
        self.apply_close_action(ctx, action, window_close);
    }

    fn apply_close_action(&mut self, ctx: &Context, action: CloseAction, window_close: bool) {
        match action {
            CloseAction::None => {}
            CloseAction::DisconnectDevice => {
                if window_close {
                    ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                }
                self.disconnect_device();
            }
            CloseAction::CompleteClose => {
                if !window_close {
                    self.shutdown.send_close_viewport(ctx);
                }
            }
        }
    }

    pub(super) fn advance_close(&mut self, ctx: &Context) {
        if matches!(self.shutdown.advance_close(), CloseAction::CompleteClose) {
            self.shutdown.send_close_viewport(ctx);
        }
    }

    pub(super) fn apply_ui_scale(&self, ctx: &Context) {
        let scale = self.settings.app.ui_scale;
        if (ctx.zoom_factor() - scale).abs() > f32::EPSILON {
            ctx.set_zoom_factor(scale);
        }
    }

    pub(super) fn ensure_chrome(&mut self, ctx: &Context) {
        self.chrome.ensure_window_icon(ctx);
        self.chrome.ensure_startup_window_size(ctx);
        self.apply_ui_scale(ctx);
        if !self.chrome.theme_ready {
            crate::theme::apply(ctx);
            self.chrome.theme_ready = true;
            self.sync_monitor_poll_interval();
            self.sync_capture_interval();
        }
        if !self.chrome.startup_scan_sent {
            self.chrome.startup_scan_sent = true;
            self.start_scan();
        }
    }

    pub(super) fn maybe_live_refresh(&mut self) {
        if self.shutting_down() || !self.is_connected() {
            return;
        }
        let spectrum_tab = self.tabs.active == ViewTab::Spectrum;
        if spectrum_tab
            && self
                .state
                .live_refresh_due(true, self.settings.app.spectrum_refresh_secs)
        {
            debug!("spectrum tab live refresh due");
            self.schedule_spectrum();
        }
    }
}
