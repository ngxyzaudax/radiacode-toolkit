use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crate::spectrogram::baseline::IngestBaseline;
use crate::spectrogram::model::SpectrogramSeries;

pub struct CaptureProgress {
    pub live_series: Option<Arc<SpectrogramSeries>>,
    pub paused_recording_path: Option<PathBuf>,
    pub error: String,
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
            error: String::new(),
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

    pub fn begin_capture(&mut self, skip_next_sample: bool) {
        self.capture_enabled = true;
        self.skip_next_sample = skip_next_sample;
        self.last_auto_save = None;
        self.error.clear();
        self.mark_dirty();
    }

    pub fn pause_capture(&mut self) {
        self.capture_enabled = false;
        self.error.clear();
        self.mark_dirty();
    }

    pub fn on_session_connect(&mut self) {
        self.capture_enabled = true;
        self.skip_next_sample = true;
        self.mark_dirty();
    }

    pub fn on_reconnect(&mut self) {
        self.skip_next_sample = true;
        self.reconnect_baseline_pending = true;
        self.baseline = None;
        self.last_ingest_at = None;
        self.mark_dirty();
    }

    pub fn on_disconnect(&mut self) {
        self.live_series = None;
        self.capture_enabled = false;
        self.skip_next_sample = false;
        self.last_auto_save = None;
        self.reset_ingest_tracking();
    }

    pub fn reset_accumulation(&mut self) {
        if let Some(series) = self.live_series.as_mut() {
            Arc::make_mut(series).rows.clear();
        }
        self.skip_next_sample = true;
        self.error.clear();
        self.reset_ingest_tracking();
    }

    fn reset_ingest_tracking(&mut self) {
        self.last_ingested_sequence = 0;
        self.reconnect_baseline_pending = false;
        self.last_ingest_at = None;
        self.baseline = None;
        self.mark_dirty();
    }

    pub fn set_live_interval_secs(&mut self, interval_secs: f64) {
        if let Some(series) = self.live_series.as_mut() {
            Arc::make_mut(series).header.interval_secs = interval_secs;
        }
        self.mark_dirty();
    }
}

impl Clone for CaptureProgress {
    fn clone(&self) -> Self {
        Self {
            live_series: self.live_series.clone(),
            paused_recording_path: self.paused_recording_path.clone(),
            error: self.error.clone(),
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
