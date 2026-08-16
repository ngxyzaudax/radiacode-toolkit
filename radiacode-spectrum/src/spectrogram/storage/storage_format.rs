use std::fs::File;
use std::io::{Read, Write};

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::spectrogram::model::{SpectrogramHeader, SpectrogramRow};

pub(crate) const MAGIC: &[u8; 4] = b"RCWF";
pub(crate) const VERSION_V1: u32 = 1;
pub(crate) const VERSION_V2: u32 = 2;
pub(crate) const VERSION_CURRENT: u32 = VERSION_V2;

pub fn header_now(
    a0: f32,
    a1: f32,
    a2: f32,
    channel_count: u32,
    interval_secs: f64,
    device_serial: Option<String>,
    energies_kev: Vec<f64>,
) -> SpectrogramHeader {
    let created_at = match OffsetDateTime::now_local() {
        Ok(now) => now,
        Err(_) => OffsetDateTime::now_utc(),
    }
    .format(&Rfc3339)
    .unwrap_or_else(|_| "unknown".into());
    SpectrogramHeader {
        created_at,
        a0,
        a1,
        a2,
        channel_count,
        interval_secs,
        device_serial,
        energies_kev,
    }
}

pub(crate) fn read_recording_prefix(
    file: &mut File,
) -> std::io::Result<(u32, SpectrogramHeader, u32, u32)> {
    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid spectrogram file magic",
        ));
    }
    let version = read_u32(file)?;
    let header_len = read_u32(file)? as usize;
    let mut header_bytes = vec![0_u8; header_len];
    file.read_exact(&mut header_bytes)?;
    let header: SpectrogramHeader = serde_json::from_slice(&header_bytes)?;
    let channel_count = read_u32(file)?;
    let row_count = read_u32(file)?;
    Ok((version, header, channel_count, row_count))
}

pub(crate) fn assign_elapsed_secs(rows: &mut [SpectrogramRow]) {
    let mut elapsed = 0.0;
    for row in rows.iter_mut() {
        row.elapsed_secs = elapsed;
        elapsed += row.interval_secs;
    }
}

pub(crate) fn read_row_counts(
    reader: &mut File,
    channel_count: usize,
) -> std::io::Result<Vec<u32>> {
    let mut counts = Vec::with_capacity(channel_count);
    for _ in 0..channel_count {
        counts.push(read_u32(reader)?);
    }
    Ok(counts)
}

pub(crate) fn write_u32(writer: &mut impl Write, value: u32) -> std::io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

pub(crate) fn read_u32(reader: &mut File) -> std::io::Result<u32> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

pub(crate) fn write_f64(writer: &mut impl Write, value: f64) -> std::io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

pub(crate) fn read_f64(reader: &mut File) -> std::io::Result<f64> {
    let mut bytes = [0_u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(f64::from_le_bytes(bytes))
}
