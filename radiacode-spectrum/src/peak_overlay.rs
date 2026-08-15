use egui::{Align2, Color32, Pos2, Rect, RichText, Stroke};
use egui_plot::{MarkerShape, PlotPoints, PlotUi, Points, VLine};
use radiacode_nuclides::{PeakIdentification, best_match};

use crate::peak_detect::SpectrumPeak;
use crate::scale::{YScale, display_rate, display_value};
use crate::theme::{ACCENT, MUTED};

const ISOTOPE_STROKE: Color32 = Color32::from_rgb(255, 220, 80);
const PEAK_LABEL_FONT_SIZE: f32 = 14.0;
const PEAK_LABEL_Y_OFFSET_FRAC: f64 = 0.06;

pub enum SpectrumPlotAction {
    OpenCatalogue(radiacode_nuclides::NuclideId),
}

pub fn draw_spectrum_peaks(plot_ui: &mut PlotUi, peaks: &[SpectrumPeak], y_scale: YScale) {
    for (index, peak) in peaks.iter().enumerate() {
        let height = display_value(peak.counts, y_scale);
        let name = format!("peak_{index}");
        plot_ui.vline(
            VLine::new(format!("{name}_line"), peak.energy_kev)
                .color(Color32::from_rgba_unmultiplied(120, 220, 180, 180))
                .width(1.0),
        );
        plot_ui.points(
            Points::new(name, PlotPoints::new(vec![[peak.energy_kev, height]]))
                .radius(4.0)
                .shape(MarkerShape::Diamond)
                .color(ACCENT),
        );
    }
}

pub fn draw_identified_peaks(
    plot_ui: &mut PlotUi,
    identifications: &[PeakIdentification],
    y_scale: YScale,
) {
    for (index, identification) in identifications.iter().enumerate() {
        let peak = identification.peak;
        let height = display_value(peak.counts, y_scale);
        let name = format!("ident_{index}");
        let line_name = format!("{name}_line");
        plot_ui.vline(
            VLine::new(line_name, peak.energy_kev)
                .color(ISOTOPE_STROKE)
                .width(1.0),
        );
        let label = best_match(identification)
            .map(|candidate| candidate.display_name.clone())
            .unwrap_or_else(|| format!("{:.0} keV", peak.energy_kev));
        let label_y = peak_label_y(plot_ui, height);
        plot_ui.text(
            egui_plot::Text::new(
                format!("{name}_text"),
                egui_plot::PlotPoint::new(peak.energy_kev, label_y),
                isotope_label_text(label),
            )
            .color(ACCENT)
            .anchor(Align2::CENTER_BOTTOM),
        );
        plot_ui.points(
            Points::new(name, PlotPoints::new(vec![[peak.energy_kev, height]]))
                .radius(4.0)
                .shape(MarkerShape::Diamond)
                .color(ISOTOPE_STROKE),
        );
    }
}

pub fn draw_identified_rate_peaks(
    plot_ui: &mut PlotUi,
    identifications: &[PeakIdentification],
    y_scale: YScale,
    log_floor: f64,
) {
    for (index, identification) in identifications.iter().enumerate() {
        let peak = identification.peak;
        let height = display_rate(peak.counts, y_scale, log_floor);
        let name = format!("ident_{index}");
        let line_name = format!("{name}_line");
        plot_ui.vline(
            VLine::new(line_name, peak.energy_kev)
                .color(ISOTOPE_STROKE)
                .width(1.0),
        );
        let label = best_match(identification)
            .map(|candidate| candidate.display_name.clone())
            .unwrap_or_else(|| format!("{:.0} keV", peak.energy_kev));
        let label_y = peak_label_y(plot_ui, height);
        plot_ui.text(
            egui_plot::Text::new(
                format!("{name}_text"),
                egui_plot::PlotPoint::new(peak.energy_kev, label_y),
                isotope_label_text(label),
            )
            .color(ACCENT)
            .anchor(Align2::CENTER_BOTTOM),
        );
        plot_ui.points(
            Points::new(name, PlotPoints::new(vec![[peak.energy_kev, height]]))
                .radius(4.0)
                .shape(MarkerShape::Diamond)
                .color(ISOTOPE_STROKE),
        );
    }
}

