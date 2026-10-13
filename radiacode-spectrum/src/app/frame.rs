use std::time::Duration;

use eframe::App;
use egui::{CentralPanel, Context, Ui, UiBuilder};

use super::SpectrumApp;
use crate::monitor::draw_monitor_leave_confirm;
use crate::tabs::draw_tab_bar;
use crate::ui_chrome::{tab_uses_page_inset, tab_uses_plot_pad, with_page_inset, with_plot_pad};
use crate::ui_disconnected::{shows_tab_content, tab_works_offline};
use crate::usb_access::draw_usb_access_dialog;
use crate::view_tab::ViewTab;

const SHUTDOWN_REPAINT: Duration = Duration::from_millis(16);
const LIVE_REPAINT: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContentPadding {
    PageInset,
    PlotPad,
    None,
}

pub(super) fn content_padding(tab: ViewTab, connected: bool) -> ContentPadding {
    let page_inset = tab_uses_page_inset(tab) || (!connected && !tab_works_offline(tab));
    let plot_pad = connected && tab_uses_plot_pad(tab) && tab != ViewTab::Monitor;
    if page_inset {
        ContentPadding::PageInset
    } else if plot_pad {
        ContentPadding::PlotPad
    } else {
        ContentPadding::None
    }
}

impl SpectrumApp {
    fn schedule_repaint(&self, ctx: &Context) {
        if self.shutting_down() {
            ctx.request_repaint_after(SHUTDOWN_REPAINT);
            if self.shutdown.closing() {
                self.shutdown.send_close_viewport(ctx);
            }
        } else if self.is_connected() || self.spectrogram.is_recording() {
            ctx.request_repaint_after(LIVE_REPAINT);
        }
    }

    fn draw_modals(&mut self, ctx: &Context) {
        if let Some(prompt) = self.usb_access_prompt.as_mut()
            && let Some(action) = draw_usb_access_dialog(ctx, prompt)
        {
            self.handle_usb_access_action(action);
        }
        if let Some(choice) = draw_monitor_leave_confirm(ctx, self.tabs.monitor_leave_open) {
            self.handle_monitor_leave_choice(choice);
        }
    }

    fn draw_tabs(&mut self, ui: &mut Ui) {
        let previous_tab = self.tabs.previous;
        if let Some(requested) = draw_tab_bar(ui, self.tabs.active) {
            self.try_switch_tab(requested);
        }
        self.on_tab_changed(previous_tab);
        self.tabs.previous = self.tabs.active;
    }

    fn draw_padded_content(&mut self, ui: &mut Ui, ctx: &Context) {
        let content_rect = ui.available_rect_before_wrap();
        ui.scope_builder(UiBuilder::new().max_rect(content_rect), |ui| {
            ui.set_clip_rect(content_rect);
            let connected = shows_tab_content(self.state.connection);
            match content_padding(self.tabs.active, connected) {
                ContentPadding::PageInset => {
                    with_page_inset(ui, |ui| self.draw_central_content(ui, ctx))
                }
                ContentPadding::PlotPad => {
                    with_plot_pad(ui, |ui| self.draw_central_content(ui, ctx))
                }
                ContentPadding::None => self.draw_central_content(ui, ctx),
            }
        });
    }
}

impl App for SpectrumApp {
    fn logic(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.ensure_chrome(ctx);
        self.handle_close_request(ctx);
        self.poll_events();
        self.advance_close(ctx);
        self.poll_usb_access();
        if self.is_connected() {
            self.spectrogram.sync_from_capture();
        }
        self.maybe_live_refresh();
        self.maybe_device_config_auto_load();
        self.schedule_repaint(ctx);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.draw_modals(&ctx);
        CentralPanel::default().show(ui, |ui| {
            self.draw_tabs(ui);
            ui.separator();
            self.draw_padded_content(ui, &ctx);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{ContentPadding, content_padding};
    use crate::view_tab::ViewTab;

    #[test]
    fn monitor_tab_never_uses_plot_pad() {
        assert_ne!(
            content_padding(ViewTab::Monitor, true),
            ContentPadding::PlotPad
        );
    }

    #[test]
    fn offline_live_tabs_fall_back_to_page_inset() {
        assert_eq!(
            content_padding(ViewTab::Spectrum, false),
            ContentPadding::PageInset
        );
    }
}
