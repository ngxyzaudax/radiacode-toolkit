use std::path::Path;

use crate::spectrogram::model::{RowKind, SpectrogramHeader, SpectrogramRow, SpectrogramSeries};

pub fn import_recording(path: &Path) -> std::io::Result<SpectrogramSeries> {
    let content = std::fs::read_to_string(path)?;
    let mut lines = content.lines();
    let header_line = lines.next().ok_or_else(|| invalid("missing header"))?;
    if !header_line.starts_with("Spectrogram:") {
        return Err(invalid("not an rcspg file"));
    }
    let spectrum_line = lines
        .next()
        .ok_or_else(|| invalid("missing spectrum line"))?;
    let hist = decode_historical_spectrum(spectrum_line)?;
    let channels = parse_header_field(header_line, "Channels")
        .and_then(|value| value.parse().ok())
        .unwrap_or(hist.counts.len() as u32);
    let interval = parse_header_field(header_line, "Accumulation time")
        .and_then(|value| value.parse().ok())
        .unwrap_or(60.0);
    let serial = parse_header_field(header_line, "Device serial").map(str::to_string);
    let created = parse_header_field(header_line, "Time")
        .unwrap_or("unknown")
        .to_string();
    let a0 = hist.calibration[0];
    let a1 = hist.calibration[1];
    let a2 = hist.calibration[2];
    let energies_kev: Vec<f64> = (0..channels as usize)
        .map(|index| {
            let ch = index as f64;
            f64::from(a0) + f64::from(a1) * ch + f64::from(a2) * ch * ch
        })
        .collect();
    let header = SpectrogramHeader {
        created_at: created,
        a0,
        a1,
        a2,
        channel_count: channels,
        interval_secs: interval,
        device_serial: serial,
        energies_kev: energies_kev.clone(),
    };
    let mut rows = Vec::new();
    let mut elapsed = 0.0;
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let values = parse_number_list(line);
        if values.len() < 2 {
            continue;
        }
        let row_interval = values[1] as f64;
        let kind = if values.len() >= 4 {
            let raw_total = values
                .iter()
                .skip(4)
                .take(channels as usize)
                .map(|&value| value as u64)
                .sum();
            RowKind::from_storage_tag(values[2] as u8, values[3] as f64, raw_total)
        } else {
            RowKind::Normal
        };
        let count_offset = if values.len() >= 4 { 4 } else { 2 };
        let counts: Vec<u32> = values
            .iter()
            .skip(count_offset)
            .take(channels as usize)
            .map(|&value| value as u32)
            .collect();
        rows.push(SpectrogramRow {
            elapsed_secs: elapsed,
            interval_secs: row_interval,
            kind,
            counts,
        });
        elapsed += row_interval;
    }
    Ok(SpectrogramSeries {
        header,
        energies_kev,
        rows,
    })
}

struct HistoricalSpectrum {
    calibration: [f32; 3],
    counts: Vec<u32>,
}

fn decode_historical_spectrum(line: &str) -> std::io::Result<HistoricalSpectrum> {
    let hex = line
        .trim()
        .strip_prefix("Spectrum:")
        .ok_or_else(|| invalid("missing spectrum prefix"))?
        .replace(' ', "");
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| invalid(&error.to_string()))?;
    if bytes.len() < 16 {
        return Err(invalid("spectrum line too short"));
    }
    let _duration = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let calibration = [
        f32::from_le_bytes(bytes[4..8].try_into().unwrap()),
        f32::from_le_bytes(bytes[8..12].try_into().unwrap()),
        f32::from_le_bytes(bytes[12..16].try_into().unwrap()),
    ];
    let counts = bytes[16..]
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();
    Ok(HistoricalSpectrum {
        calibration,
        counts,
    })
}

fn parse_header_field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.split('\t').find_map(|field| {
        let mut parts = field.splitn(2, ':');
        let name = parts.next()?.trim();
        if name == key {
            parts.next().map(str::trim)
        } else {
            None
        }
    })
}

fn parse_number_list(line: &str) -> Vec<i64> {
    line.split_whitespace()
        .filter_map(|token| token.parse().ok())
        .collect()
}

fn invalid(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.to_string())
}

#[cfg(test)]
mod rcspg_import_tests;
