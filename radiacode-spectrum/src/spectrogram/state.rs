#[path = "state_lifecycle.rs"]
mod state_lifecycle;
#[path = "state_recording.rs"]
mod state_recording;
#[path = "state_settings.rs"]
mod state_settings;
#[path = "state_sync.rs"]
mod state_sync;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::peaks::PeakMemo;
use crate::scale::YScale;
use crate::spectrogram::capture::SpectrogramCapture;
use crate::spectrogram::capture_lock::lock_capture;
use crate::spectrogram::model::{RecordingEntry, SpectrogramDisplay, SpectrogramSeries};
use crate::spectrogram::preview::ChannelTotalsMemo;
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
    pub error: String,
    pub settings: SpectrogramSettings,
    pub view_range: SpectrogramViewRange,
    pub show_peaks: bool,
    pub preview_scale: YScale,
    pub capture_enabled: bool,
    pub z_range: Option<ZScaleRange>,
    pub z_range_rows: usize,
    pub pane_open: bool,
    pub peak_memo: PeakMemo,
    pub totals_memo: ChannelTotalsMemo,
}

impl SpectrogramState {
    pub fn new(capture: Arc<Mutex<SpectrogramCapture>>) -> Self {
        let settings = load_settings();
        lock_capture(&capture).settings = settings.clone();
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
            error: String::new(),
            settings,
            view_range: SpectrogramViewRange::new(),
            show_peaks: false,
            preview_scale: YScale::Linear,
            capture_enabled: false,
            z_range: None,
            z_range_rows: 0,
            pane_open: false,
            peak_memo: PeakMemo::new(),
            totals_memo: ChannelTotalsMemo::new(),
        }
    }

    pub fn is_recording(&self) -> bool {
        lock_capture(&self.capture).recording.is_some()
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
