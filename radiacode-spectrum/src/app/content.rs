use egui::{Context, Ui};

use super::SpectrumApp;
use crate::about::draw_about_view;
use crate::catalogue::draw_catalogue_view;
use crate::compare::draw_compare_view;
use crate::device::{DeviceViewProps, draw_device_view};
use crate::layout::page_scroll;
use crate::settings::draw_settings_view;
use crate::ui_disconnected::{draw_disconnected_view, shows_tab_content};
use crate::view_tab::ViewTab;

impl SpectrumApp {
    pub(super) fn draw_central_content(&mut self, ui: &mut Ui, ctx: &Context) {
        match self.tabs.active {
            ViewTab::Device => self.draw_device_tab(ui),
            ViewTab::Settings => self.draw_settings_tab(ui),
            ViewTab::About => page_scroll(ui, "about_page", draw_about_view),
            ViewTab::Compare => self.draw_compare_tab(ui),
            ViewTab::Catalogue => self.draw_catalogue_tab(ui),
            ViewTab::Monitor | ViewTab::Spectrum | ViewTab::Spectrogram => {
                self.draw_live_tab(ui, ctx)
            }
        }
    }

    fn draw_device_tab(&mut self, ui: &mut Ui) {
        let props = DeviceViewProps {
            devices: &self.state.devices,
            connection: self.state.connection,
            connecting_endpoint: self.state.connecting_endpoint.as_ref(),
            device_info: self.state.device_info.as_ref(),
            remembered_endpoint: self.settings.app.last_endpoint.as_ref(),
            scanning: self.state.scanning,
            busy: self.state.busy,
            scanned_once: self.state.scanned_once,
            status: &self.state.status,
            link_health: self.state.monitor.link_health(),
            last_spectrum_fetch: self.state.last_fetch,
            last_monitor_fetch: self.state.last_monitor_fetch,
        };
        if let Some(action) = draw_device_view(ui, props) {
            self.handle_device_action(action);
        }
    }

    fn draw_settings_tab(&mut self, ui: &mut Ui) {
        let action = draw_settings_view(
            ui,
            &mut self.settings,
            self.state.connection,
            self.state.device_info.as_ref(),
            self.spectrogram.is_recording(),
        );
        if let Some(action) = action {
            self.handle_settings_action(action);
        }
    }

    fn draw_compare_tab(&mut self, ui: &mut Ui) {
        let action = draw_compare_view(
            ui,
            &mut self.compare,
            &mut self.view.y_scale,
            &self.settings.app,
        );
        if let Some(action) = action {
            self.handle_spectrum_plot_action(action);
        }
    }

    fn draw_catalogue_tab(&mut self, ui: &mut Ui) {
        if draw_catalogue_view(ui, &mut self.catalogue, &mut self.settings.app) {
            self.settings.persist_app();
        }
    }

    fn draw_live_tab(&mut self, ui: &mut Ui, ctx: &Context) {
        if !shows_tab_content(self.state.connection) {
            draw_disconnected_view(ui, self.state.connection);
            return;
        }
        match self.tabs.active {
            ViewTab::Monitor => self.draw_monitor_tab(ui),
            ViewTab::Spectrum => self.draw_spectrum_tab(ui),
            ViewTab::Spectrogram => self.draw_spectrogram_tab(ui, ctx),
            _ => {}
        }
    }
}
