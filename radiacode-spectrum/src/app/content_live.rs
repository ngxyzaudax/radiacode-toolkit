use egui::{Context, Ui};

use super::SpectrumApp;
use crate::monitor::{MonitorViewProps, draw_monitor_view};
use crate::plot_style::histogram_style;
use crate::spectrogram::ui_view::draw_spectrogram_view;
use crate::ui_plot::{SpectrumPlotDrawContext, draw_spectrum_plot};
use crate::ui_toolbar::{SpectrumToolbarProps, draw_spectrum_toolbar};

impl SpectrumApp {
    pub(super) fn draw_monitor_tab(&mut self, ui: &mut Ui) {
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

    pub(super) fn draw_spectrum_tab(&mut self, ui: &mut Ui) {
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

    pub(super) fn draw_spectrogram_tab(&mut self, ui: &mut Ui, ctx: &Context) {
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
