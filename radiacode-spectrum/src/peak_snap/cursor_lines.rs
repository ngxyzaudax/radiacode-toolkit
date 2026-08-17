use egui_plot::{HLine, LineStyle, PlotUi, VLine};

use crate::peak_overlay::PEAK_LINE;
use crate::theme::MUTED;

pub fn draw_plot_cursor(plot_ui: &mut PlotUi, energy_kev: f64, focused: bool) {
    if !plot_ui.response().hovered() {
        return;
    }
    let Some(pointer) = plot_ui.pointer_coordinate() else {
        return;
    };
    let vline = if focused {
        VLine::new("peak_cursor_v", energy_kev)
            .color(PEAK_LINE)
            .width(2.5)
            .style(LineStyle::Solid)
    } else {
        VLine::new("peak_cursor_v", energy_kev)
            .color(MUTED)
            .width(1.0)
            .style(LineStyle::Dashed { length: 4.0 })
    };
    plot_ui.vline(vline.allow_hover(false));
    plot_ui.hline(
        HLine::new("peak_cursor_h", pointer.y)
            .color(MUTED)
            .width(1.0)
            .style(LineStyle::Dashed { length: 4.0 })
            .allow_hover(false),
    );
}