pub fn draw_rate_peaks(
    plot_ui: &mut PlotUi,
    peaks: &[SpectrumPeak],
    y_scale: YScale,
    log_floor: f64,
) {
    for (index, peak) in peaks.iter().enumerate() {
        let height = display_rate(peak.counts, y_scale, log_floor);
        let name = format!("peak_{index}");
        plot_ui.vline(
            VLine::new(format!("{name}_line"), peak.energy_kev)
                .color(Color32::from_rgba_unmultiplied(120, 220, 180, 180))
                .width(1.0),
        );
        plot_ui.points(
            Points::new(name, PlotPoints::new(vec![[peak.energy_kev, height]]))
                .radius(4.0)
                .shape(MarkerShape::Diamond)
                .color(ACCENT),
        );
    }
}

pub fn draw_spectrogram_peaks(
    painter: &egui::Painter,
    image_rect: Rect,
    energy_min: f64,
    energy_max: f64,
    peaks: &[SpectrumPeak],
) {
    let span = (energy_max - energy_min).max(1.0);
    let stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(120, 220, 180, 200));
    for peak in peaks {
        if peak.energy_kev < energy_min || peak.energy_kev > energy_max {
            continue;
        }
        let t = ((peak.energy_kev - energy_min) / span) as f32;
        let x = egui::lerp(image_rect.left()..=image_rect.right(), t);
        painter.line_segment(
            [
                Pos2::new(x, image_rect.top()),
                Pos2::new(x, image_rect.bottom()),
            ],
            stroke,
        );
        painter.text(
            Pos2::new(x + 2.0, image_rect.top() + 2.0),
            egui::Align2::LEFT_TOP,
            format!("{:.0}", peak.energy_kev),
            egui::FontId::new(10.0, egui::FontFamily::Proportional),
            ACCENT,
        );
    }
}

pub fn draw_identified_spectrogram_lines(
    painter: &egui::Painter,
    image_rect: Rect,
    energy_min: f64,
    energy_max: f64,
    identifications: &[PeakIdentification],
) {
    let span = (energy_max - energy_min).max(1.0);
    let stroke = Stroke::new(1.0, ISOTOPE_STROKE);
    for identification in identifications {
        let peak = identification.peak;
        if peak.energy_kev < energy_min || peak.energy_kev > energy_max {
            continue;
        }
        let t = ((peak.energy_kev - energy_min) / span) as f32;
        let x = egui::lerp(image_rect.left()..=image_rect.right(), t);
        painter.line_segment(
            [
                Pos2::new(x, image_rect.top()),
                Pos2::new(x, image_rect.bottom()),
            ],
            stroke,
        );
        let label = best_match(identification)
            .map(|candidate| candidate.display_name.clone())
            .unwrap_or_else(|| format!("{:.0}", peak.energy_kev));
        painter.text(
            Pos2::new(x + 2.0, image_rect.top() + 2.0),
            Align2::LEFT_TOP,
            label,
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
            MUTED,
        );
    }
}

fn peak_label_y(plot_ui: &PlotUi, peak_height: f64) -> f64 {
    let bounds = plot_ui.plot_bounds();
    let span = (bounds.max()[1] - bounds.min()[1]).max(0.01);
    peak_height + span * PEAK_LABEL_Y_OFFSET_FRAC
}

fn isotope_label_text(label: String) -> RichText {
    RichText::new(label).size(PEAK_LABEL_FONT_SIZE)
}

pub fn draw_identification_chips(
    ui: &mut egui::Ui,
    identifications: &[PeakIdentification],
) -> Option<SpectrumPlotAction> {
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        for identification in identifications {
            let Some(candidate) = best_match(identification) else {
                continue;
            };
            let label = format!(
                "{} {:.0} keV",
                candidate.display_name, identification.peak.energy_kev
            );
            if ui.button(label).clicked() {
                action = Some(SpectrumPlotAction::OpenCatalogue(candidate.nuclide_id));
            }
        }
    });
    action
}
