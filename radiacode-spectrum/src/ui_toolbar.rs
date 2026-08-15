use egui::Ui;

use crate::layout::draw_toolbar;
use crate::model::ConnectionState;
use crate::plot_style::draw_plot_style_toggle;
use crate::scale::YScale;
use crate::smooth::normalize_window;
use crate::theme::SPACE_XS;
use crate::ui::{SPECTRUM_RESET, draw_reset_confirm};

pub struct SpectrumToolbarProps<'a> {
    pub connection: ConnectionState,
    pub y_scale: &'a mut YScale,
    pub smooth_window: &'a mut usize,
    pub outline_only: &'a mut bool,
    pub show_peaks: &'a mut bool,
    pub identify_isotopes: &'a mut bool,
}

pub enum SpectrumToolbarAction {
    Reset,
}

pub fn draw_spectrum_toolbar(
    ui: &mut Ui,
    props: SpectrumToolbarProps<'_>,
) -> Option<SpectrumToolbarAction> {
    if props.connection != ConnectionState::Connected {
        return None;
    }
    let mut action = None;
    let ctx = ui.ctx().clone();
    draw_toolbar(ui, |ui| {
        ui.selectable_value(props.y_scale, YScale::Linear, "Linear");
        ui.selectable_value(props.y_scale, YScale::Logarithmic, "Log");
        draw_plot_style_toggle(ui, props.outline_only);
        ui.checkbox(props.show_peaks, "Peaks");
        ui.add_enabled_ui(*props.show_peaks, |ui| {
            ui.checkbox(props.identify_isotopes, "Identify");
        });
        ui.label("Smoothing");
        let mut slider = (*props.smooth_window).clamp(1, 16) as i32;
        if ui
            .add(egui::Slider::new(&mut slider, 1..=16).text("channels"))
            .changed()
        {
            *props.smooth_window = normalize_window(slider as usize);
        }
        if draw_reset_confirm(
            ui,
            &ctx,
            "spectrum_reset",
            true,
            "Reset spectrum accumulation",
            SPECTRUM_RESET,
        ) {
            action = Some(SpectrumToolbarAction::Reset);
        }
    });
    ui.add_space(SPACE_XS);
    action
}
