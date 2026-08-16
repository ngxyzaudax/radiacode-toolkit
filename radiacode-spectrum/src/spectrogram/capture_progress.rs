use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use std::sync::Arc;

use crate::spectrogram::baseline::IngestBaseline;
use crate::spectrogram::model::SpectrogramSeries;

pub struct CaptureProgress {
    pub live_series: Option<Arc<SpectrogramSeries>>,
    pub paused_recording_path: Option<PathBuf>,
    pub status: String,
    pub last_ingested_sequence: u64,
    pub skip_next_sample: bool,
    pub reconnect_baseline_pending: bool,
    pub last_ingest_at: Option<Instant>,
    pub last_auto_save: Option<Instant>,
    pub capture_enabled: bool,
    pub baseline: Option<IngestBaseline>,
    dirty: AtomicBool,
}

impl CaptureProgress {
    pub fn new() -> Self {
        Self {
            live_series: None,
            paused_recording_path: None,
            status: String::new(),
            last_ingested_sequence: 0,
            skip_next_sample: false,
            reconnect_baseline_pending: false,
            last_ingest_at: None,
            last_auto_save: None,
            capture_enabled: false,
            baseline: None,
            dirty: AtomicBool::new(false),
        }
    }

    pub fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Release);
    }

    pub fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Release);
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Acquire)
    }
}

impl Clone for CaptureProgress {
    fn clone(&self) -> Self {
        Self {
            live_series: self.live_series.clone(),
            paused_recording_path: self.paused_recording_path.clone(),
            status: self.status.clone(),
            last_ingested_sequence: self.last_ingested_sequence,
            skip_next_sample: self.skip_next_sample,
            reconnect_baseline_pending: self.reconnect_baseline_pending,
            last_ingest_at: self.last_ingest_at,
            last_auto_save: self.last_auto_save,
            capture_enabled: self.capture_enabled,
            baseline: self.baseline.clone(),
            dirty: AtomicBool::new(self.dirty.load(Ordering::Relaxed)),
        }
    }
}

impl Default for CaptureProgress {
    fn default() -> Self {
        Self::new()
    }
}
