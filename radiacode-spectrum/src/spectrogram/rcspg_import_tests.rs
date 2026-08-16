use super::import_recording;
use crate::spectrogram::model::{RowKind, SpectrogramHeader, SpectrogramSeries};
use crate::spectrogram::rcspg::export_recording;
use tempfile::tempdir;

#[test]
fn rcspg_round_trip() {
    let header = SpectrogramHeader {
        created_at: "2024-01-01 00:00:00".into(),
        a0: 0.0,
        a1: 1.0,
        a2: 0.0,
        channel_count: 3,
        interval_secs: 60.0,
        device_serial: Some("RC-TEST".into()),
        energies_kev: vec![0.0, 1.0, 2.0],
    };
    let mut series = SpectrogramSeries::new(header, vec![0.0, 1.0, 2.0]);
    series.push_row(vec![1, 2, 3], 60.0, RowKind::Normal, 1000);
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.rcspg");
    export_recording(&path, &series, "test", "note").unwrap();
    let loaded = import_recording(&path).unwrap();
    assert_eq!(loaded.rows.len(), 1);
    assert_eq!(loaded.rows[0].counts, vec![1, 2, 3]);
}
