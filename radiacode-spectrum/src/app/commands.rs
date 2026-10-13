use tracing::{debug, warn};

use super::SpectrumApp;
use crate::worker::WorkerCommand;

impl SpectrumApp {
    pub(super) fn send(&mut self, command: WorkerCommand) {
        debug!(?command, "sending worker command");
        if self.worker.commands.send(command).is_err() {
            warn!("device worker stopped");
            self.state.status = "Device worker stopped.".into();
        }
    }

    pub(super) fn schedule_spectrum(&mut self) {
        if self.state.try_schedule_spectrum() {
            self.send(WorkerCommand::FetchSpectrum);
        }
    }

    pub(super) fn schedule_monitor(&mut self) {
        if self.state.try_schedule_monitor() {
            self.send(WorkerCommand::FetchMonitor);
        }
    }

    pub(super) fn sync_capture_interval(&mut self) {
        self.send(WorkerCommand::SetCaptureInterval(
            self.spectrogram.settings.capture_interval_secs,
        ));
    }

    pub(super) fn sync_monitor_poll_interval(&mut self) {
        self.send(WorkerCommand::SetMonitorPollInterval(
            self.settings.app.monitor_poll_secs,
        ));
    }

    pub(super) fn dispatch_pending(&mut self, commands: Vec<WorkerCommand>) {
        for command in commands {
            match command {
                WorkerCommand::FetchMonitor => self.schedule_monitor(),
                other => self.send(other),
            }
        }
    }
}
