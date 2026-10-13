use crate::app_config::AppConfig;
use crate::compare::peak_overlay_build::build_peak_overlay;
use crate::compare::plot_series_build::{owned_series, series_peak, shared_log_floor};
use crate::compare::spectrum::CollapsedSpectrum;
use crate::compare::state::ComparedSample;
use crate::compare::subtract::subtract;
use crate::compare::ui_plot_bars::PlotSeries;
use crate::peaks::PeakMemo;
use crate::peaks::detect_fixtures::load_fixture;
use crate::scale::YScale;

fn collapsed(name: &str, live_time_secs: f64) -> CollapsedSpectrum {
    let fixture = load_fixture(name);
    CollapsedSpectrum {
        name: name.into(),
        path: format!("/tmp/{name}.rcwf").into(),
        a0: 0.0,
        a1: 1.0,
        a2: 0.0,
        channel_count: fixture.counts.len() as u32,
        total_counts: fixture.counts.iter().sum(),
        counts: fixture.counts,
        energies_kev: fixture.energies_kev,
        live_time_secs,
        gap_count: 0,
        gap_offline_secs: 0.0,
        device_serial: None,
    }
}

fn sample_with_background() -> (ComparedSample, CollapsedSpectrum) {
    let spectrum = collapsed("thoriated_rods", 600.0);
    let background = collapsed("background_indoor_outdoor", 600.0);
    let subtraction = subtract(&spectrum, &background).ok();
    (
        ComparedSample {
            spectrum,
            subtraction,
        },
        background,
    )
}

fn overlay(
    samples: &[ComparedSample],
    background: Option<&CollapsedSpectrum>,
    subtract_background: bool,
    show_peaks: bool,
) -> Option<crate::compare::peak_overlay_build::BuiltPeakOverlay> {
    let config = AppConfig::default();
    let mut memo = PeakMemo::new();
    build_peak_overlay(
        samples,
        background,
        subtract_background,
        show_peaks,
        3,
        &config,
        &mut memo,
    )
}

#[test]
fn hidden_peaks_produce_no_overlay() {
    let (sample, _) = sample_with_background();
    assert!(overlay(&[sample], None, false, false).is_none());
}

#[test]
fn sample_overlay_detects_peaks_and_matches_sample_axis() {
    let (sample, _) = sample_with_background();
    let built = overlay(std::slice::from_ref(&sample), None, false, true).expect("overlay");
    assert!(built.peak_count > 0);
    assert_eq!(built.energies, sample.spectrum.energies_kev);
    assert_eq!(built.display_values.len(), sample.spectrum.counts.len());
}

#[test]
fn net_overlay_requires_subtraction_and_background() {
    let (sample, background) = sample_with_background();
    assert!(overlay(std::slice::from_ref(&sample), None, true, true).is_none());
    let without_subtraction = ComparedSample {
        spectrum: sample.spectrum.clone(),
        subtraction: None,
    };
    assert!(overlay(&[without_subtraction], Some(&background), true, true).is_none());
    let built = overlay(&[sample], Some(&background), true, true).expect("overlay");
    assert!(built.display_values.iter().all(|value| *value >= 0.0));
}

#[test]
fn background_only_overlay_uses_background_axis() {
    let (_, background) = sample_with_background();
    assert!(overlay(&[], None, false, true).is_none());
    let built = overlay(&[], Some(&background), false, true).expect("overlay");
    assert_eq!(built.energies, background.energies_kev);
}

#[test]
fn overlay_plot_view_borrows_built_data() {
    let (sample, _) = sample_with_background();
    let built = overlay(&[sample], None, false, true).expect("overlay");
    let view = built.plot_overlay(YScale::Logarithmic, 0.01);
    assert_eq!(view.identifications.len(), built.identifications.len());
    assert_eq!(view.y_scale, YScale::Logarithmic);
}

#[test]
fn owned_series_builds_one_bar_per_channel() {
    let energies = [10.0, 20.0, 30.0];
    let values = [1.0, 2.0, 0.0];
    let (id, bars, _) = owned_series(
        "s",
        &energies,
        5.0,
        &values,
        egui::Color32::RED,
        YScale::Linear,
        0.0,
    );
    assert_eq!(id, "s");
    assert_eq!(bars.len(), 3);
    assert_eq!(bars[1].value, 2.0);
    let series = [PlotSeries {
        id: "s",
        bars: &bars,
        color: egui::Color32::RED,
    }];
    assert_eq!(series_peak(&series), 2.0);
}

#[test]
fn shared_log_floor_spans_all_value_sets() {
    let floor = shared_log_floor(&[&[0.0, 10.0], &[0.5, 20.0]]);
    assert!(floor > 0.0);
    assert!(floor <= 0.5);
}
