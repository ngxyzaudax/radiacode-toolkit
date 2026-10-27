use std::time::Duration;

use crossbeam_channel::Sender;
use radiacode_lib::{Error, RadiaCode, Spectrum};
use tracing::{debug, info, warn};

use super::epoch::SessionEpoch;
use super::messages::WorkerEvent;
use super::ops_reconnect::handle_device_error;
use super::session::WorkerSession;
use crate::model::SpectrumView;

const TRANSIENT_RETRIES: usize = 2;
const TRANSIENT_RETRY_DELAY: Duration = Duration::from_millis(250);

pub async fn handle_spectrum(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    epoch: &SessionEpoch,
) {
    let Some(mut device) = session.device.take() else {
        warn!("spectrum fetch skipped: no active device");
        return;
    };
    session.device = match fetch_spectrum_with_retries(&mut device).await {
        Ok(spectrum) => {
            publish_spectrum(events, spectrum, epoch);
            Some(device)
        }
        Err(error) => handle_device_error(events, session, device, error, epoch).await,
    };
}

fn publish_spectrum(events: &Sender<WorkerEvent>, spectrum: Spectrum, epoch: &SessionEpoch) {
    if !epoch.active() {
        return;
    }
    debug!(
        channels = spectrum.counts.len(),
        duration_secs = spectrum.duration.as_secs(),
        "spectrum fetched"
    );
    let _ = events.send(WorkerEvent::Spectrum(SpectrumView::from_spectrum(spectrum)));
}

pub async fn handle_dose_reset(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    epoch: &SessionEpoch,
) {
    let Some(mut device) = session.device.take() else {
        warn!("dose reset skipped: no active device");
        return;
    };
    info!("resetting accumulated dose");
    session.device = match device.dose_reset().await {
        Ok(()) => {
            if epoch.active() {
                let _ = events.send(WorkerEvent::DoseResetComplete);
            }
            Some(device)
        }
        Err(error) => handle_device_error(events, session, device, error, epoch).await,
    };
}

pub async fn handle_reset(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    epoch: &SessionEpoch,
) {
    let Some(mut device) = session.device.take() else {
        warn!("spectrum reset skipped: no active device");
        return;
    };
    info!("resetting spectrum");
    match device.spectrum_reset().await {
        Ok(()) => {
            session.device = Some(device);
            handle_spectrum(events, session, epoch).await;
        }
        Err(error) => {
            session.device = handle_device_error(events, session, device, error, epoch).await;
        }
    }
}

pub async fn fetch_spectrum_with_retries(
    device: &mut RadiaCode,
) -> radiacode_lib::Result<Spectrum> {
    let mut last_error: Option<Error> = None;
    for attempt in 0..=TRANSIENT_RETRIES {
        if attempt > 0 {
            debug!(attempt, "retrying spectrum fetch after transient error");
            tokio::time::sleep(TRANSIENT_RETRY_DELAY).await;
        }
        match device.spectrum().await {
            Ok(spectrum) => return Ok(spectrum),
            Err(error) if error.is_transient() => {
                warn!(attempt, %error, "transient spectrum error");
                last_error = Some(error);
            }
            Err(error) => return Err(error),
        }
    }
    Err(last_error.unwrap_or(Error::Timeout))
}
