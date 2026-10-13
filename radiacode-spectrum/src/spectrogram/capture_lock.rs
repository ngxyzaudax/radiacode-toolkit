use std::sync::{Mutex, MutexGuard};

use tracing::warn;

use crate::spectrogram::capture::SpectrogramCapture;
use crate::spectrogram::capture_progress::CaptureProgress;

pub fn lock_capture(capture: &Mutex<SpectrogramCapture>) -> MutexGuard<'_, SpectrogramCapture> {
    capture.lock().unwrap_or_else(|poisoned| {
        warn!("capture mutex poisoned, recovering");
        poisoned.into_inner()
    })
}

pub fn lock_progress(progress: &Mutex<CaptureProgress>) -> MutexGuard<'_, CaptureProgress> {
    progress.lock().unwrap_or_else(|poisoned| {
        warn!("capture progress mutex poisoned, recovering");
        poisoned.into_inner()
    })
}
