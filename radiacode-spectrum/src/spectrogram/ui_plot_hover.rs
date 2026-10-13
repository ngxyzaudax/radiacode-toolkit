use egui::{Pos2, Rect};

use crate::identify::PeakAnalysis;
use crate::peak_snap::peak_hover_text;
use crate::spectrogram::model::SpectrogramSeries;
use crate::spectrogram::peak_cursor::snapped_hover;

pub fn snapped_peak_hover(
    hover: Pos2,
    image_rect: Rect,
    preview_area: Rect,
    series: &SpectrogramSeries,
    source_cols: &[usize],
    peak_analysis: Option<&PeakAnalysis>,
) -> (Pos2, Option<usize>) {
    let in_plot = image_rect.contains(hover) || preview_area.contains(hover);
    match peak_analysis {
        Some(analysis) if in_plot => snapped_hover(
            hover,
            image_rect,
            series,
            source_cols,
            &analysis.identifications,
        ),
        _ => (hover, None),
    }
}

pub fn spectrogram_tooltip(
    details: String,
    peak_analysis: Option<&PeakAnalysis>,
    focused: Option<usize>,
) -> String {
    let identification = focused
        .zip(peak_analysis)
        .and_then(|(index, analysis)| analysis.identifications.get(index));
    let Some(identification) = identification else {
        return details;
    };
    let peak_text = peak_hover_text(identification);
    if details.is_empty() {
        peak_text
    } else {
        format!("{peak_text}\n\n{details}")
    }
}

#[cfg(test)]
mod tests {
    use super::spectrogram_tooltip;

    #[test]
    fn tooltip_without_focus_returns_details_unchanged() {
        assert_eq!(
            spectrogram_tooltip("row 1".into(), None, None),
            "row 1".to_string()
        );
        assert_eq!(
            spectrogram_tooltip("row 1".into(), None, Some(0)),
            "row 1".to_string()
        );
    }
}
