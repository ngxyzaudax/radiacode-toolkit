use egui::{RichText, Ui};

use crate::model::ConnectionState;
use crate::plot_style::draw_plot_style_toggle;
use crate::scale::YScale;
use crate::smooth::normalize_window;
use crate::theme::{SPACE_SM, SPACE_XS};
use crate::ui::{SPECTRUM_RESET, draw_reset_confirm};

pub struct ControlsProps<'a> {
    pub connection: ConnectionState,
    pub y_scale: &'a mut YScale,
    pub smooth_window: &'a mut usize,
    pub outline_only: &'a mut bool,
}

pub enum ControlsAction {
    Reset,
}

pub fn draw_spectrum_controls(ui: &mut Ui, props: ControlsProps<'_>) -> Option<ControlsAction> {
    if props.connection != ConnectionState::Connected {
        return None;
    }

    let mut action = None;
    let ctx = ui.ctx().clone();
    ui.horizontal(|ui| {
        ui.label(RichText::new("Spectrum").strong());
        if draw_reset_confirm(
            ui,
            &ctx,
            "spectrum_reset",
            true,
            "Reset spectrum accumulation",
            SPECTRUM_RESET,
        ) {
            action = Some(ControlsAction::Reset);
        }
    });
    ui.add_space(SPACE_XS);
    ui.label("Y scale");
    ui.horizontal(|ui| {
        ui.selectable_value(props.y_scale, YScale::Linear, "Linear");
        ui.selectable_value(props.y_scale, YScale::Logarithmic, "Log");
    });
    draw_plot_style_toggle(ui, props.outline_only);
    ui.add_space(SPACE_XS);
    ui.label("Smooth window (channels)");
    let mut slider = (*props.smooth_window).clamp(1, 16) as i32;
    if ui
        .add(egui::Slider::new(&mut slider, 1..=16).text("channels"))
        .changed()
    {
        *props.smooth_window = normalize_window(slider as usize);
    }
    ui.add_space(SPACE_SM);
    action
}
