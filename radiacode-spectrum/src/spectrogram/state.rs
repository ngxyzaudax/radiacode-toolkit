#[path = "state_lifecycle.rs"]
mod state_lifecycle;
#[path = "state_recording.rs"]
mod state_recording;
#[path = "state_settings.rs"]
mod state_settings;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::peaks::PeakMemo;
use crate::scale::YScale;
use crate::spectrogram::preview::ChannelTotalsMemo;

use crate::spectrogram::baseline::IngestBaseline;
use crate::spectrogram::capture::SpectrogramCapture;
use crate::spectrogram::model::{RecordingEntry, SpectrogramDisplay, SpectrogramSeries};
use crate::spectrogram::settings::{SpectrogramSettings, load_settings};
use crate::spectrogram::texture::SpectrogramTexture;
use crate::spectrogram::view_range::SpectrogramViewRange;
use crate::spectrogram::zscale::ZScaleRange;

pub struct SpectrogramState {
    pub capture: Arc<Mutex<SpectrogramCapture>>,
    pub display: SpectrogramDisplay,
    pub live_series: Option<Arc<SpectrogramSeries>>,
    pub loaded_series: Option<SpectrogramSeries>,
    pub loaded_path: Option<PathBuf>,
    pub paused_recording_path: Option<PathBuf>,
    pub history: Vec<RecordingEntry>,
    pub library_filter: String,
    pub library_edit_path: Option<PathBuf>,
    pub library_edit_name: String,
    pub library_edit_comment: String,
    pub texture: SpectrogramTexture,
    pub texture_handle: Option<egui::TextureHandle>,
    pub status: String,
    pub settings: SpectrogramSettings,
    pub view_range: SpectrogramViewRange,
    pub last_ingested_sequence: u64,
    pub skip_next_sample: bool,
    pub reconnect_baseline_pending: bool,
    pub last_ingest_at: Option<Instant>,
    pub last_auto_save: Option<Instant>,
    pub show_grid: bool,
    pub show_count_rate: bool,
    pub show_peaks: bool,
    pub preview_scale: YScale,
    pub capture_enabled: bool,
    pub z_range: Option<ZScaleRange>,
    pub z_range_rows: usize,
    pub(crate) baseline: Option<IngestBaseline>,
    pub pane_open: bool,
    pub peak_memo: PeakMemo,
    pub totals_memo: ChannelTotalsMemo,
}

impl SpectrogramState {
    pub fn new(capture: Arc<Mutex<SpectrogramCapture>>) -> Self {
        let settings = load_settings();
        if let Ok(mut cap) = capture.lock() {
            cap.settings = settings.clone();
        }
        Self {
            capture,
            display: SpectrogramDisplay::Live,
            live_series: None,
            loaded_series: None,
            loaded_path: None,
            paused_recording_path: None,
            history: Vec::new(),
            library_filter: String::new(),
            library_edit_path: None,
            library_edit_name: String::new(),
            library_edit_comment: String::new(),
            texture: SpectrogramTexture::new(1, 1),
            texture_handle: None,
            status: String::new(),
            settings,
            view_range: SpectrogramViewRange::new(),
            last_ingested_sequence: 0,
            skip_next_sample: false,
            reconnect_baseline_pending: false,
            last_ingest_at: None,
            last_auto_save: None,
            show_grid: true,
            show_count_rate: false,
            show_peaks: false,
            preview_scale: YScale::Linear,
            capture_enabled: false,
            z_range: None,
            z_range_rows: 0,
            baseline: None,
            pane_open: false,
            peak_memo: PeakMemo::new(),
            totals_memo: ChannelTotalsMemo::new(),
        }
    }

    pub fn sync_from_capture(&mut self) {
        let Ok(cap) = self.capture.lock() else {
            return;
        };
        let Ok(progress) = cap.progress.lock() else {
            return;
        };
        if !progress.is_dirty() {
            return;
        }
        let snapshot = progress.clone();
        progress.clear_dirty();
        drop(progress);
        drop(cap);
        self.apply_progress_snapshot(snapshot);
    }

    fn apply_progress_snapshot(
        &mut self,
        snapshot: crate::spectrogram::capture_progress::CaptureProgress,
    ) {
        self.baseline = snapshot.baseline;
        let had_series = self.live_series.is_some();
        self.live_series = snapshot.live_series;
        if let Some(series) = self.live_series.as_ref() {
            if had_series {
                self.view_range
                    .set_series_energy_bounds(&series.energies_kev);
            } else {
                self.view_range.fit_series_energy(&series.energies_kev);
            }
        }
        self.paused_recording_path = snapshot.paused_recording_path;
        self.status = snapshot.status;
        self.last_ingested_sequence = snapshot.last_ingested_sequence;
        self.skip_next_sample = snapshot.skip_next_sample;
        self.reconnect_baseline_pending = snapshot.reconnect_baseline_pending;
        self.last_ingest_at = snapshot.last_ingest_at;
        self.last_auto_save = snapshot.last_auto_save;
        self.capture_enabled = snapshot.capture_enabled;
        if self.live_series.is_some() {
            self.texture.dirty = true;
        }
    }

    pub fn is_recording(&self) -> bool {
        self.capture
            .lock()
            .ok()
            .is_some_and(|cap| cap.recording.is_some())
    }

    pub fn active_series(&self) -> Option<&SpectrogramSeries> {
        match self.display {
            SpectrogramDisplay::Live => self.live_series.as_deref(),
            SpectrogramDisplay::Loaded => self.loaded_series.as_ref(),
        }
    }

    pub fn live_row_count(&self) -> usize {
        self.live_series
            .as_ref()
            .map(|series| series.row_count())
            .unwrap_or(0)
    }

    #[cfg(test)]
    pub fn ingest_spectrum(
        &mut self,
        spectrum: &crate::model::SpectrumView,
        device_serial: Option<&str>,
        sequence: u64,
    ) {
        crate::spectrogram::ingest::ingest_spectrum(self, spectrum, device_serial, sequence);
    }

    pub(crate) fn mark_texture_dirty_empty(&mut self) {
        self.texture = SpectrogramTexture::new(1, 1);
        self.texture_handle = None;
        self.texture.dirty = true;
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
