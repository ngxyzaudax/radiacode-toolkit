use crate::spectrogram::model::{RowKind, SpectrogramRow};

const TAGGED_ROW_MIN_FIELDS: usize = 4;
const UNTAGGED_COUNT_OFFSET: usize = 2;

pub(super) fn parse_rows(
    lines: impl Iterator<Item = impl AsRef<str>>,
    channels: usize,
) -> Vec<SpectrogramRow> {
    let mut elapsed = 0.0;
    lines
        .filter_map(|line| parse_row_values(line.as_ref()))
        .map(|values| {
            let row = row_from_values(&values, channels, elapsed);
            elapsed += row.interval_secs;
            row
        })
        .collect()
}

fn parse_row_values(line: &str) -> Option<Vec<f64>> {
    let values = parse_number_list(line);
    (values.len() >= 2).then_some(values)
}

fn row_from_values(values: &[f64], channels: usize, elapsed_secs: f64) -> SpectrogramRow {
    let tagged = values.len() >= TAGGED_ROW_MIN_FIELDS;
    let count_offset = if tagged {
        TAGGED_ROW_MIN_FIELDS
    } else {
        UNTAGGED_COUNT_OFFSET
    };
    let counts = channel_counts(values, count_offset, channels);
    let kind = if tagged {
        let raw_total = counts.iter().map(|&value| u64::from(value)).sum();
        RowKind::from_storage_tag(values[2].round() as u8, values[3], raw_total)
    } else {
        RowKind::Normal
    };
    SpectrogramRow {
        elapsed_secs,
        interval_secs: values[1],
        kind,
        counts,
    }
}

fn channel_counts(values: &[f64], offset: usize, channels: usize) -> Vec<u32> {
    values
        .iter()
        .skip(offset)
        .take(channels)
        .map(|&value| value.max(0.0).round() as u32)
        .collect()
}

pub(super) fn parse_number_list(line: &str) -> Vec<f64> {
    line.split_whitespace()
        .filter_map(|token| token.parse().ok())
        .collect()
}
