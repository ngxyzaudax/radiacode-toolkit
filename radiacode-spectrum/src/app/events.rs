use super::SpectrumApp;
use crate::events::{EventOutcome, EventRouter};
use crate::worker::{WorkerCommand, WorkerEvent};

impl SpectrumApp {
    pub(super) fn poll_events(&mut self) {
        while let Ok(event) = self.worker.events.try_recv() {
            self.handle_event(event);
        }
    }

    fn handle_event(&mut self, event: WorkerEvent) {
        let disconnected = matches!(&event, WorkerEvent::Disconnected);
        let (outcome, pending_commands) = self.route_event(event);
        self.dispatch_pending(pending_commands);
        if disconnected {
            self.session_blocked = false;
            self.shutdown.on_disconnected();
        }
        self.apply_event_outcome(outcome);
    }

    fn route_event(&mut self, event: WorkerEvent) -> (EventOutcome, Vec<WorkerCommand>) {
        let mut pending_commands = Vec::new();
        let pc_alarm_repeat = self.settings.app.pc_alarm_repeat;
        let mut router = EventRouter {
            state: &mut self.state,
            settings: &mut self.settings,
            spectrogram: &mut self.spectrogram,
            usb_prompt: &mut self.usb_access_prompt,
            session_blocked: self.session_blocked,
            active_tab: self.tabs.active,
            pending_tab_after_save: &mut self.tabs.pending_after_save,
            pending_tab: &mut self.tabs.pending,
            monitor_leave_open: &mut self.tabs.monitor_leave_open,
            last_alarm_level: &mut self.last_alarm_level,
            pc_alarm_repeat,
        };
        let outcome = router.dispatch(event, &mut |command| pending_commands.push(command));
        (outcome, pending_commands)
    }

    fn apply_event_outcome(&mut self, outcome: EventOutcome) {
        if let Some(tab) = outcome.switch_tab {
            self.tabs.active = tab;
        }
        if outcome.sync_capture_interval {
            self.sync_capture_interval();
        }
        if outcome.sync_monitor_poll_interval {
            self.sync_monitor_poll_interval();
        }
        if let Some(info) = outcome.remember_device {
            self.remember_connected_device(&info);
        }
        if outcome.fetch_spectrum {
            self.schedule_spectrum();
        }
        if outcome.scan_finished && !self.shutting_down() {
            self.maybe_auto_connect();
        }
    }
}
