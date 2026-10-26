use std::sync::mpsc::{self, Receiver, TryRecvError};

use radiacode_lib::DeviceEndpoint;
use radiacode_lib::usb::UsbAccessStatus;
use tracing::info;

use super::blocking::access_status;
use super::install::install_access_rule;

type InstallResult = Result<(), String>;

pub struct UsbAccessPrompt {
    pub endpoint: DeviceEndpoint,
    pub status: UsbAccessStatus,
    pub installing: bool,
    pub message: String,
    install_rx: Option<Receiver<InstallResult>>,
}

pub enum UsbAccessOutcome {
    Installed {
        endpoint: DeviceEndpoint,
        need_replug: bool,
    },
}

pub enum UsbAccessAction {
    Install,
    RescanAndConnect,
    Dismiss,
}

impl UsbAccessPrompt {
    pub fn new(endpoint: DeviceEndpoint, status: UsbAccessStatus) -> Self {
        Self {
            endpoint,
            status,
            installing: false,
            message: prompt_message(status),
            install_rx: None,
        }
    }

    pub fn refresh_status(&mut self) {
        self.apply_status(access_status());
    }

    pub(crate) fn apply_status(&mut self, status: UsbAccessStatus) {
        self.status = status;
        self.message = prompt_message(status);
    }

    pub fn poll_install(&mut self) -> Option<UsbAccessOutcome> {
        let received = self.install_rx.as_ref()?.try_recv();
        if matches!(received, Err(TryRecvError::Empty)) {
            return None;
        }
        self.install_rx = None;
        self.installing = false;
        self.apply_install_result(install_result(received), access_status())
    }

    pub(crate) fn apply_install_result(
        &mut self,
        result: InstallResult,
        status: UsbAccessStatus,
    ) -> Option<UsbAccessOutcome> {
        match result {
            Ok(()) => {
                self.apply_status(status);
                Some(UsbAccessOutcome::Installed {
                    endpoint: self.endpoint.clone(),
                    need_replug: status == UsbAccessStatus::RuleInstalledNeedReplug,
                })
            }
            Err(message) => {
                self.message = message;
                None
            }
        }
    }

    pub fn start_install(&mut self) {
        self.start_install_with(install_access_rule);
    }

    pub(crate) fn start_install_with(
        &mut self,
        installer: impl FnOnce() -> InstallResult + Send + 'static,
    ) {
        if self.installing {
            return;
        }
        info!("starting usb access rule install via pkexec");
        let (tx, rx) = mpsc::channel();
        self.install_rx = Some(rx);
        self.installing = true;
        self.message = "Waiting for system authentication…".into();
        std::thread::spawn(move || {
            let _ = tx.send(installer());
        });
    }
}

fn install_result(received: Result<InstallResult, TryRecvError>) -> InstallResult {
    match received {
        Ok(result) => result,
        Err(_) => Err("USB setup stopped unexpectedly.".into()),
    }
}

pub fn usb_access_required(endpoint: &DeviceEndpoint) -> Option<UsbAccessStatus> {
    if !matches!(endpoint, DeviceEndpoint::Usb { .. }) {
        return None;
    }
    match access_status() {
        UsbAccessStatus::Granted => None,
        status => Some(status),
    }
}

pub(crate) fn prompt_message(status: UsbAccessStatus) -> String {
    match status {
        UsbAccessStatus::Granted => "USB access is available.".into(),
        UsbAccessStatus::RuleMissing => {
            "Linux blocked USB access. Grant access once to install a udev rule, then replug the device.".into()
        }
        UsbAccessStatus::RuleInstalledNeedReplug => {
            "Unplug and replug the Radiacode, then press Rescan and connect.".into()
        }
    }
}
