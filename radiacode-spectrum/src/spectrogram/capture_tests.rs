use std::sync::{Arc, Mutex};

use crossbeam_channel::unbounded;

use crate::events::test_fixtures::{device_info, isolated_data_dir, spectrum_view};
use crate::spectrogram::capture::{SpectrogramCapture, spawn_capture_router};
use crate::spectrogram::capture_lock::{lock_capture, lock_progress};
use crate::worker::WorkerEvent;

struct Router {
    capture: Arc<Mutex<SpectrogramCapture>>,
    worker_events: crossbeam_channel::Sender<WorkerEvent>,
    ui_events: crossbeam_channel::Receiver<WorkerEvent>,
    _data_dir: tempfile::TempDir,
}

fn router() -> Router {
    let data_dir = isolated_data_dir();
    let capture = Arc::new(Mutex::new(SpectrogramCapture::new()));
    let (worker_tx, worker_rx) = unbounded();
    let (ui_tx, ui_rx) = unbounded();
    spawn_capture_router(worker_rx, ui_tx, Arc::clone(&capture));
    Router {
        capture,
        worker_events: worker_tx,
        ui_events: ui_rx,
        _data_dir: data_dir,
    }
}

impl Router {
    fn round_trip(&self, event: WorkerEvent) -> WorkerEvent {
        self.worker_events.send(event).expect("router alive");
        self.ui_events
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("event forwarded")
    }

    fn serial(&self) -> Option<String> {
        lock_capture(&self.capture).device_serial.clone()
    }

    fn last_sequence(&self) -> u64 {
        let capture = lock_capture(&self.capture);
        let progress = lock_progress(&capture.progress);
        progress.last_ingested_sequence
    }
}

#[test]
fn router_forwards_events_and_tracks_session_serial() {
    let router = router();
    let forwarded = router.round_trip(WorkerEvent::Connected(device_info()));
    assert!(matches!(forwarded, WorkerEvent::Connected(_)));
    assert_eq!(router.serial().as_deref(), Some("RC-103-000001"));
    router.round_trip(WorkerEvent::Disconnected);
    assert!(router.serial().is_none());
}

#[test]
fn router_ingests_spectra_only_while_capturing() {
    let router = router();
    router.round_trip(WorkerEvent::Spectrum(spectrum_view()));
    assert_eq!(router.last_sequence(), 0);
    router.round_trip(WorkerEvent::Connected(device_info()));
    router.round_trip(WorkerEvent::Spectrum(spectrum_view()));
    assert_eq!(router.last_sequence(), 1);
    router.round_trip(WorkerEvent::Reconnecting);
    router.round_trip(WorkerEvent::Spectrum(spectrum_view()));
    assert_eq!(router.last_sequence(), 2);
    router.round_trip(WorkerEvent::Busy(true));
    assert_eq!(router.last_sequence(), 2);
}
