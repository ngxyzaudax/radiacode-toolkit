use crate::spectrogram::model::{RowKind, SpectrogramRow};
use tempfile::tempdir;

use super::recording_load::load_recording;
use super::recording_writer::{RecordingWriter, open_recording_append};
use super::storage_dir::load_recording_index;
use super::storage_format::{VERSION_V1, header_now};

fn gap_row(counts: Vec<u32>) -> SpectrogramRow {
    let raw_total = counts.iter().map(|&value| value as u64).sum();
    SpectrogramRow {
        elapsed_secs: 0.0,
        interval_secs: 45.0,
        kind: RowKind::GapRecovery {
            offline_secs: 45.0,
            raw_total,
        },
        counts,
    }
}

#[test]
fn round_trip_recording_v2() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.rcwf");
    let header = header_now(0.0, 1.0, 0.0, 2, 5.0, None, vec![100.0, 200.0]);
    let mut writer = RecordingWriter::create(path.clone(), &header).unwrap();
    writer
        .append_row(&SpectrogramRow {
            elapsed_secs: 0.0,
            interval_secs: 5.0,
            kind: RowKind::Normal,
            counts: vec![1, 2],
        })
        .unwrap();
    writer.append_row(&gap_row(vec![100, 200])).unwrap();
    writer.finalize().unwrap();
    let loaded = load_recording(&path).unwrap();
    assert_eq!(loaded.rows.len(), 2);
    assert_eq!(loaded.rows[0].counts, vec![1, 2]);
    assert!(matches!(loaded.rows[1].kind, RowKind::GapRecovery { .. }));
    assert!((loaded.rows[1].interval_secs - 45.0).abs() < 0.001);
    assert!((loaded.duration_secs() - 50.0).abs() < 0.001);
}

#[test]
fn recording_index_reads_header_without_row_payload() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("index.rcwf");
    let header = header_now(0.0, 1.0, 0.0, 2, 5.0, None, vec![100.0, 200.0]);
    let mut writer = RecordingWriter::create(path.clone(), &header).unwrap();
    writer
        .append_row(&SpectrogramRow {
            elapsed_secs: 0.0,
            interval_secs: 5.0,
            kind: RowKind::Normal,
            counts: vec![1, 2],
        })
        .unwrap();
    writer.finalize().unwrap();
    let index = load_recording_index(&path).unwrap();
    assert_eq!(index.row_count, 1);
    assert_eq!(index.header.channel_count, 2);
}

#[test]
fn v1_loader_treats_rows_as_normal() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("legacy.rcwf");
    let header = header_now(0.0, 1.0, 0.0, 2, 5.0, None, vec![100.0, 200.0]);
    let mut file = std::fs::File::create(&path).unwrap();
    use std::io::Write;
    file.write_all(b"RCWF").unwrap();
    file.write_all(&VERSION_V1.to_le_bytes()).unwrap();
    let header_json = serde_json::to_vec(&header).unwrap();
    file.write_all(&(header_json.len() as u32).to_le_bytes())
        .unwrap();
    file.write_all(&header_json).unwrap();
    file.write_all(&2_u32.to_le_bytes()).unwrap();
    file.write_all(&1_u32.to_le_bytes()).unwrap();
    file.write_all(&1_u32.to_le_bytes()).unwrap();
    file.write_all(&2_u32.to_le_bytes()).unwrap();
    drop(file);
    let loaded = load_recording(&path).unwrap();
    assert_eq!(loaded.rows.len(), 1);
    assert!(matches!(loaded.rows[0].kind, RowKind::Normal));
    assert!((loaded.rows[0].interval_secs - 5.0).abs() < 0.001);
}

#[test]
fn append_reopen_finalize_preserves_row_count() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("append.rcwf");
    let header = header_now(0.0, 1.0, 0.0, 2, 5.0, None, vec![100.0, 200.0]);
    let mut writer = RecordingWriter::create(path.clone(), &header).unwrap();
    writer
        .append_row(&SpectrogramRow {
            elapsed_secs: 0.0,
            interval_secs: 5.0,
            kind: RowKind::Normal,
            counts: vec![1, 2],
        })
        .unwrap();
    writer.finalize().unwrap();
    let mut writer = open_recording_append(path.clone()).unwrap();
    writer
        .append_row(&SpectrogramRow {
            elapsed_secs: 5.0,
            interval_secs: 5.0,
            kind: RowKind::Normal,
            counts: vec![3, 4],
        })
        .unwrap();
    writer.finalize().unwrap();
    let loaded = load_recording(&path).unwrap();
    assert_eq!(loaded.rows.len(), 2);
    assert_eq!(loaded.rows[1].counts, vec![3, 4]);
}
