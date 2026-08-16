use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectrogramDisplay {
    Live,
    Loaded,
}

#[derive(Debug, Clone)]
pub struct RecordingEntry {
    pub path: PathBuf,
    pub name: String,
    pub comment: String,
    pub created_at: String,
    pub device_serial: Option<String>,
    pub interval_secs: f64,
    pub row_count: u32,
    pub channel_count: u32,
}
