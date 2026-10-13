use tracing::info;

use crate::spectrogram::capture_lock::{lock_capture, lock_progress};
use crate::spectrogram::model::SpectrogramDisplay;
use crate::spectrogram::state::SpectrogramState;

impl SpectrogramState {
    pub fn on_reconnect(&mut self) {
        lock_capture(&self.capture).on_reconnect();
        self.sync_from_capture();
    }

    pub fn on_tab_enter(&mut self) {
        if self.live_series.is_some() {
            self.texture.dirty = true;
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
        lock_capture(&self.capture).on_disconnect();
        self.live_series = None;
        self.close_loaded();
        self.capture_enabled = false;
        self.view_range.reset();
        self.clear_z_range();
        self.mark_texture_dirty_empty();
    }

    pub fn reset_accumulation(&mut self) {
        if self.is_recording() {
            return;
        }
        if self.display == SpectrogramDisplay::Loaded {
            self.close_loaded();
        }
        {
            let cap = lock_capture(&self.capture);
            lock_progress(&cap.progress).reset_accumulation();
        }
        self.sync_from_capture();
        self.fit_view_to_live_series();
        self.clear_z_range();
        self.mark_texture_dirty_empty();
        info!("spectrogram accumulation reset");
    }

    pub fn reset_view(&mut self) {
        let energies = self
            .active_series()
            .map(|series| series.energies_kev.clone());
        match energies {
            Some(energies_kev) => self.view_range.fit_series_energy(&energies_kev),
            None => self.view_range.reset(),
        }
        self.texture.dirty = true;
    }

    fn fit_view_to_live_series(&mut self) {
        match self.live_series.as_ref() {
            Some(series) => self.view_range.fit_series_energy(&series.energies_kev),
            None => self.view_range.reset(),
        }
    }

    fn clear_z_range(&mut self) {
        self.z_range = None;
        self.z_range_rows = 0;
    }
}
