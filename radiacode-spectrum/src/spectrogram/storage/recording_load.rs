use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::spectrogram::model::{RowKind, SpectrogramHeader, SpectrogramRow, SpectrogramSeries};

use super::storage_format::{
    VERSION_V1, VERSION_V2, assign_elapsed_secs, le_u32_words, read_f64, read_recording_prefix,
    read_row_counts,
};

pub fn load_recording(path: &Path) -> std::io::Result<SpectrogramSeries> {
    let mut file = File::open(path)?;
    let (version, header, channel_count, row_count) = read_recording_prefix(&mut file)?;
    let channel_count = channel_count as usize;
    let row_count = row_count as usize;
    let mut rows = match version {
        VERSION_V1 => read_rows_v1(&mut file, &header, channel_count, row_count)?,
        VERSION_V2 => read_rows_v2(&mut file, channel_count, row_count)?,
        _ => return Err(invalid_data("unsupported spectrogram file version")),
    };
    assign_elapsed_secs(&mut rows);
    let energies_kev = header.energies_kev.clone();
    Ok(SpectrogramSeries {
        header,
        energies_kev,
        rows,
    })
}

fn read_rows_v1(
    file: &mut File,
    header: &SpectrogramHeader,
    channel_count: usize,
    row_count: usize,
) -> std::io::Result<Vec<SpectrogramRow>> {
    let row_bytes = channel_count
        .checked_mul(4)
        .ok_or_else(|| invalid_data("channel count overflow"))?;
    let payload_len = row_bytes
        .checked_mul(row_count)
        .ok_or_else(|| invalid_data("row count overflow"))?;
    let mut payload = vec![0_u8; payload_len];
    file.read_exact(&mut payload)?;
    Ok((0..row_count)
        .map(|index| &payload[index * row_bytes..(index + 1) * row_bytes])
        .map(|row| row_v1(header, le_u32_words(row)))
        .collect())
}

fn row_v1(header: &SpectrogramHeader, counts: Vec<u32>) -> SpectrogramRow {
    SpectrogramRow {
        elapsed_secs: 0.0,
        interval_secs: header.interval_secs,
        kind: RowKind::Normal,
        counts,
    }
}

fn read_rows_v2(
    file: &mut File,
    channel_count: usize,
    row_count: usize,
) -> std::io::Result<Vec<SpectrogramRow>> {
    (0..row_count)
        .map(|_| read_row_v2(file, channel_count))
        .collect()
}

fn read_row_v2(file: &mut File, channel_count: usize) -> std::io::Result<SpectrogramRow> {
    let counts = read_row_counts(file, channel_count)?;
    let interval_secs = read_f64(file)?;
    let mut kind_tag = [0_u8; 1];
    file.read_exact(&mut kind_tag)?;
    let extra = read_f64(file)?;
    let raw_total = counts.iter().map(|&value| value as u64).sum();
    Ok(SpectrogramRow {
        elapsed_secs: 0.0,
        interval_secs,
        kind: RowKind::from_storage_tag(kind_tag[0], extra, raw_total),
        counts,
    })
}

fn invalid_data(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.to_string())
}
