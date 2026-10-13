use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};
use tracing::{debug, info};

use crate::model::SpectrumView;
use crate::spectrogram::capture_lock::{lock_capture, lock_progress};
use crate::spectrogram::capture_progress::CaptureProgress;
use crate::spectrogram::ingest;
use crate::spectrogram::settings::{SpectrogramSettings, load_settings};
use crate::spectrogram::storage::RecordingWriter;
use crate::worker::WorkerEvent;

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

    pub fn with_progress_mut<R>(&self, f: impl FnOnce(&mut CaptureProgress) -> R) -> R {
        f(&mut lock_progress(&self.progress))
    }

    pub fn on_session_connect(&mut self, serial: &str) {
        self.device_serial = Some(serial.to_string());
        self.with_progress_mut(CaptureProgress::on_session_connect);
    }

    pub fn on_reconnect(&mut self) {
        self.with_progress_mut(CaptureProgress::on_reconnect);
    }

    pub fn on_disconnect(&mut self) {
        self.device_serial = None;
        self.with_progress_mut(CaptureProgress::on_disconnect);
    }

    pub fn ingest_spectrum(&mut self, spectrum: &SpectrumView) {
        let next_sequence = self.with_progress_mut(|progress| {
            progress
                .capture_enabled
                .then(|| progress.last_ingested_sequence.saturating_add(1))
        });
        if let Some(sequence) = next_sequence {
            ingest::ingest_capture(self, spectrum, sequence);
        }
    }

    pub fn maybe_auto_save(&mut self) {
        ingest::maybe_auto_save_capture(self);
    }

    fn on_worker_event(&mut self, event: &WorkerEvent) {
        match event {
            WorkerEvent::Spectrum(spectrum) => {
                self.ingest_spectrum(spectrum);
                self.maybe_auto_save();
            }
            WorkerEvent::Connected(info) => self.on_session_connect(&info.serial),
            WorkerEvent::Reconnecting => self.on_reconnect(),
            WorkerEvent::Disconnected => self.on_disconnect(),
            _ => {}
        }
    }
}

pub fn spawn_capture_router(
    worker_events: Receiver<WorkerEvent>,
    ui_events: Sender<WorkerEvent>,
    capture: Arc<Mutex<SpectrogramCapture>>,
) {
    std::thread::spawn(move || {
        debug!("spectrogram capture router ready");
        while let Ok(event) = worker_events.recv() {
            lock_capture(&capture).on_worker_event(&event);
            if ui_events.send(event).is_err() {
                break;
            }
        }
        info!("spectrogram capture router stopped");
    });
}
