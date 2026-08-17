use egui::{Rect, pos2};

use super::peak_cursor::snapped_hover;
use crate::peak_overlay::spectrogram_energy_to_x;
use crate::spectrogram::model::{SpectrogramHeader, SpectrogramSeries};

#[test]
fn snap_within_radius() {
    let series = series_with_energies((0..5).map(|index| index as f64 * 100.0).collect());
    let source_cols = vec![0, 1, 2, 3, 4];
    let image_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 50.0));
    let peak_x = spectrogram_energy_to_x(image_rect, &series, &source_cols, 200.0).unwrap();
    let identifications = vec![identification_at(200.0)];
    let (snapped, focused) = snapped_hover(
        pos2(peak_x + 8.0, 25.0),
        image_rect,
        &series,
        &source_cols,
        &identifications,
    );
    assert_eq!(focused, Some(0));
    assert!((snapped.x - peak_x).abs() < 0.01);
}

#[test]
fn snap_beyond_radius_stays_free() {
    let series = series_with_energies((0..5).map(|index| index as f64 * 100.0).collect());
    let source_cols = vec![0, 1, 2, 3, 4];
    let image_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 50.0));
    let identifications = vec![identification_at(200.0)];
    let hover = pos2(10.0, 25.0);
    let (snapped, focused) =
        snapped_hover(hover, image_rect, &series, &source_cols, &identifications);
    assert!(focused.is_none());
    assert!((snapped.x - hover.x).abs() < 0.01);
}

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

fn identification_at(energy_kev: f64) -> radiacode_nuclides::PeakIdentification {
    radiacode_nuclides::PeakIdentification {
        peak: radiacode_nuclides::SpectrumPeak {
            energy_kev,
            counts: 100.0,
        },
        candidates: Vec::new(),
    }
}
