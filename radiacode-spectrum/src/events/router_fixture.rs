use std::sync::{Arc, Mutex};

use super::test_fixtures::isolated_data_dir;
use super::{AppState, EventOutcome, EventRouter};
use crate::monitor::AlarmLevel;
use crate::settings::SettingsState;
use crate::spectrogram::SpectrogramState;
use crate::spectrogram::capture::SpectrogramCapture;
use crate::usb_access::UsbAccessPrompt;
use crate::view_tab::ViewTab;
use crate::worker::{WorkerCommand, WorkerEvent};

pub struct Fixture {
    pub state: AppState,
    pub settings: SettingsState,
    pub spectrogram: SpectrogramState,
    pub usb_prompt: Option<UsbAccessPrompt>,
    pub pending_after_save: Option<ViewTab>,
    pub pending: Option<ViewTab>,
    pub monitor_leave_open: bool,
    pub last_alarm_level: AlarmLevel,
    pub active_tab: ViewTab,
    pub session_blocked: bool,
    _dir: tempfile::TempDir,
}

impl Fixture {
    pub fn new(active_tab: ViewTab) -> Self {
        let dir = isolated_data_dir();
        let capture = Arc::new(Mutex::new(SpectrogramCapture::new()));
        Self {
            state: AppState::new(),
            settings: SettingsState::new(),
            spectrogram: SpectrogramState::new(capture),
            usb_prompt: None,
            pending_after_save: None,
            pending: None,
            monitor_leave_open: false,
            last_alarm_level: AlarmLevel::Normal,
            active_tab,
            session_blocked: false,
            _dir: dir,
        }
    }

    pub fn dispatch(&mut self, event: WorkerEvent) -> (EventOutcome, Vec<WorkerCommand>) {
        let mut sent = Vec::new();
        let mut router = EventRouter {
            state: &mut self.state,
            settings: &mut self.settings,
            spectrogram: &mut self.spectrogram,
            usb_prompt: &mut self.usb_prompt,
            session_blocked: self.session_blocked,
            active_tab: self.active_tab,
            pending_tab_after_save: &mut self.pending_after_save,
            pending_tab: &mut self.pending,
            monitor_leave_open: &mut self.monitor_leave_open,
            last_alarm_level: &mut self.last_alarm_level,
            pc_alarm_repeat: false,
        };
        let outcome = router.dispatch(event, &mut |command| sent.push(command));
        (outcome, sent)
    }
}
