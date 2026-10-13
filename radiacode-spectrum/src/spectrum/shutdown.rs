use std::time::{Duration, Instant};

use egui::{Context, ViewportCommand};
use tracing::{info, warn};

pub(super) const SHUTDOWN_DISCONNECT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseState {
    None,
    AwaitingDisconnect,
    Closing,
}

pub enum CloseAction {
    None,
    DisconnectDevice,
    CompleteClose,
}

pub struct ShutdownSequence {
    pub state: CloseState,
    pub started: Option<Instant>,
    pub after_disconnect: bool,
}

impl ShutdownSequence {
    pub fn new() -> Self {
        Self {
            state: CloseState::None,
            started: None,
            after_disconnect: false,
        }
    }

    pub fn active(&self) -> bool {
        self.state != CloseState::None
    }

    pub fn closing(&self) -> bool {
        self.state == CloseState::Closing
    }

    pub fn on_close_request(
        &mut self,
        close_requested: bool,
        device_link_active: bool,
    ) -> CloseAction {
        if !close_requested {
            return CloseAction::None;
        }
        if self.state == CloseState::Closing {
            return CloseAction::None;
        }
        if self.state == CloseState::AwaitingDisconnect {
            warn!("application close requested again during shutdown; closing now");
            self.complete_close();
            return CloseAction::CompleteClose;
        }
        if device_link_active {
            info!("application close requested; disconnecting device");
            self.state = CloseState::AwaitingDisconnect;
            self.started = Some(Instant::now());
            return CloseAction::DisconnectDevice;
        }
        self.complete_close();
        CloseAction::CompleteClose
    }

    pub fn advance_close(&mut self) -> CloseAction {
        if self.after_disconnect {
            self.after_disconnect = false;
            self.complete_close();
            return CloseAction::CompleteClose;
        }
        if self.state != CloseState::AwaitingDisconnect {
            return CloseAction::None;
        }
        let Some(started) = self.started else {
            self.complete_close();
            return CloseAction::CompleteClose;
        };
        if started.elapsed() >= SHUTDOWN_DISCONNECT_TIMEOUT {
            warn!("device disconnect timed out during shutdown; closing anyway");
            self.complete_close();
            return CloseAction::CompleteClose;
        }
        CloseAction::None
    }

    pub fn on_disconnected(&mut self) {
        if self.state == CloseState::AwaitingDisconnect {
            self.after_disconnect = true;
        }
    }

    fn complete_close(&mut self) {
        if self.state == CloseState::Closing {
            return;
        }
        info!("application shutdown complete");
        self.state = CloseState::Closing;
        self.started = None;
        self.after_disconnect = false;
    }

    pub fn send_close_viewport(&self, ctx: &Context) {
        if self.state == CloseState::Closing {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
}

#[cfg(test)]
#[path = "shutdown_tests.rs"]
mod tests;
