use super::SpectrumApp;
use crate::monitor::MonitorLeaveChoice;
use crate::settings::SettingsAction;
use crate::view_tab::ViewTab;
use crate::worker::WorkerCommand;

const INTERVAL_EPSILON: f64 = 1e-9;

pub(super) fn interval_changed(previous: f64, current: f64) -> bool {
    (previous - current).abs() > INTERVAL_EPSILON
}

impl SpectrumApp {
    pub(super) fn handle_monitor_leave_choice(&mut self, choice: MonitorLeaveChoice) {
        self.tabs.monitor_leave_open = false;
        match choice {
            MonitorLeaveChoice::Save => {
                self.tabs.pending_after_save = self.tabs.pending.take();
                self.handle_settings_action(SettingsAction::SaveDevice);
            }
            MonitorLeaveChoice::Discard => {
                self.discard_draft();
                if let Some(tab) = self.tabs.pending.take() {
                    self.tabs.active = tab;
                }
            }
            MonitorLeaveChoice::Stay => self.tabs.pending = None,
        }
    }

    pub(super) fn handle_settings_action(&mut self, action: SettingsAction) {
        match action {
            SettingsAction::LoadDevice if self.settings.draft_dirty() => {
                self.settings.request_load();
            }
            SettingsAction::LoadDevice | SettingsAction::ConfirmLoad => self.start_device_load(),
            SettingsAction::CancelLoad => {}
            SettingsAction::SaveDevice => self.save_draft_to_device(),
            SettingsAction::DiscardChanges => self.discard_draft(),
            SettingsAction::SyncClock => {
                self.settings.status = "Syncing clock…".into();
                self.send(WorkerCommand::SyncDeviceClock);
            }
            SettingsAction::AppChanged => {
                self.settings.persist_app();
                self.sync_monitor_poll_interval();
            }
            SettingsAction::SpectrogramChanged => self.apply_spectrogram_settings(),
        }
    }

    fn save_draft_to_device(&mut self) {
        let Some(draft) = self.settings.draft else {
            return;
        };
        self.settings.begin_save();
        self.send(WorkerCommand::ApplyDeviceConfig(draft));
    }

    fn discard_draft(&mut self) {
        self.settings.discard();
        self.sync_draft_alarm_limits();
    }

    fn apply_spectrogram_settings(&mut self) {
        let previous_interval = self.spectrogram.settings.capture_interval_secs;
        let previous_dir = self.spectrogram.settings.recordings_dir.clone();
        self.settings.persist_spectrogram();
        self.settings
            .apply_spectrogram_to(&mut self.spectrogram.settings);
        self.spectrogram.on_settings_changed();
        if self.spectrogram.settings.recordings_dir != previous_dir {
            self.compare
                .refresh_library(&self.spectrogram.settings.recordings_dir);
        }
        if interval_changed(
            previous_interval,
            self.spectrogram.settings.capture_interval_secs,
        ) {
            self.sync_capture_interval();
        }
    }

    pub(super) fn start_device_load(&mut self) {
        if !self.is_connected() || self.settings.device_busy() {
            return;
        }
        self.settings.begin_load();
        self.send(WorkerCommand::FetchDeviceConfig);
    }

    pub(super) fn maybe_device_config_auto_load(&mut self) {
        let config_tab = matches!(self.tabs.active, ViewTab::Settings | ViewTab::Monitor);
        if config_tab && self.is_connected() && self.settings.needs_auto_load() {
            self.start_device_load();
        }
    }

    pub(super) fn sync_draft_alarm_limits(&mut self) {
        if let Some(draft) = self.settings.draft.as_ref() {
            self.state.apply_alarm_limits(draft.alarms);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::interval_changed;

    #[test]
    fn interval_change_detection_ignores_noise() {
        assert!(!interval_changed(5.0, 5.0));
        assert!(!interval_changed(5.0, 5.0 + 1e-12));
        assert!(interval_changed(5.0, 6.0));
    }
}
