use std::sync::Arc;

use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::zscale::compute_series_z_range;

impl SpectrogramState {
    pub fn persist_settings(&mut self) {
        self.settings.clamp();
        let _ = crate::spectrogram::settings::save_settings(&self.settings);
    }

    pub fn is_capture_paused(&self) -> bool {
        !self.capture_enabled
    }

    pub fn on_settings_changed(&mut self) {
        self.settings.clamp();
        let previous = self.capture.lock().ok().map(|cap| {
            (
                cap.settings.capture_interval_secs,
                cap.settings.recordings_dir.clone(),
            )
        });
        let (previous_interval, previous_dir) = previous.unwrap_or((
            self.settings.capture_interval_secs,
            self.settings.recordings_dir.clone(),
        ));
        let interval_changed =
            (previous_interval - self.settings.capture_interval_secs).abs() > 1e-9;
        let dir_changed = previous_dir != self.settings.recordings_dir;
        self.persist_settings();
        if let Ok(mut cap) = self.capture.lock() {
            cap.settings = self.settings.clone();
        }
        if interval_changed {
            self.apply_interval_change();
        }
        if dir_changed {
            self.refresh_history();
        }
        self.z_range_rows = 0;
        self.texture.dirty = true;
    }

    fn apply_interval_change(&mut self) {
        self.reset_accumulation();
        self.apply_capture_interval_to_live_header();
        self.sync_from_capture();
    }

    fn apply_capture_interval_to_live_header(&mut self) {
        let interval = self.settings.capture_interval_secs;
        if let Ok(cap) = self.capture.lock() {
            let interval = self.settings.capture_interval_secs;
            let mut progress = cap.progress.lock().expect("capture progress lock poisoned");
            if let Some(series) = progress.live_series.as_mut() {
                Arc::make_mut(series).header.interval_secs = interval;
            }
            progress.mark_dirty();
        }
        if let Some(series) = self.live_series.as_mut() {
            Arc::make_mut(series).header.interval_secs = interval;
        }
    }

    pub fn refresh_z_range(&mut self) {
        let snapshot = self.active_series().map(|series| {
            (
                compute_series_z_range(series, &self.settings),
                series.row_count(),
            )
        });
        match snapshot {
            Some((range, rows)) => {
                self.z_range = Some(range);
                self.z_range_rows = rows;
            }
            None => {
                self.z_range = None;
                self.z_range_rows = 0;
            }
        }
    }

    pub fn ensure_z_range(&mut self) {
        let rows = self
            .active_series()
            .map(|series| series.row_count())
            .unwrap_or(0);
        if self.z_range.is_none() || self.z_range_rows != rows {
            self.refresh_z_range();
        }
    }
}
