use egui::{Context, Ui};

use super::SpectrumApp;
use crate::about::draw_about_view;
use crate::catalogue::draw_catalogue_view;
use crate::compare::draw_compare_view;
use crate::device::{DeviceViewProps, draw_device_view};
use crate::layout::page_scroll;
use crate::monitor::{MonitorViewProps, draw_monitor_view};
use crate::plot_style::histogram_style;
use crate::settings::draw_settings_view;
use crate::spectrogram::ui_view::draw_spectrogram_view;
use crate::ui_disconnected::{draw_disconnected_view, shows_tab_content};
use crate::ui_plot::{SpectrumPlotDrawContext, draw_spectrum_plot};
use crate::ui_toolbar::{SpectrumToolbarProps, draw_spectrum_toolbar};
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

    fn draw_monitor_tab(&mut self, ui: &mut Ui) {
        let plot_style = histogram_style(self.view.plot_outline_only);
        let action = draw_monitor_view(
            ui,
            &self.state.monitor,
            &self.state.dosimeter,
            plot_style,
            self.settings.app.monitor_smoothing_window,
            MonitorViewProps {
                settings: &mut self.settings,
                connection: self.state.connection,
                outline_only: &mut self.view.plot_outline_only,
            },
        );
        if let Some(action) = action {
            self.handle_monitor_view_action(action);
        }
    }

    fn draw_spectrum_tab(&mut self, ui: &mut Ui) {
        let plot_style = histogram_style(self.view.plot_outline_only);
        let toolbar_action = draw_spectrum_toolbar(
            ui,
            SpectrumToolbarProps {
                connection: self.state.connection,
                spectrum: self.state.spectrum.as_ref(),
                y_scale: &mut self.view.y_scale,
                smooth_window: &mut self.view.smooth_window,
                outline_only: &mut self.view.plot_outline_only,
                show_peaks: &mut self.view.show_spectrum_peaks,
            },
        );
        if let Some(action) = toolbar_action {
            self.handle_spectrum_toolbar_action(action);
        }
        let plot_action = draw_spectrum_plot(
            ui,
            self.state.spectrum.as_ref(),
            self.view.y_scale,
            self.view.smooth_window,
            plot_style,
            self.view.show_spectrum_peaks,
            SpectrumPlotDrawContext {
                config: &self.settings.app,
                spectrum_sequence: self.state.spectrum_sequence,
                peak_memo: &mut self.view.peak_memo,
            },
        );
        if let Some(action) = plot_action {
            self.handle_spectrum_plot_action(action);
        }
    }

    fn draw_spectrogram_tab(&mut self, ui: &mut Ui, ctx: &Context) {
        let (controls_action, plot_action) = draw_spectrogram_view(
            ui,
            ctx,
            &mut self.spectrogram,
            &self.settings.app,
            self.state.connection,
        );
        if let Some(action) = controls_action {
            self.handle_spectrogram_action(action);
        }
        if let Some(action) = plot_action {
            self.handle_spectrum_plot_action(action);
        }
    }
}
