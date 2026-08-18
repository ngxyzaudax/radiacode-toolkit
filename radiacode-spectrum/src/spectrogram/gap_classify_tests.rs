use std::time::{Duration, Instant};

use crate::model::SpectrumView;
use crate::spectrogram::baseline::IngestBaseline;
use crate::spectrogram::model::RowKind;

use super::{classify_row, display_count};

#[test]
fn gap_row_when_device_duration_jumps() {
    let baseline = IngestBaseline {
        counts: vec![10; 512],
        device_duration_secs: 5.0,
        ingested_at: Instant::now(),
    };
    let cumulative = vec![5000; 512];
    let classified = classify_row(
        &spectrum(5000, 50),
        &baseline,
        &cumulative,
        5.0,
        &[20 * 512],
    );
    assert!(matches!(classified.kind, RowKind::GapRecovery { .. }));
    assert!((classified.interval_secs - 45.0).abs() < 0.1);
}

#[test]
fn live_spike_when_connected_and_elevated() {
    let baseline = IngestBaseline {
        counts: vec![100; 512],
        device_duration_secs: 10.0,
        ingested_at: Instant::now(),
    };
    let cumulative = vec![5000; 512];
    let recent: Vec<u64> = (0..5).map(|_| 22 * 512).collect();
    let classified = classify_row(&spectrum(5000, 15), &baseline, &cumulative, 5.0, &recent);
    assert!(matches!(classified.kind, RowKind::LiveSpike { .. }));
}

#[test]
fn device_timeline_regressed_on_duration_drop() {
    assert!(super::device_timeline_regressed(
        -10.0, &[100; 4], &[120; 4]
    ));
}

#[test]
fn device_timeline_regressed_on_count_drop() {
    assert!(super::device_timeline_regressed(5.0, &[1000; 4], &[10; 4]));
}

#[test]
fn row_interval_ready_requires_capture_window() {
    assert!(super::row_interval_ready(8.5, 10.0));
    assert!(!super::row_interval_ready(4.0, 10.0));
}

#[test]
fn normal_row_scales_partial_interval() {
    let scaled = display_count(100, RowKind::Normal, 10.0, 5.0);
    assert_eq!(scaled, 200);
}

#[test]
fn gap_display_scales_brightness_down() {
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
    assert!(scaled < raw);
    assert_eq!(scaled, 100);
}

#[test]
fn normal_row_preserves_sub_threshold_gap_interval() {
    let baseline = IngestBaseline {
        counts: vec![10; 512],
        device_duration_secs: 0.0,
        ingested_at: Instant::now(),
    };
    let cumulative = vec![100; 512];
    let classified = classify_row(
        &spectrum(100, 9),
        &baseline,
        &cumulative,
        1.0,
        &[20 * 512],
    );
    assert!(matches!(classified.kind, RowKind::Normal));
    assert!((classified.interval_secs - 9.0).abs() < 0.1);
}

fn spectrum(total: u32, duration_secs: u64) -> SpectrumView {
    SpectrumView {
        duration: Duration::from_secs(duration_secs),
        a0: 0.0,
        a1: 1.0,
        a2: 0.0,
        counts: vec![total; 512],
        total_counts: total as u64 * 512,
    }
}
