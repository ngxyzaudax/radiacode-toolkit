use std::sync::{Arc, Mutex, MutexGuard};

use crossbeam_channel::{Receiver, Sender};
use tracing::{debug, info, warn};

use crate::model::SpectrumView;
use crate::spectrogram::capture_progress::CaptureProgress;
use crate::spectrogram::ingest;
use crate::spectrogram::settings::{SpectrogramSettings, load_settings};
use crate::spectrogram::storage::RecordingWriter;

pub struct SpectrogramCapture {
    pub progress: Arc<Mutex<CaptureProgress>>,
    pub recording: Option<RecordingWriter>,
    pub settings: SpectrogramSettings,
    pub device_serial: Option<String>,
}

impl SpectrogramCapture {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(Mutex::new(CaptureProgress::new())),
            recording: None,
            settings: load_settings(),
            device_serial: None,
        }
    }

    pub fn with_progress_mut<R>(&mut self, f: impl FnOnce(&mut CaptureProgress) -> R) -> Option<R> {
        lock_capture_progress(&self.progress).map(|mut progress| f(&mut progress))
    }

    pub fn on_session_connect(&mut self, serial: &str) {
        self.device_serial = Some(serial.to_string());
        let _ = self.with_progress_mut(|progress| {
            progress.capture_enabled = true;
            progress.skip_next_sample = true;
            progress.mark_dirty();
        });
    }

    pub fn on_reconnect(&mut self) {
        let _ = self.with_progress_mut(|progress| {
            progress.skip_next_sample = true;
            progress.reconnect_baseline_pending = true;
            progress.baseline = None;
            progress.last_ingest_at = None;
            progress.mark_dirty();
        });
    }

    pub fn on_disconnect(&mut self) {
        self.device_serial = None;
        let _ = self.with_progress_mut(|progress| {
            progress.capture_enabled = false;
            progress.live_series = None;
            progress.last_ingested_sequence = 0;
            progress.skip_next_sample = false;
            progress.reconnect_baseline_pending = false;
            progress.last_ingest_at = None;
            progress.baseline = None;
            progress.last_auto_save = None;
            progress.mark_dirty();
        });
    }

    pub fn ingest_spectrum(&mut self, spectrum: &SpectrumView) {
        let enabled =
            lock_capture_progress(&self.progress).is_some_and(|progress| progress.capture_enabled);
        if !enabled {
            return;
        }
        let sequence = lock_capture_progress(&self.progress)
            .map(|progress| progress.last_ingested_sequence.saturating_add(1))
            .unwrap_or(1);
        ingest::ingest_capture(self, spectrum, sequence);
    }

    pub fn maybe_auto_save(&mut self) {
        ingest::maybe_auto_save_capture(self);
    }
}

pub fn spawn_capture_router(
    worker_events: Receiver<crate::worker::WorkerEvent>,
    ui_events: Sender<crate::worker::WorkerEvent>,
    capture: Arc<Mutex<SpectrogramCapture>>,
) {
    std::thread::spawn(move || {
        debug!("spectrogram capture router ready");
        while let Ok(event) = worker_events.recv() {
            match &event {
                crate::worker::WorkerEvent::Spectrum(spectrum) => {
                    if let Some(mut cap) = lock_capture(&capture) {
                        cap.ingest_spectrum(spectrum);
                        cap.maybe_auto_save();
                    }
                }
                crate::worker::WorkerEvent::Connected(info) => {
                    if let Some(mut cap) = lock_capture(&capture) {
                        cap.on_session_connect(&info.serial);
                    }
                }
                crate::worker::WorkerEvent::Reconnecting => {
                    if let Some(mut cap) = lock_capture(&capture) {
                        cap.on_reconnect();
                    }
                }
                crate::worker::WorkerEvent::Disconnected => {
                    if let Some(mut cap) = lock_capture(&capture) {
                        cap.on_disconnect();
                    }
                }
                _ => {}
            }
            if ui_events.send(event).is_err() {
                break;
            }
        }
        info!("spectrogram capture router stopped");
    });
}

fn lock_capture(
    capture: &Arc<Mutex<SpectrogramCapture>>,
) -> Option<MutexGuard<'_, SpectrogramCapture>> {
    match capture.lock() {
        Ok(guard) => Some(guard),
        Err(error) => {
            warn!("capture mutex poisoned, recovering");
            Some(error.into_inner())
        }
    }
}

fn lock_capture_progress(
    progress: &Arc<Mutex<CaptureProgress>>,
) -> Option<MutexGuard<'_, CaptureProgress>> {
    match progress.lock() {
        Ok(guard) => Some(guard),
        Err(error) => {
            warn!("capture progress mutex poisoned, recovering");
            Some(error.into_inner())
        }
    }
}
