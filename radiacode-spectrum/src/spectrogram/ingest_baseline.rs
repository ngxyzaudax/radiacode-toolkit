use std::time::Instant;

use tracing::debug;

use crate::model::SpectrumView;
use std::sync::Arc;

use crate::spectrogram::baseline::IngestBaseline;
use crate::spectrogram::capture_progress::CaptureProgress;
use crate::spectrogram::model::SpectrogramSeries;

pub fn ensure_live_series(
    settings: &crate::spectrogram::settings::SpectrogramSettings,
    progress: &mut CaptureProgress,
    spectrum: &SpectrumView,
    device_serial: Option<&str>,
    energies_kev: &[f64],
) {
    if progress.live_series.is_some() {
        return;
    }
    let header = crate::spectrogram::storage::header_now(
        spectrum.a0,
        spectrum.a1,
        spectrum.a2,
        energies_kev.len() as u32,
        settings.capture_interval(),
        device_serial.map(str::to_string),
        energies_kev.to_vec(),
    );
    progress.live_series = Some(Arc::new(SpectrogramSeries::new(
        header,
        energies_kev.to_vec(),
    )));
    progress.mark_dirty();
}

pub fn store_baseline_capture(
    progress: &mut CaptureProgress,
    sequence: u64,
    cumulative: Vec<u32>,
    device_duration_secs: f64,
) {
    debug!(sequence, "spectrogram baseline sample stored");
    progress.skip_next_sample = false;
    progress.reconnect_baseline_pending = false;
    progress.baseline = Some(IngestBaseline::new(cumulative, device_duration_secs));
    progress.last_ingested_sequence = sequence;
    progress.last_ingest_at = Some(Instant::now());
    progress.status = "Synced. Adding rows on each spectrum refresh.".into();
    progress.mark_dirty();
}
