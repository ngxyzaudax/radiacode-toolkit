use super::{RowKind, SpectrogramHeader, SpectrogramSeries};

fn sample_header() -> SpectrogramHeader {
    SpectrogramHeader {
        created_at: "t".into(),
        a0: 0.0,
        a1: 1.0,
        a2: 0.0,
        channel_count: 2,
        interval_secs: 5.0,
        device_serial: None,
        energies_kev: vec![100.0, 200.0],
    }
}

#[test]
fn duration_secs_sums_variable_intervals() {
    let mut series = SpectrogramSeries::new(sample_header(), vec![100.0, 200.0]);
    series.push_row(vec![1, 2], 5.0, RowKind::Normal, 100);
    series.push_row(
        vec![3, 4],
        45.0,
        RowKind::GapRecovery {
            offline_secs: 45.0,
            raw_total: 7,
        },
        100,
    );
    series.push_row(vec![5, 6], 5.0, RowKind::Normal, 100);
    assert!((series.duration_secs() - 55.0).abs() < 0.001);
}

#[test]
fn age_secs_before_uses_row_intervals() {
    let mut series = SpectrogramSeries::new(sample_header(), vec![100.0, 200.0]);
    series.push_row(vec![1, 2], 5.0, RowKind::Normal, 100);
    series.push_row(
        vec![3, 4],
        45.0,
        RowKind::GapRecovery {
            offline_secs: 45.0,
            raw_total: 7,
        },
        100,
    );
    series.push_row(vec![5, 6], 5.0, RowKind::Normal, 100);
    assert!((series.age_secs_before(0) - 50.0).abs() < 0.001);
}
