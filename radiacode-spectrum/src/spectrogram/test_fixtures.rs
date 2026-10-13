use std::path::{Path, PathBuf};

use crate::spectrogram::model::{RowKind, SpectrogramRow};
use crate::spectrogram::storage::{RecordingWriter, header_now};

pub const FIXTURE_CHANNELS: u32 = 3;
pub const FIXTURE_INTERVAL_SECS: f64 = 10.0;

pub fn write_fixture_recording(dir: &Path, file_stem: &str, rows: &[Vec<u32>]) -> PathBuf {
    let path = dir.join(format!("{file_stem}.rcwf"));
    let header = header_now(
        0.0,
        1.0,
        0.0,
        FIXTURE_CHANNELS,
        FIXTURE_INTERVAL_SECS,
        Some("SN-FIXTURE".into()),
        vec![10.0, 20.0, 30.0],
    );
    let mut writer = RecordingWriter::create(path.clone(), &header).expect("create recording");
    for (index, counts) in rows.iter().enumerate() {
        writer
            .append_row(&SpectrogramRow {
                elapsed_secs: index as f64 * FIXTURE_INTERVAL_SECS,
                interval_secs: FIXTURE_INTERVAL_SECS,
                kind: RowKind::Normal,
                counts: counts.clone(),
            })
            .expect("append row");
    }
    writer.finalize().expect("finalize recording")
}

pub fn dir_string(dir: &Path) -> String {
    dir.display().to_string()
}
