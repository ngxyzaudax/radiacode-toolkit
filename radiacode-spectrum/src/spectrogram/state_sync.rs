use crate::spectrogram::capture_lock::{lock_capture, lock_progress};
use crate::spectrogram::capture_progress::CaptureProgress;
use crate::spectrogram::state::SpectrogramState;

impl SpectrogramState {
    pub fn sync_from_capture(&mut self) {
        let Some(snapshot) = self.take_dirty_snapshot() else {
            return;
        };
        self.apply_progress_snapshot(snapshot);
    }

    fn take_dirty_snapshot(&self) -> Option<CaptureProgress> {
        let cap = lock_capture(&self.capture);
        let progress = lock_progress(&cap.progress);
        if !progress.is_dirty() {
            return None;
        }
        progress.clear_dirty();
        Some(progress.clone())
    }

    fn apply_progress_snapshot(&mut self, snapshot: CaptureProgress) {
        let had_series = self.live_series.is_some();
        self.live_series = snapshot.live_series;
        if let Some(series) = self.live_series.as_ref() {
            if had_series {
                self.view_range
                    .set_series_energy_bounds(&series.energies_kev);
            } else {
                self.view_range.fit_series_energy(&series.energies_kev);
            }
            self.texture.dirty = true;
        }
        self.paused_recording_path = snapshot.paused_recording_path;
        self.error = snapshot.error;
        self.capture_enabled = snapshot.capture_enabled;
    }
}
