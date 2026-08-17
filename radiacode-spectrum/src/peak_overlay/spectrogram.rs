use egui::{Align2, FontId, Pos2, Rect, Stroke};
use radiacode_nuclides::PeakIdentification;

use crate::peak_overlay::label::peak_label;
use crate::peak_overlay::markers::PEAK_LABEL_FONT_SIZE;
use crate::peak_overlay::markers::PEAK_LINE;
use crate::spectrogram::model::SpectrogramSeries;
use crate::spectrogram::preview::energy_to_x;
use crate::theme::ACCENT;

pub fn draw_spectrogram_peaks(
    painter: &egui::Painter,
    image_rect: Rect,
    series: &SpectrogramSeries,
    source_cols: &[usize],
    identifications: &[PeakIdentification],
    focused: Option<usize>,
) {
    for (index, identification) in identifications.iter().enumerate() {
        let Some(x) = spectrogram_energy_to_x(
            image_rect,
            series,
            source_cols,
            identification.peak.energy_kev,
        ) else {
            continue;
        };
        let is_focused = focused == Some(index);
        let stroke = Stroke::new(if is_focused { 2.5 } else { 1.0 }, PEAK_LINE);
        painter.line_segment(
            [
                Pos2::new(x, image_rect.top()),
                Pos2::new(x, image_rect.bottom()),
            ],
            stroke,
        );
        let font_size = if is_focused {
            PEAK_LABEL_FONT_SIZE + 2.0
        } else {
            PEAK_LABEL_FONT_SIZE
        };
        painter.text(
            Pos2::new(x + 2.0, image_rect.top() + 2.0),
            Align2::LEFT_TOP,
            peak_label(identification),
            FontId::new(font_size, egui::FontFamily::Proportional),
            ACCENT,
        );
    }
}

pub fn spectrogram_energy_to_x(
    image_rect: Rect,
    series: &SpectrogramSeries,
    source_cols: &[usize],
    energy_kev: f64,
) -> Option<f32> {
    energy_to_x(image_rect, series, source_cols, energy_kev)
}

#[cfg(test)]
mod tests {
    use egui::{Rect, pos2};

    use super::spectrogram_energy_to_x;
    use crate::spectrogram::model::{SpectrogramHeader, SpectrogramSeries};

    fn series_with_energies(energies_kev: Vec<f64>) -> SpectrogramSeries {
        let channel_count = energies_kev.len() as u32;
        SpectrogramSeries::new(
            SpectrogramHeader {
                created_at: "t".into(),
                a0: 0.0,
                a1: 1.0,
                a2: 0.0,
                channel_count,
                interval_secs: 1.0,
                device_serial: None,
                energies_kev: energies_kev.clone(),
            },
            energies_kev,
        )
    }

    #[test]
    fn peak_x_matches_column_energy_mapping() {
        let series = series_with_energies((0..5).map(|index| index as f64 * 100.0).collect());
        let source_cols = vec![0, 1, 2, 3, 4];
        let image_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 50.0));
        let x = spectrogram_energy_to_x(image_rect, &series, &source_cols, 200.0).unwrap();
        assert!((x - 50.0).abs() < 0.01);
    }

    #[test]
    fn peak_outside_visible_columns_is_omitted() {
        let series = series_with_energies((0..5).map(|index| index as f64 * 100.0).collect());
        let source_cols = vec![2, 3, 4];
        let image_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 50.0));
        assert!(spectrogram_energy_to_x(image_rect, &series, &source_cols, 50.0).is_none());
    }
}
