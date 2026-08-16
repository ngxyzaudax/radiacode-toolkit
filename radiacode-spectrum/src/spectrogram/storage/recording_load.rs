use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::spectrogram::model::{RowKind, SpectrogramRow, SpectrogramSeries};

use super::storage_format::{
    VERSION_V1, VERSION_V2, assign_elapsed_secs, read_f64, read_recording_prefix, read_row_counts,
};

pub fn load_recording(path: &Path) -> std::io::Result<SpectrogramSeries> {
    let mut file = File::open(path)?;
    let (version, header, channel_count, row_count) = read_recording_prefix(&mut file)?;
    if version != VERSION_V1 && version != VERSION_V2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unsupported spectrogram file version",
        ));
    }
    let channel_count = channel_count as usize;
    let row_count = row_count as usize;
    let mut rows = Vec::with_capacity(row_count);
    if version == VERSION_V1 {
        let mut payload = vec![0_u8; row_count * channel_count * 4];
        file.read_exact(&mut payload)?;
        for row_index in 0..row_count {
            let start = row_index * channel_count * 4;
            let counts = payload[start..start + channel_count * 4]
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();
            rows.push(SpectrogramRow {
                elapsed_secs: 0.0,
                interval_secs: header.interval_secs,
                kind: RowKind::Normal,
                counts,
            });
        }
    } else {
        for _ in 0..row_count {
            let counts = read_row_counts(&mut file, channel_count)?;
            let interval_secs = read_f64(&mut file)?;
            let mut kind_tag = [0_u8; 1];
            file.read_exact(&mut kind_tag)?;
            let extra = read_f64(&mut file)?;
            let raw_total = counts.iter().map(|&value| value as u64).sum();
            rows.push(SpectrogramRow {
                elapsed_secs: 0.0,
                interval_secs,
                kind: RowKind::from_storage_tag(kind_tag[0], extra, raw_total),
                counts,
            });
        }
    }
    assign_elapsed_secs(&mut rows);
    let energies_kev = header.energies_kev.clone();
    Ok(SpectrogramSeries {
        header,
        energies_kev,
        rows,
    })
}
