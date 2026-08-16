use std::sync::{Arc, Mutex};
use std::time::Duration;

use eframe::App;
use egui::{CentralPanel, Context, Ui, ViewportCommand};
use radiacode_core::{DeviceEndpoint, TransportKind, merge_discovered, resolve_usb_endpoint};
use tracing::{debug, info, warn};

use crate::about::draw_about_view;
use crate::analysis::{AnalysisState, draw_analysis_view};
use crate::catalogue::{CatalogueState, draw_catalogue_view};
use crate::device::{DeviceAction, DeviceViewProps, draw_device_view};
use crate::events::{AppState, EventRouter};
use crate::layout::page_scroll;
use crate::model::{ConnectionState, DeviceInfo};
use crate::monitor::{
    AlarmLevel, MonitorLeaveChoice, MonitorViewAction, MonitorViewProps,
    draw_monitor_leave_confirm, draw_monitor_view,
};
use crate::peak_overlay::SpectrumPlotAction;
use crate::plot_style::histogram_style;
use crate::settings::{SettingsAction, SettingsState, draw_settings_view};
use crate::spectrogram::SpectrogramState;
use crate::spectrogram::capture::SpectrogramCapture;
use crate::spectrogram::controls_action::SpectrogramControlsAction;
use crate::spectrogram::ui_view::draw_spectrogram_view;
use crate::spectrum::{
    CloseAction, ShutdownSequence, SpectrumViewState, StartupChrome, TabNavigation,
};
use crate::tabs::draw_tab_bar;
use crate::theme;
use crate::ui_chrome::{tab_uses_page_inset, tab_uses_plot_pad, with_page_inset, with_plot_pad};
use crate::ui_disconnected::{draw_disconnected_view, shows_tab_content, tab_works_offline};
use crate::ui_plot::draw_spectrum_plot;
use crate::ui_toolbar::{SpectrumToolbarAction, SpectrumToolbarProps, draw_spectrum_toolbar};
use crate::usb_access::{
    UsbAccessAction, UsbAccessOutcome, UsbAccessPrompt, draw_usb_access_dialog, usb_access_required,
};
use crate::view_tab::ViewTab;
use crate::worker::{WorkerCommand, WorkerEvent, WorkerHandle, spawn_worker};

