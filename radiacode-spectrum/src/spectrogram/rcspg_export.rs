use std::path::Path;

use crate::spectrogram::model::{SpectrogramRow, SpectrogramSeries};

pub fn export_recording(
    path: &Path,
    series: &SpectrogramSeries,
    name: &str,
    comment: &str,
) -> std::io::Result<()> {
    let header_line = build_header_line(series, name, comment);
    let spectrum_line = encode_historical_spectrum(series);
    let mut body = String::new();
    for row in &series.rows {
        body.push_str(&format_row(row));
        body.push('\n');
    }
    let content = format!("{header_line}\n{spectrum_line}\n{body}");
    std::fs::write(path, content.as_bytes())?;
    Ok(())
}

fn build_header_line(series: &SpectrogramSeries, name: &str, comment: &str) -> String {
    format!(
        "Spectrogram: {name}\tTime: {}\tTimestamp: 0\tAccumulation time: {}\tChannels: {}\tDevice serial: {}\tFlags: 0\tComment: {comment}",
        series.header.created_at,
        series.header.interval_secs as u32,
        series.header.channel_count,
        series.header.device_serial.as_deref().unwrap_or(""),
    )
}

fn encode_historical_spectrum(series: &SpectrogramSeries) -> String {
    let cumulative: Vec<u32> = if let Some(first) = series.rows.first() {
        first.counts.clone()
    } else {
        vec![0; series.header.channel_count as usize]
    };
    let duration = series.duration_secs().max(0.0) as u32;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&duration.to_le_bytes());
    for value in [series.header.a0, series.header.a1, series.header.a2] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in &cumulative {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    let hex: String = bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("Spectrum: {hex}")
}

fn format_row(row: &SpectrogramRow) -> String {
    let mut parts = vec![
        (row.elapsed_secs as u64).to_string(),
        row.interval_secs.round().to_string(),
        row.kind.storage_tag().to_string(),
        row.kind.storage_extra().to_string(),
    ];
    parts.extend(row.counts.iter().map(|value| value.to_string()));
    parts.join(" ")
}
