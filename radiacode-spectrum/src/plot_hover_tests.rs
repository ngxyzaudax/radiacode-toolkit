use crate::plot_hover::{
    SpectrogramHoverInfo, format_counts_value, format_rate_value, format_relative_intensity_value,
    format_spectrum_hover, spectrogram_hover_text,
};
use crate::scale::YScale;
use crate::spectrogram::model::RowKind;

#[test]
fn spectrum_hover_uses_shared_header() {
    let label = format_spectrum_hover(662.0, "Counts", "1234.5");
    assert_eq!(label, "662.0 keV\nCounts: 1234.5");
}

#[test]
fn logarithmic_values_are_converted_back_to_linear() {
    assert_eq!(format_counts_value(2.0, YScale::Logarithmic), "100.0");
    assert_eq!(format_counts_value(2.0, YScale::Linear), "2.0");
    assert_eq!(
        format_rate_value(1.0, YScale::Logarithmic, 0.5),
        "5.000 cps"
    );
    assert_eq!(format_rate_value(1.25, YScale::Linear, 0.5), "1.25 cps");
    assert_eq!(
        format_relative_intensity_value(-1.0, YScale::Logarithmic),
        "0.1000"
    );
    assert_eq!(
        format_relative_intensity_value(0.5, YScale::Linear),
        "0.500"
    );
}

fn info(kind: RowKind) -> SpectrogramHoverInfo {
    SpectrogramHoverInfo {
        energy_kev: 100.0,
        counts: 7,
        channel: 3,
        absolute_row: 2,
        total_rows: 10,
        age_secs: 30.0,
        interval_secs: 5.0,
        row_total: 42,
        kind,
    }
}

#[test]
fn spectrogram_hover_describes_row_kind() {
    let normal = spectrogram_hover_text(&info(RowKind::Normal));
    assert!(normal.starts_with("100.0 keV\nCounts: 7\n\nchannel 3\nrow 2 / 10\n30 s ago"));
    assert!(normal.ends_with("kind: normal"));
    let gap = spectrogram_hover_text(&info(RowKind::GapRecovery {
        offline_secs: 20.0,
        raw_total: 400,
    }));
    assert!(gap.ends_with("kind: gap recovery\noffline 20 s\nraw total 400\n20.0 counts/s"));
    let spike = spectrogram_hover_text(&info(RowKind::LiveSpike { rate_factor: 3.5 }));
    assert!(spike.ends_with("kind: live spike\n3.5× recent median\nrow total 42"));
}
