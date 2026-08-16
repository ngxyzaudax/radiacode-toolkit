use crate::events::AppState;
use crate::model::ConnectionState;
use crate::monitor::AlarmLevel;
use crate::pc_alarm::maybe_beep_alarm;
use crate::settings::{SettingsDeviceOp, SettingsState};
use crate::spectrogram::SpectrogramState;
use crate::usb_access::{UsbAccessPrompt, usb_access_required};
use crate::view_tab::ViewTab;
use crate::worker::WorkerCommand;
use crate::worker::{WorkerCommand as Wc, WorkerEvent};

pub struct EventRouter<'a> {
    pub state: &'a mut AppState,
    pub settings: &'a mut SettingsState,
    pub spectrogram: &'a mut SpectrogramState,
    pub usb_prompt: &'a mut Option<UsbAccessPrompt>,
    pub session_blocked: bool,
    pub active_tab: ViewTab,
    pub pending_tab_after_save: &'a mut Option<ViewTab>,
    pub pending_tab: &'a mut Option<ViewTab>,
    pub monitor_leave_open: &'a mut bool,
    pub last_alarm_level: &'a mut AlarmLevel,
    pub pc_alarm_repeat: bool,
}

#[derive(Default)]
pub struct EventOutcome {
    pub fetch_spectrum: bool,
    pub scan_finished: bool,
    pub monitor_sample: bool,
    pub switch_tab: Option<ViewTab>,
    pub sync_capture_interval: bool,
    pub sync_monitor_poll_interval: bool,
    pub remember_device: Option<crate::model::DeviceInfo>,
}

impl<'a> EventRouter<'a> {
    pub fn dispatch(
        &mut self,
        event: WorkerEvent,
        send: &mut impl FnMut(WorkerCommand),
    ) -> EventOutcome {
        let accept_session = !self.session_blocked;
        let mut outcome = EventOutcome::default();

        if let WorkerEvent::UsbPermissionRequired { endpoint } = &event
            && accept_session
            && let Some(status) = usb_access_required(endpoint)
        {
            *self.usb_prompt = Some(UsbAccessPrompt::new(endpoint.clone(), status));
        }

        if let WorkerEvent::DeviceConfig(config) = &event
            && accept_session
        {
            let saving = self.settings.device_op == SettingsDeviceOp::Saving;
            match self.settings.device_op {
                SettingsDeviceOp::Loading => self.settings.on_loaded(*config),
                SettingsDeviceOp::Saving => self.settings.on_saved(*config),
                SettingsDeviceOp::Idle => {}
            }
            self.state.apply_alarm_limits(config.alarms);
            if saving && let Some(tab) = self.pending_tab_after_save.take() {
                outcome.switch_tab = Some(tab);
            }
        }

        if let WorkerEvent::Error(message) = &event {
            if self.settings.device_op == SettingsDeviceOp::Loading
                || self.settings.device_op == SettingsDeviceOp::Saving
            {
                self.settings.on_device_op_failed(message.clone());
                if self.pending_tab_after_save.is_some() {
                    *self.pending_tab = self.pending_tab_after_save.take();
                    *self.monitor_leave_open = true;
                }
            } else if self.settings.status == "Saved to device" {
                self.settings.status = message.clone();
            }
        }

        outcome.fetch_spectrum = matches!(&event, WorkerEvent::Connected(_)) && accept_session;
        let initial_connect = matches!(&event, WorkerEvent::Connected(_))
            && self.state.connection == ConnectionState::Connecting;
        outcome.scan_finished = matches!(&event, WorkerEvent::ScanFinished(_));
        outcome.monitor_sample = matches!(&event, WorkerEvent::MonitorSample(_));

        match &event {
            WorkerEvent::Reconnecting if accept_session => {
                self.spectrogram.on_reconnect();
            }
            WorkerEvent::Connected(info) if accept_session && initial_connect => {
                outcome.sync_capture_interval = true;
                outcome.sync_monitor_poll_interval = true;
                self.spectrogram.sync_from_capture();
                outcome.remember_device = Some(info.clone());
                if self.active_tab == ViewTab::Device {
                    outcome.switch_tab = Some(ViewTab::Monitor);
                }
                if matches!(self.active_tab, ViewTab::Settings | ViewTab::Monitor)
                    && !self.settings.draft_dirty()
                {
                    send(Wc::FetchDeviceConfig);
                }
            }
            WorkerEvent::Disconnected => {
                self.spectrogram.on_disconnect();
                self.settings.on_disconnect();
                *self.last_alarm_level = AlarmLevel::Normal;
            }
            _ => {}
        }

        if !matches!(&event, WorkerEvent::DeviceConfig(_))
            && let Some(command) = self.state.apply_event(event, accept_session)
        {
            match command {
                Wc::FetchMonitor => {
                    if self.state.try_schedule_monitor() {
                        send(Wc::FetchMonitor);
                    }
                }
                other => send(other),
            }
        }

        if outcome.monitor_sample && accept_session {
            self.check_pc_alarm();
        }

        outcome
    }

    fn check_pc_alarm(&mut self) {
        let dose = self.state.monitor.dose_alarm_level();
        let count = self.state.monitor.count_alarm_level();
        let accum = self.state.dosimeter.dose_alarm_level();
        let current = dose.max(count).max(accum);
        let rising = current > *self.last_alarm_level && current > AlarmLevel::Normal;
        *self.last_alarm_level = current;
        if rising {
            maybe_beep_alarm(self.pc_alarm_repeat);
        }
    }
}
