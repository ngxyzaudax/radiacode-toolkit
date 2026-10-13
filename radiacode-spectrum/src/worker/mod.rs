use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, unbounded};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tracing::info;

use crate::spectrogram::capture::{SpectrogramCapture, spawn_capture_router};

mod batch;
mod device_info;
mod epoch;
mod link;
mod messages;
mod ops_config;
mod ops_connect;
mod ops_monitor;
mod ops_reconnect;
mod ops_scan;
mod ops_spectrum;
mod run_loop;
mod session;
mod timers;

#[cfg(test)]
mod ops_config_tests;
#[cfg(test)]
mod ops_connect_tests;
#[cfg(test)]
mod ops_monitor_tests;
#[cfg(test)]
mod ops_spectrum_tests;
#[cfg(test)]
mod run_loop_tests;
#[cfg(test)]
pub(crate) mod test_fixtures;

pub use messages::{WorkerCommand, WorkerEvent};

use run_loop::{WorkerContext, worker_loop};
use session::WorkerSession;

pub struct WorkerHandle {
    pub commands: UnboundedSender<WorkerCommand>,
    pub events: Receiver<WorkerEvent>,
    session_epoch: Arc<AtomicU64>,
}

impl WorkerHandle {
    pub fn end_session(&self) {
        epoch::end_session(&self.session_epoch);
    }
}

pub fn spawn_worker(capture: Arc<Mutex<SpectrogramCapture>>) -> WorkerHandle {
    let (commands, command_rx) = unbounded_channel();
    let (worker_event_tx, worker_event_rx) = unbounded();
    let (ui_event_tx, ui_event_rx) = unbounded();
    spawn_capture_router(worker_event_rx, ui_event_tx, capture);
    let session_epoch = Arc::new(AtomicU64::new(0));
    let ctx = WorkerContext {
        events: worker_event_tx,
        session_epoch: Arc::clone(&session_epoch),
        session: WorkerSession::new(),
    };
    info!("spawning device worker thread");
    std::thread::spawn(move || run_worker_runtime(command_rx, ctx));
    WorkerHandle {
        commands,
        events: ui_event_rx,
        session_epoch,
    }
}

fn run_worker_runtime(
    command_rx: tokio::sync::mpsc::UnboundedReceiver<WorkerCommand>,
    ctx: WorkerContext,
) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    runtime.block_on(worker_loop(command_rx, ctx));
}
