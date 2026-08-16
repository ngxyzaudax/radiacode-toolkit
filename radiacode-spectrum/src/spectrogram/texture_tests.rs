use crate::spectrogram::color_scheme::ColorScheme;
use crate::spectrogram::gap::display_count;
use crate::spectrogram::model::{RowKind, SpectrogramHeader, SpectrogramRow, SpectrogramSeries};
use crate::spectrogram::settings::SpectrogramSettings;
use crate::spectrogram::texture::{SpectrogramTexture, native_rows, source_columns};
use crate::spectrogram::zscale::compute_series_z_range;

#[test]
fn native_row_maps_one_to_one() {
    let row = SpectrogramRow {
        elapsed_secs: 0.0,
        interval_secs: 5.0,
        kind: RowKind::Normal,
        counts: vec![1, 50, 2, 3],
    };
    let cols = vec![0, 1, 2, 3];
    assert_eq!(native_rows(&[row], &cols, 5.0)[0], vec![1, 50, 2, 3]);
}

#[test]
fn gap_row_brightness_is_rate_normalized() {
    let raw = 1000;
    let scaled = display_count(
        raw,
        RowKind::GapRecovery {
            offline_secs: 50.0,
            raw_total: 1000,
        },
        5.0,
        50.0,
    );
    let row = SpectrogramRow {
        elapsed_secs: 0.0,
        interval_secs: 50.0,
        kind: RowKind::GapRecovery {
            offline_secs: 50.0,
            raw_total: 1000,
        },
        counts: vec![raw, 0, 0, 0],
    };
    let cols = vec![0, 1, 2, 3];
    assert_eq!(native_rows(&[row], &cols, 5.0)[0][0], scaled);
    assert!(scaled < raw);
}

#[test]
fn source_columns_respects_window() {
    let header = SpectrogramHeader {
        created_at: "t".into(),
        a0: 0.0,
        a1: 1.0,
        a2: 0.0,
        channel_count: 4,
        interval_secs: 5.0,
        device_serial: None,
        energies_kev: vec![10.0, 20.0, 30.0, 40.0],
    };
    let series = SpectrogramSeries::new(header, vec![10.0, 20.0, 30.0, 40.0]);
    assert_eq!(source_columns(&series, 0.0, 3000.0, 1, 2), vec![1, 2]);
}

#[test]
fn rebuild_lights_nonzero_bins() {
    let header = SpectrogramHeader {
        created_at: "t".into(),
        a0: 0.0,
        a1: 1.0,
        a2: 0.0,
        channel_count: 4,
        interval_secs: 5.0,
        device_serial: None,
        energies_kev: vec![10.0, 20.0, 30.0, 40.0],
    };
    let mut series = SpectrogramSeries::new(header, vec![10.0, 20.0, 30.0, 40.0]);
    series.push_row(vec![0, 50, 0, 10], 5.0, RowKind::Normal, 1000);
    let cols = vec![0, 1, 2, 3];
    let mut texture = SpectrogramTexture::new(1, 1);
    let z_range = compute_series_z_range(&series, &SpectrogramSettings::default());
    texture.rebuild(
        &series,
        &series.rows,
        &cols,
        8,
        &SpectrogramSettings {
            palette: ColorScheme::Viridis,
            ..SpectrogramSettings::default()
        },
        &z_range,
    );
    let lit = texture
        .image
        .pixels
        .iter()
        .filter(|pixel| **pixel != egui::Color32::from_rgb(8, 10, 16))
        .count();
    assert!(lit > 0);
}
