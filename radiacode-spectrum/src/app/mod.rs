use std::sync::{Arc, Mutex};

use crate::catalogue::CatalogueState;
use crate::compare::CompareState;
use crate::events::AppState;
use crate::model::ConnectionState;
use crate::monitor::AlarmLevel;
use crate::settings::SettingsState;
use crate::spectrogram::SpectrogramState;
use crate::spectrogram::capture::SpectrogramCapture;
use crate::spectrum::{ShutdownSequence, SpectrumViewState, StartupChrome, TabNavigation};
use crate::usb_access::UsbAccessPrompt;
use crate::worker::{WorkerHandle, spawn_worker};

mod actions_device;
mod actions_settings;
mod actions_spectrogram;
mod commands;
mod connection;
mod content;
mod content_live;
mod events;
mod frame;
mod lifecycle;
mod tab_entry;
mod usb_access;

#[cfg(test)]
mod connection_tests;
#[cfg(test)]
mod events_tests;
#[cfg(test)]
mod spectrogram_actions_tests;
#[cfg(test)]
mod test_harness;

pub struct SpectrumApp {
    worker: WorkerHandle,
    state: AppState,
    settings: SettingsState,
    spectrogram: SpectrogramState,
    compare: CompareState,
    catalogue: CatalogueState,
    tabs: TabNavigation,
    view: SpectrumViewState,
    chrome: StartupChrome,
    shutdown: ShutdownSequence,
    session_blocked: bool,
    usb_access_prompt: Option<UsbAccessPrompt>,
    last_alarm_level: AlarmLevel,
    auto_connect_attempted: bool,
}

impl SpectrumApp {
    pub fn new() -> Self {
        let capture = Arc::new(Mutex::new(SpectrogramCapture::new()));
        Self::with_worker(spawn_worker(Arc::clone(&capture)), capture)
    }

    pub(crate) fn with_worker(
        worker: WorkerHandle,
        capture: Arc<Mutex<SpectrogramCapture>>,
    ) -> Self {
        let settings = SettingsState::new();
        let spectrogram = initial_spectrogram(&settings, capture);
        let mut compare = CompareState::new();
        compare.refresh_library(&settings.spectrogram.recordings_dir);
        Self {
            worker,
            state: AppState::new(),
            settings,
            spectrogram,
            compare,
            catalogue: CatalogueState::new(),
            tabs: TabNavigation::new(),
            view: SpectrumViewState::new(),
            chrome: StartupChrome::new(),
            shutdown: ShutdownSequence::new(),
            session_blocked: false,
            usb_access_prompt: None,
            last_alarm_level: AlarmLevel::Normal,
            auto_connect_attempted: false,
        }
    }

    fn device_link_active(&self) -> bool {
        matches!(
            self.state.connection,
            ConnectionState::Connected | ConnectionState::Connecting
        )
    }

    fn is_connected(&self) -> bool {
        self.state.connection == ConnectionState::Connected
    }

    fn shutting_down(&self) -> bool {
        self.shutdown.active()
    }
}

fn initial_spectrogram(
    settings: &SettingsState,
    capture: Arc<Mutex<SpectrogramCapture>>,
) -> SpectrogramState {
    let mut spectrogram = SpectrogramState::new(capture);
    spectrogram.settings = settings.spectrogram.clone();
    spectrogram.on_settings_changed();
    spectrogram.refresh_history();
    spectrogram
}