pub struct SpectrumApp {
    worker: WorkerHandle,
    state: AppState,
    settings: SettingsState,
    spectrogram: SpectrogramState,
    analysis: AnalysisState,
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
        let settings = SettingsState::new();
        let mut spectrogram = SpectrogramState::new(Arc::clone(&capture));
        spectrogram.settings = settings.spectrogram.clone();
        spectrogram.on_settings_changed();
        spectrogram.refresh_history();
        let mut analysis = AnalysisState::new();
        analysis.refresh_library(&settings.spectrogram.recordings_dir);
        Self {
            worker: spawn_worker(capture),
            state: AppState::new(),
            settings,
            spectrogram,
            analysis,
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

    fn shutting_down(&self) -> bool {
        self.shutdown.active()
    }

    fn handle_close_request(&mut self, ctx: &Context) {
        let close_requested = ctx.input(|input| input.viewport().close_requested());
        if close_requested {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
        match self
            .shutdown
            .on_close_request(close_requested, self.device_link_active())
        {
            CloseAction::None => {}
            CloseAction::DisconnectDevice => self.disconnect_device(),
            CloseAction::CompleteClose => self.shutdown.send_close_viewport(ctx),
        }
    }

    fn advance_close(&mut self, ctx: &Context) {
        if matches!(self.shutdown.advance_close(), CloseAction::CompleteClose) {
            self.shutdown.send_close_viewport(ctx);
        }
    }

    fn apply_ui_scale(&self, ctx: &Context) {
        let scale = self.settings.app.ui_scale;
        if (ctx.zoom_factor() - scale).abs() > f32::EPSILON {
            ctx.set_zoom_factor(scale);
        }
    }

    fn ensure_window_icon(&mut self, ctx: &Context) {
        self.chrome.ensure_window_icon(ctx);
    }

    fn ensure_startup_window_size(&mut self, ctx: &Context) {
        self.chrome.ensure_startup_window_size(ctx);
    }

    fn send(&mut self, command: WorkerCommand) {
        debug!(?command, "sending worker command");
        if self.worker.commands.send(command).is_err() {
            warn!("device worker stopped");
            self.state.status = "Device worker stopped.".into();
        }
    }

    fn schedule_spectrum(&mut self) {
        if self.state.try_schedule_spectrum() {
            self.send(WorkerCommand::FetchSpectrum);
        }
    }

    fn sync_capture_interval(&mut self) {
        self.send(WorkerCommand::SetCaptureInterval(
            self.spectrogram.settings.capture_interval_secs,
        ));
    }

    fn sync_monitor_poll_interval(&mut self) {
        self.send(WorkerCommand::SetMonitorPollInterval(
            self.settings.app.monitor_poll_secs,
        ));
    }

    fn schedule_monitor(&mut self) {
        if self.state.try_schedule_monitor() {
            self.send(WorkerCommand::FetchMonitor);
        }
    }

    fn endpoint_from_info(info: &DeviceInfo) -> DeviceEndpoint {
        match info.transport {
            TransportKind::Bluetooth => DeviceEndpoint::Bluetooth {
                address: info.address.clone(),
            },
            TransportKind::Usb => DeviceEndpoint::Usb {
                serial: info.address.clone(),
            },
        }
    }

    fn remember_connected_device(&mut self, info: &DeviceInfo) {
        if !self.settings.app.remember_device {
            return;
        }
        self.settings.app.last_endpoint = Some(Self::endpoint_from_info(info));
        self.settings.persist_app();
    }

    fn maybe_auto_connect(&mut self) {
        if self.shutting_down() {
            return;
        }
        if self.auto_connect_attempted {
            return;
        }
        if !self.settings.app.auto_connect {
            return;
        }
        if self.state.connection != ConnectionState::Disconnected {
            return;
        }
        let Some(endpoint) = self.settings.app.last_endpoint.clone() else {
            return;
        };
        let found = self
            .state
            .devices
            .iter()
            .any(|device| device.endpoint == endpoint);
        if !found {
            return;
        }
        if usb_access_required(&endpoint).is_some() {
            info!(?endpoint, "auto-connect skipped: usb access required");
            return;
        }
        self.auto_connect_attempted = true;
        info!(?endpoint, "auto-connecting to remembered device");
        self.start_connect(endpoint);
    }

    fn poll_events(&mut self) {
        while let Ok(event) = self.worker.events.try_recv() {
            let disconnected = matches!(&event, WorkerEvent::Disconnected);
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
            for command in pending_commands {
                match command {
                    WorkerCommand::FetchMonitor => self.schedule_monitor(),
                    other => self.send(other),
                }
            }
            if disconnected {
                self.session_blocked = false;
                self.shutdown.on_disconnected();
            }
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

    fn start_scan(&mut self) {
        info!("ui starting scan");
        self.state.scanning = true;
        self.state.status = "Scanning for RadiaCode devices…".into();
        self.send(WorkerCommand::Scan);
    }

    fn refresh_usb_devices(&mut self) {
        let bluetooth: Vec<_> = self
            .state
            .devices
            .iter()
            .filter(|device| device.endpoint.transport() == TransportKind::Bluetooth)
            .cloned()
            .collect();
        match radiacode_usb::scan_usb_devices() {
            Ok(usb) => {
                self.state.devices = merge_discovered(usb, bluetooth);
            }
            Err(error) => {
                warn!(%error, "usb rescan failed");
            }
        }
    }

    fn start_connect(&mut self, endpoint: DeviceEndpoint) {
        self.start_connect_internal(endpoint, false);
    }

    fn start_connect_internal(&mut self, endpoint: DeviceEndpoint, force_usb: bool) {
        let endpoint = resolve_usb_endpoint(&self.state.devices, &endpoint);
        if !force_usb && let Some(status) = usb_access_required(&endpoint) {
            info!(?endpoint, ?status, "usb access required before connect");
            self.session_blocked = false;
            self.state.connection = ConnectionState::Disconnected;
            self.state.connecting_endpoint = Some(endpoint.clone());
            self.state.status = "USB access required.".into();
            self.usb_access_prompt = Some(UsbAccessPrompt::new(endpoint, status));
            return;
        }
        let address = endpoint.address_label().to_string();
        info!(%address, ?endpoint, force_usb, "ui starting connect");
        self.session_blocked = false;
        self.state.connection = ConnectionState::Connecting;
        self.state.connecting_endpoint = Some(endpoint.clone());
        self.state.status = format!("Connecting to {address}…");
        let hint_rssi = self.hint_rssi_for_endpoint(&endpoint);
        self.send(WorkerCommand::Connect {
            endpoint,
            hint_rssi,
        });
    }

    fn hint_rssi_for_endpoint(&self, endpoint: &DeviceEndpoint) -> Option<i16> {
        self.state
            .devices
            .iter()
            .find(|device| device.endpoint == *endpoint)
            .and_then(|device| device.rssi)
    }

    fn disconnect_device(&mut self) {
        info!("ui disconnect requested");
        self.worker.end_session();
        self.session_blocked = true;
        self.state.clear_session();
        self.spectrogram.on_disconnect();
        self.settings.on_disconnect();
        self.last_alarm_level = AlarmLevel::Normal;
        self.send(WorkerCommand::Disconnect);
    }

    fn handle_device_action(&mut self, action: DeviceAction) {
        match action {
            DeviceAction::Scan => self.start_scan(),
            DeviceAction::Connect(endpoint) => self.start_connect(endpoint),
            DeviceAction::Disconnect => self.disconnect_device(),
        }
    }

    fn handle_spectrum_toolbar_action(&mut self, action: SpectrumToolbarAction) {
        if matches!(action, SpectrumToolbarAction::Reset) {
            self.state.spectrum_fetch_pending = true;
            self.send(WorkerCommand::ResetSpectrum);
        }
    }

    fn handle_monitor_view_action(&mut self, action: MonitorViewAction) {
        match action {
            MonitorViewAction::ResetDose => {
                self.send(WorkerCommand::ResetDose);
            }
            MonitorViewAction::Settings(settings_action) => {
                self.handle_settings_action(settings_action);
            }
        }
    }

    fn try_switch_tab(&mut self, tab: ViewTab) -> bool {
        self.tabs.try_switch(tab, self.settings.draft_dirty())
    }

    fn handle_monitor_leave_choice(&mut self, choice: MonitorLeaveChoice) {
        match choice {
            MonitorLeaveChoice::Save => {
                self.tabs.pending_after_save = self.tabs.pending.take();
                self.tabs.monitor_leave_open = false;
                self.handle_settings_action(SettingsAction::SaveDevice);
            }
            MonitorLeaveChoice::Discard => {
                self.settings.discard();
                self.sync_draft_alarm_limits();
                if let Some(tab) = self.tabs.pending.take() {
                    self.tabs.active = tab;
                }
                self.tabs.monitor_leave_open = false;
            }
            MonitorLeaveChoice::Stay => {
                self.tabs.pending = None;
                self.tabs.monitor_leave_open = false;
            }
        }
    }

    fn handle_settings_action(&mut self, action: SettingsAction) {
        match action {
            SettingsAction::LoadDevice => {
                if self.settings.draft_dirty() {
                    self.settings.request_load();
                } else {
                    self.start_device_load();
                }
            }
            SettingsAction::ConfirmLoad => {
                self.start_device_load();
            }
            SettingsAction::CancelLoad => {}
            SettingsAction::SaveDevice => {
                let Some(draft) = self.settings.draft else {
                    return;
                };
                self.settings.begin_save();
                self.send(WorkerCommand::ApplyDeviceConfig(draft));
            }
            SettingsAction::DiscardChanges => {
                self.settings.discard();
                self.sync_draft_alarm_limits();
            }
            SettingsAction::SyncClock => {
                self.settings.status = "Syncing clock…".into();
                self.send(WorkerCommand::SyncDeviceClock);
            }
            SettingsAction::AppChanged => {
                self.settings.persist_app();
                self.sync_monitor_poll_interval();
            }
            SettingsAction::SpectrogramChanged => {
                let previous_interval = self.spectrogram.settings.capture_interval_secs;
                let previous_dir = self.spectrogram.settings.recordings_dir.clone();
                self.settings.persist_spectrogram();
                self.settings
                    .apply_spectrogram_to(&mut self.spectrogram.settings);
                self.spectrogram.on_settings_changed();
                if self.spectrogram.settings.recordings_dir != previous_dir {
                    self.analysis
                        .refresh_library(&self.spectrogram.settings.recordings_dir);
                }
                if (previous_interval - self.spectrogram.settings.capture_interval_secs).abs()
                    > 1e-9
                {
                    self.sync_capture_interval();
                }
            }
        }
    }

    fn start_device_load(&mut self) {
        if self.state.connection != ConnectionState::Connected {
            return;
        }
        if self.settings.device_busy() {
            return;
        }
        self.settings.begin_load();
        self.send(WorkerCommand::FetchDeviceConfig);
    }

    fn maybe_device_config_auto_load(&mut self) {
        if !matches!(self.tabs.active, ViewTab::Settings | ViewTab::Monitor) {
            return;
        }
        if self.state.connection != ConnectionState::Connected {
            return;
        }
        if !self.settings.needs_auto_load() {
            return;
        }
        self.start_device_load();
    }

    fn enter_device_config_tab(&mut self) {
        if self.state.connection != ConnectionState::Connected {
            return;
        }
        if self.settings.draft_dirty() {
            self.settings.show_load_confirm = true;
            return;
        }
        self.start_device_load();
    }

    fn sync_draft_alarm_limits(&mut self) {
        let Some(draft) = self.settings.draft.as_ref() else {
            return;
        };
        self.state.apply_alarm_limits(draft.alarms);
    }

    fn enter_settings_tab(&mut self) {
        self.enter_device_config_tab();
    }

    fn enter_monitor_tab(&mut self) {
        if self.state.connection != ConnectionState::Connected {
            return;
        }
        if self.settings.draft_dirty() {
            self.sync_draft_alarm_limits();
            return;
        }
        self.start_device_load();
    }

    fn enter_catalogue_tab(&mut self) {
        self.catalogue.on_tab_enter();
    }

    fn handle_spectrum_plot_action(&mut self, action: SpectrumPlotAction) {
        match action {
            SpectrumPlotAction::OpenCatalogue(id) => {
                self.catalogue.reveal(id);
                self.tabs.active = ViewTab::Catalogue;
            }
            SpectrumPlotAction::OpenCatalogueChain(head) => {
                self.catalogue.select_chain_by_head(head);
                self.tabs.active = ViewTab::Catalogue;
            }
        }
    }

    fn handle_spectrogram_action(&mut self, action: SpectrogramControlsAction) {
        match action {
            SpectrogramControlsAction::StartRecording => {
                let serial = self
                    .state
                    .device_info
                    .as_ref()
                    .map(|info| info.serial.as_str());
                if let Err(message) = self
                    .spectrogram
                    .start_recording(self.state.spectrum.as_ref(), serial)
                {
                    self.spectrogram.status = message;
                }
            }
            SpectrogramControlsAction::StopRecording => {
                if let Err(message) = self.spectrogram.stop_recording() {
                    self.spectrogram.status = message;
                }
            }
            SpectrogramControlsAction::PauseCapture => {
                if let Err(message) = self.spectrogram.pause_capture() {
                    self.spectrogram.status = message;
                }
            }
            SpectrogramControlsAction::ResumeCapture => {
                if let Err(message) = self.spectrogram.resume_capture() {
                    self.spectrogram.status = message;
                }
            }
            SpectrogramControlsAction::ResumeRecording => {
                let serial = self
                    .state
                    .device_info
                    .as_ref()
                    .map(|info| info.serial.as_str());
                if let Err(message) = self
                    .spectrogram
                    .resume_recording(self.state.spectrum.as_ref(), serial)
                {
                    self.spectrogram.status = message;
                }
            }
            SpectrogramControlsAction::CloseLoaded => self.spectrogram.close_loaded(),
            SpectrogramControlsAction::Load(path) => self.spectrogram.request_load(path),
            SpectrogramControlsAction::SettingsChanged => {
                let previous_interval = self.settings.spectrogram.capture_interval_secs;
                self.settings.spectrogram = self.spectrogram.settings.clone();
                self.spectrogram.on_settings_changed();
                if (previous_interval - self.spectrogram.settings.capture_interval_secs).abs()
                    > 1e-9
                {
                    self.sync_capture_interval();
                }
            }
            SpectrogramControlsAction::LibraryChanged => {
                self.spectrogram.refresh_history();
                self.analysis
                    .refresh_library(&self.spectrogram.settings.recordings_dir);
            }
        }
    }

    fn enter_spectrogram_tab(&mut self) {
        info!("entered spectrogram tab");
        self.spectrogram.on_tab_enter();
    }

    fn enter_analysis_tab(&mut self) {
        self.analysis
            .refresh_library(&self.spectrogram.settings.recordings_dir);
    }

    fn poll_usb_access(&mut self) {
        let Some(prompt) = self.usb_access_prompt.as_mut() else {
            return;
        };
        if let Some(outcome) = prompt.poll_install() {
            match outcome {
                UsbAccessOutcome::Installed {
                    endpoint,
                    need_replug,
                } => {
                    let message = prompt.message.clone();
                    self.start_scan();
                    if need_replug {
                        self.state.status = message;
                    } else {
                        self.usb_access_prompt = None;
                        self.refresh_usb_devices();
                        let endpoint = resolve_usb_endpoint(&self.state.devices, &endpoint);
                        self.start_connect_internal(endpoint, true);
                    }
                }
            }
        }
    }

    fn handle_usb_access_action(&mut self, action: UsbAccessAction) {
        let Some(prompt) = self.usb_access_prompt.as_mut() else {
            return;
        };
        match action {
            UsbAccessAction::Install => prompt.start_install(),
            UsbAccessAction::RescanAndConnect => {
                prompt.refresh_status();
                let preferred = prompt.endpoint.clone();
                self.usb_access_prompt = None;
                self.refresh_usb_devices();
                let endpoint = resolve_usb_endpoint(&self.state.devices, &preferred);
                self.start_connect_internal(endpoint, true);
            }
            UsbAccessAction::Dismiss => {
                self.usb_access_prompt = None;
                self.state.connecting_endpoint = None;
                if self.state.status == "USB access required." {
                    self.state.status = "USB setup cancelled.".into();
                }
            }
        }
    }

    fn maybe_live_refresh(&mut self) {
        if self.shutting_down() {
            return;
        }
        if self.state.connection != ConnectionState::Connected {
            return;
        }
        if self.tabs.active == ViewTab::Spectrum
            && self
                .state
                .live_refresh_due(true, self.settings.app.spectrum_refresh_secs)
        {
            debug!("spectrum tab live refresh due");
            self.schedule_spectrum();
        }
    }

    fn draw_central_content(&mut self, ui: &mut Ui, ctx: &Context) {
        if self.tabs.active == ViewTab::Device {
            if let Some(action) = draw_device_view(
                ui,
                DeviceViewProps {
                    devices: &self.state.devices,
                    connection: self.state.connection,
                    connecting_endpoint: self.state.connecting_endpoint.as_ref(),
                    device_info: self.state.device_info.as_ref(),
                    remembered_endpoint: self.settings.app.last_endpoint.as_ref(),
                    scanning: self.state.scanning,
                    busy: self.state.busy,
                    scanned_once: self.state.scanned_once,
                    status: &self.state.status,
                    link_health: self.state.monitor.link_health(),
                    last_spectrum_fetch: self.state.last_fetch,
                    last_monitor_fetch: self.state.last_monitor_fetch,
                },
            ) {
                self.handle_device_action(action);
            }
            return;
        }
        if self.tabs.active == ViewTab::Settings {
            if let Some(action) = draw_settings_view(
                ui,
                &mut self.settings,
                self.state.connection,
                self.state.device_info.as_ref(),
                self.spectrogram.is_recording(),
            ) {
                self.handle_settings_action(action);
            }
            return;
        }
        if self.tabs.active == ViewTab::About {
            page_scroll(ui, "about_page", draw_about_view);
            return;
        }
        if self.tabs.active == ViewTab::Analysis {
            if let Some(action) = draw_analysis_view(
                ui,
                &mut self.analysis,
                &mut self.view.y_scale,
                &self.settings.app,
            ) {
                self.handle_spectrum_plot_action(action);
            }
            return;
        }
        if self.tabs.active == ViewTab::Catalogue {
            if draw_catalogue_view(ui, &mut self.catalogue, &mut self.settings.app) {
                self.settings.persist_app();
            }
            return;
        }
        if shows_tab_content(self.state.connection) {
            let plot_style = histogram_style(self.view.plot_outline_only);
            match self.tabs.active {
                ViewTab::Monitor => {
                    if let Some(action) = draw_monitor_view(
                        ui,
                        &self.state.monitor,
                        &self.state.dosimeter,
                        plot_style,
                        self.settings.app.monitor_smoothing_window,
                        MonitorViewProps {
                            settings: &mut self.settings,
                            connection: self.state.connection,
                            outline_only: &mut self.view.plot_outline_only,
                        },
                    ) {
                        self.handle_monitor_view_action(action);
                    }
                }
                ViewTab::Spectrum => {
                    if let Some(action) = draw_spectrum_toolbar(
                        ui,
                        SpectrumToolbarProps {
                            connection: self.state.connection,
                            y_scale: &mut self.view.y_scale,
                            smooth_window: &mut self.view.smooth_window,
                            outline_only: &mut self.view.plot_outline_only,
                            show_peaks: &mut self.view.show_spectrum_peaks,
                        },
                    ) {
                        self.handle_spectrum_toolbar_action(action);
                    }
                    if let Some(action) = draw_spectrum_plot(
                        ui,
                        self.state.spectrum.as_ref(),
                        self.view.y_scale,
                        self.view.smooth_window,
                        plot_style,
                        self.view.show_spectrum_peaks,
                        crate::ui_plot::SpectrumPlotDrawContext {
                            config: &self.settings.app,
                            spectrum_sequence: self.state.spectrum_sequence,
                            peak_memo: &mut self.view.peak_memo,
                        },
                    ) {
                        self.handle_spectrum_plot_action(action);
                    }
                }
                ViewTab::Spectrogram => {
                    let (controls_action, plot_action) = draw_spectrogram_view(
                        ui,
                        ctx,
                        &mut self.spectrogram,
                        &self.settings.app,
                        self.state.connection,
                    );
                    if let Some(action) = controls_action {
                        self.handle_spectrogram_action(action);
                    }
                    if let Some(action) = plot_action {
                        self.handle_spectrum_plot_action(action);
                    }
                }
                ViewTab::Device
                | ViewTab::Analysis
                | ViewTab::Catalogue
                | ViewTab::Settings
                | ViewTab::About => {}
            }
            return;
        }
        draw_disconnected_view(ui, self.state.connection);
    }
}

impl App for SpectrumApp {
    fn logic(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.ensure_window_icon(ctx);
        self.ensure_startup_window_size(ctx);
        self.apply_ui_scale(ctx);
        if !self.chrome.theme_ready {
            theme::apply(ctx);
            self.chrome.theme_ready = true;
            self.sync_monitor_poll_interval();
            self.sync_capture_interval();
        }
        if !self.chrome.startup_scan_sent {
            self.chrome.startup_scan_sent = true;
            self.start_scan();
        }
        self.handle_close_request(ctx);
        self.poll_events();
        self.advance_close(ctx);
        self.poll_usb_access();
        if self.state.connection == ConnectionState::Connected {
            self.spectrogram.sync_from_capture();
        }
        self.maybe_live_refresh();
        self.maybe_device_config_auto_load();
        if self.shutting_down() {
            ctx.request_repaint_after(Duration::from_millis(16));
        } else if self.state.connection == ConnectionState::Connected
            || self.spectrogram.is_recording()
        {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if let Some(prompt) = self.usb_access_prompt.as_mut()
            && let Some(action) = draw_usb_access_dialog(&ctx, prompt)
        {
            self.handle_usb_access_action(action);
        }
        if let Some(choice) = draw_monitor_leave_confirm(&ctx, self.tabs.monitor_leave_open) {
            self.handle_monitor_leave_choice(choice);
        }
        CentralPanel::default().show(ui, |ui| {
            let previous_tab = self.tabs.previous;
            if let Some(requested) = draw_tab_bar(ui, self.tabs.active) {
                self.try_switch_tab(requested);
            }
            if self.tabs.active == ViewTab::Spectrogram && previous_tab != ViewTab::Spectrogram {
                self.enter_spectrogram_tab();
            }
            if self.tabs.active == ViewTab::Analysis && previous_tab != ViewTab::Analysis {
                self.enter_analysis_tab();
            }
            if self.tabs.active == ViewTab::Catalogue && previous_tab != ViewTab::Catalogue {
                self.enter_catalogue_tab();
            }
            if self.tabs.active == ViewTab::Monitor && previous_tab != ViewTab::Monitor {
                self.enter_monitor_tab();
            }
            if self.tabs.active == ViewTab::Settings && previous_tab != ViewTab::Settings {
                self.enter_settings_tab();
            }
            self.tabs.previous = self.tabs.active;
            ui.separator();
            let content_rect = ui.available_rect_before_wrap();
            ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                ui.set_clip_rect(content_rect);
                let connected = shows_tab_content(self.state.connection);
                let page_inset = tab_uses_page_inset(self.tabs.active)
                    || (!connected && !tab_works_offline(self.tabs.active));
                let plot_pad = connected
                    && tab_uses_plot_pad(self.tabs.active)
                    && self.tabs.active != ViewTab::Monitor;
                if page_inset {
                    with_page_inset(ui, |ui| self.draw_central_content(ui, &ctx));
                } else if plot_pad {
                    with_plot_pad(ui, |ui| self.draw_central_content(ui, &ctx));
                } else {
                    self.draw_central_content(ui, &ctx);
                }
            });
        });
    }
}
