use super::SpectrumApp;
use crate::view_tab::ViewTab;

impl SpectrumApp {
    pub(super) fn on_tab_changed(&mut self, previous: ViewTab) {
        let active = self.tabs.active;
        if active == previous {
            return;
        }
        match active {
            ViewTab::Spectrogram => self.enter_spectrogram_tab(),
            ViewTab::Compare => self.enter_compare_tab(),
            ViewTab::Monitor => self.enter_monitor_tab(),
            ViewTab::Settings => self.enter_device_config_tab(),
            ViewTab::Device
            | ViewTab::Spectrum
            | ViewTab::PeakId
            | ViewTab::Sources
            | ViewTab::About => {}
        }
    }

    fn enter_device_config_tab(&mut self) {
        if !self.is_connected() {
            return;
        }
        if self.settings.draft_dirty() {
            self.settings.show_load_confirm = true;
            return;
        }
        self.start_device_load();
    }

    fn enter_monitor_tab(&mut self) {
        if !self.is_connected() {
            return;
        }
        if self.settings.draft_dirty() {
            self.sync_draft_alarm_limits();
            return;
        }
        self.start_device_load();
    }
}
