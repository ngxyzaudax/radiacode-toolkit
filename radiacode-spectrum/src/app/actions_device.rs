use super::SpectrumApp;
use crate::device::DeviceAction;
use crate::monitor::MonitorViewAction;
use crate::peak_overlay::SpectrumPlotAction;
use crate::ui_toolbar::SpectrumToolbarAction;
use crate::view_tab::ViewTab;
use crate::worker::WorkerCommand;

impl SpectrumApp {
    pub(super) fn handle_device_action(&mut self, action: DeviceAction) {
        match action {
            DeviceAction::Scan => self.start_scan(),
            DeviceAction::Connect(endpoint) => self.start_connect(endpoint),
            DeviceAction::Disconnect => self.disconnect_device(),
        }
    }

    pub(super) fn handle_spectrum_toolbar_action(&mut self, action: SpectrumToolbarAction) {
        if matches!(action, SpectrumToolbarAction::Reset) {
            self.state.spectrum_fetch_pending = true;
            self.send(WorkerCommand::ResetSpectrum);
        }
    }

    pub(super) fn handle_monitor_view_action(&mut self, action: MonitorViewAction) {
        match action {
            MonitorViewAction::ResetDose => self.send(WorkerCommand::ResetDose),
            MonitorViewAction::Settings(settings_action) => {
                self.handle_settings_action(settings_action);
            }
        }
    }

    pub(super) fn handle_spectrum_plot_action(&mut self, action: SpectrumPlotAction) {
        match action {
            SpectrumPlotAction::OpenCatalogue(id) => self.catalogue.reveal(id),
            SpectrumPlotAction::OpenCatalogueChain(head) => {
                self.catalogue.select_chain_by_head(head);
            }
        }
        self.tabs.active = ViewTab::Catalogue;
    }

    pub(super) fn try_switch_tab(&mut self, tab: ViewTab) -> bool {
        self.tabs.try_switch(tab, self.settings.draft_dirty())
    }
}
