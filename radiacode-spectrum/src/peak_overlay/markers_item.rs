use egui::{Align2, RichText};
use egui_plot::{MarkerShape, PlotPoint, PlotUi, Points, Text, VLine};
use radiacode_nuclides::PeakIdentification;

use crate::peak_overlay::label::peak_label;
use crate::peak_overlay::markers::{PEAK_LABEL_FONT_SIZE, PEAK_LINE};
use crate::theme::ACCENT;

#[derive(Copy, Clone)]
pub enum MarkerFocus {
    Normal,
    Focused,
}

impl MarkerFocus {
    pub fn from_index(index: usize, focused: Option<usize>) -> Self {
        if focused == Some(index) {
            Self::Focused
        } else {
            Self::Normal
        }
    }

    fn line_width(self) -> f32 {
        match self {
            Self::Normal => 1.0,
            Self::Focused => 2.5,
        }
    }

    fn marker_radius(self) -> f32 {
        match self {
            Self::Normal => 4.0,
            Self::Focused => 6.5,
        }
    }

    fn label_size(self) -> f32 {
        match self {
            Self::Normal => PEAK_LABEL_FONT_SIZE,
            Self::Focused => PEAK_LABEL_FONT_SIZE + 2.0,
        }
    }
}

pub fn draw_peak_marker(
    plot_ui: &mut PlotUi,
    index: usize,
    identification: &PeakIdentification,
    focus: MarkerFocus,
    curve_y: impl Fn(f64) -> f64,
) {
    let energy = identification.peak.energy_kev;
    let height = curve_y(energy);
    let name = format!("peak_{index}");
    plot_ui.vline(
        VLine::new(format!("{name}_line"), energy)
            .color(PEAK_LINE)
            .width(focus.line_width()),
    );
    let label_y = peak_label_y(plot_ui, height);
    plot_ui.text(
        Text::new(
            format!("{name}_text"),
            PlotPoint::new(energy, label_y),
            RichText::new(peak_label(identification)).size(focus.label_size()),
        )
        .color(ACCENT)
        .anchor(Align2::CENTER_BOTTOM),
    );
    plot_ui.points(
        Points::new(name, vec![[energy, height]])
            .radius(focus.marker_radius())
            .shape(MarkerShape::Diamond)
            .color(PEAK_LINE),
    );
}

fn peak_label_y(plot_ui: &PlotUi, peak_height: f64) -> f64 {
    let bounds = plot_ui.plot_bounds();
    let span = (bounds.max()[1] - bounds.min()[1]).max(0.01);
    peak_height + span * 0.06
}
