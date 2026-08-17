use std::sync::Arc;

use tracing::info;

use crate::spectrogram::model::SpectrogramDisplay;
use crate::spectrogram::state::SpectrogramState;

impl SpectrogramState {
    pub fn on_reconnect(&mut self) {
        if let Ok(mut cap) = self.capture.lock() {
            cap.on_reconnect();
        }
        self.sync_from_capture();
    }

    pub fn on_tab_enter(&mut self) {
        if self.live_series.is_some() {
            self.texture.dirty = true;
        } else {
            self.skip_next_sample = true;
        }
    }

    pub fn close_loaded(&mut self) {
        self.loaded_series = None;
        self.loaded_path = None;
        self.display = SpectrogramDisplay::Live;
        self.z_range_rows = 0;
        self.texture.dirty = true;
    }

    pub fn on_disconnect(&mut self) {
        let _ = self.stop_recording();
        if let Ok(mut cap) = self.capture.lock() {
            cap.on_disconnect();
        }
        self.live_series = None;
        self.display = SpectrogramDisplay::Live;
        self.loaded_series = None;
        self.loaded_path = None;
        self.last_ingested_sequence = 0;
        self.skip_next_sample = false;
        self.reconnect_baseline_pending = false;
        self.last_ingest_at = None;
        self.baseline = None;
        self.last_auto_save = None;
        self.capture_enabled = false;
        self.z_range = None;
        self.z_range_rows = 0;
        self.view_range.reset();
        self.mark_texture_dirty_empty();
    }

    pub fn reset_accumulation(&mut self) {
        if self.is_recording() {
            return;
        }
        if self.display == SpectrogramDisplay::Loaded {
            self.close_loaded();
        }
        if let Ok(cap) = self.capture.lock() {
            let mut progress = cap.progress.lock().expect("capture progress lock poisoned");
            if let Some(series) = progress.live_series.as_mut() {
                Arc::make_mut(series).rows.clear();
            }
            progress.baseline = None;
            progress.skip_next_sample = true;
            progress.reconnect_baseline_pending = false;
            progress.last_ingest_at = None;
            progress.last_ingested_sequence = 0;
            progress.error.clear();
            progress.mark_dirty();
        }
        self.sync_from_capture();
        if let Some(series) = self.live_series.as_ref() {
            self.view_range.fit_series_energy(&series.energies_kev);
        } else {
            self.view_range.reset();
        }
        self.z_range = None;
        self.z_range_rows = 0;
        self.mark_texture_dirty_empty();
        info!("spectrogram accumulation reset");
    }

    pub fn reset_view(&mut self) {
        let energies = self
            .active_series()
            .map(|series| series.energies_kev.clone());
        if let Some(energies_kev) = energies {
            self.view_range.fit_series_energy(&energies_kev);
        } else {
            self.view_range.reset();
        }
        self.texture.dirty = true;
    }
}
