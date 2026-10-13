use super::SpectrumApp;
use crate::usb_access::{UsbAccessAction, UsbAccessOutcome};

const USB_ACCESS_REQUIRED_STATUS: &str = "USB access required.";

impl SpectrumApp {
    pub(super) fn poll_usb_access(&mut self) {
        let Some(outcome) = self
            .usb_access_prompt
            .as_mut()
            .and_then(|prompt| prompt.poll_install())
        else {
            return;
        };
        let UsbAccessOutcome::Installed {
            endpoint,
            need_replug,
        } = outcome;
        let message = self
            .usb_access_prompt
            .as_ref()
            .map(|prompt| prompt.message.clone())
            .unwrap_or_default();
        self.start_scan();
        if need_replug {
            self.state.status = message;
        } else {
            self.usb_access_prompt = None;
            self.connect_resolved(&endpoint);
        }
    }

    pub(super) fn handle_usb_access_action(&mut self, action: UsbAccessAction) {
        let Some(prompt) = self.usb_access_prompt.as_mut() else {
            return;
        };
        match action {
            UsbAccessAction::Install => prompt.start_install(),
            UsbAccessAction::RescanAndConnect => {
                prompt.refresh_status();
                let preferred = prompt.endpoint.clone();
                self.usb_access_prompt = None;
                self.connect_resolved(&preferred);
            }
            UsbAccessAction::Dismiss => self.dismiss_usb_access_prompt(),
        }
    }

    fn dismiss_usb_access_prompt(&mut self) {
        self.usb_access_prompt = None;
        self.state.connecting_endpoint = None;
        if self.state.status == USB_ACCESS_REQUIRED_STATUS {
            self.state.status = "USB setup cancelled.".into();
        }
    }
}
