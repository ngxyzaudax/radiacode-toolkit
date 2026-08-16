use crate::spectrogram::model::RecordingEntry;

pub fn filter_recordings(entries: &[RecordingEntry], filter: &str) -> Vec<RecordingEntry> {
    let needle = filter.trim().to_lowercase();
    if needle.is_empty() {
        return entries.to_vec();
    }
    entries
        .iter()
        .filter(|entry| {
            entry.name.to_lowercase().contains(&needle)
                || entry.comment.to_lowercase().contains(&needle)
                || entry
                    .device_serial
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&needle)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::filter_recordings;
    use crate::spectrogram::model::RecordingEntry;

    fn entry(name: &str, comment: &str, serial: Option<&str>) -> RecordingEntry {
        RecordingEntry {
            path: PathBuf::from(name),
            name: name.into(),
            comment: comment.into(),
            created_at: String::new(),
            device_serial: serial.map(str::to_string),
            interval_secs: 1.0,
            row_count: 1,
            channel_count: 1,
        }
    }

    #[test]
    fn matches_name_comment_and_serial() {
        let entries = vec![
            entry("alpha", "", None),
            entry("beta", "gamma note", None),
            entry("other", "", Some("RC-12345")),
        ];
        assert_eq!(filter_recordings(&entries, "alp").len(), 1);
        assert_eq!(filter_recordings(&entries, "gamma").len(), 1);
        assert_eq!(filter_recordings(&entries, "12345").len(), 1);
    }
}
