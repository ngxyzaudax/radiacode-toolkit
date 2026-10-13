use std::path::Path;

use crate::spectrogram::model::{SpectrogramHeader, SpectrogramSeries};
use crate::spectrogram::rcspg_import_rows::parse_rows;
use crate::spectrogram::storage::le_u32_words;

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
    let rows = parse_rows(lines, channels as usize);
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
    let bytes = decode_hex_bytes(&hex)?;
    if bytes.len() < 16 {
        return Err(invalid("spectrum line too short"));
    }
    let words = le_u32_words(&bytes);
    let calibration = [
        f32::from_bits(words[1]),
        f32::from_bits(words[2]),
        f32::from_bits(words[3]),
    ];
    Ok(HistoricalSpectrum {
        calibration,
        counts: words[4..].to_vec(),
    })
}

fn decode_hex_bytes(hex: &str) -> std::io::Result<Vec<u8>> {
    if !hex.is_ascii() || !hex.len().is_multiple_of(2) {
        return Err(invalid("spectrum hex payload malformed"));
    }
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| invalid(&error.to_string()))
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

fn invalid(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.to_string())
}

#[cfg(test)]
mod rcspg_import_tests;
