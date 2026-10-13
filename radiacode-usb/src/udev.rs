use std::fs;
use std::process::Command;

use rusb::{Context, UsbContext};

use crate::transport::usb_permission_denied;

pub const RULES: &str = "SUBSYSTEM==\"usb\", ATTR{idVendor}==\"0483\", ATTR{idProduct}==\"f123\", MODE=\"0666\", TAG+=\"uaccess\"\n";
pub const RULE_DEST: &str = "/etc/udev/rules.d/99-radiacode.rules";
const TEMP_RULE: &str = "/tmp/radiacode-udev.rules";
const PKEXEC_DISMISSED: i32 = 126;
const PKEXEC_NOT_AUTHORIZED: i32 = 127;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbAccessStatus {
    Granted,
    RuleMissing,
    RuleInstalledNeedReplug,
}

pub fn access_status() -> UsbAccessStatus {
    if !device_present() || !usb_permission_denied() {
        return UsbAccessStatus::Granted;
    }
    status_for_denied_access(rule_installed())
}

pub(crate) fn status_for_denied_access(rule_installed: bool) -> UsbAccessStatus {
    if rule_installed {
        UsbAccessStatus::RuleInstalledNeedReplug
    } else {
        UsbAccessStatus::RuleMissing
    }
}

pub fn rule_installed() -> bool {
    fs::read_to_string(RULE_DEST)
        .ok()
        .is_some_and(|content| rule_grants_access(&content))
}

pub(crate) fn rule_grants_access(content: &str) -> bool {
    content.contains("0483") && content.contains("f123") && content.contains("0666")
}

pub fn install_access_rule() -> Result<(), String> {
    fs::write(TEMP_RULE, RULES).map_err(|error| error.to_string())?;
    let script = format!(
        "cp '{}' '{}' && udevadm control --reload && udevadm trigger",
        TEMP_RULE, RULE_DEST
    );
    run_pkexec(&script)
}

fn device_present() -> bool {
    let Ok(context) = Context::new() else {
        return false;
    };
    let Ok(devices) = context.devices() else {
        return false;
    };
    devices
        .iter()
        .any(|device| crate::transport::is_radiacode_device(&device))
}

fn run_pkexec(script: &str) -> Result<(), String> {
    let mut command = Command::new("pkexec");
    command.args(["sh", "-c", script]);
    for variable in ["DISPLAY", "XAUTHORITY"] {
        if let Ok(value) = std::env::var(variable) {
            command.env(variable, value);
        }
    }
    let status = command
        .status()
        .map_err(|error| format!("pkexec unavailable: {error}"))?;
    if status.success() {
        return Ok(());
    }
    Err(pkexec_failure_message(status.code().unwrap_or(-1)))
}

pub(crate) fn pkexec_failure_message(code: i32) -> String {
    match code {
        PKEXEC_DISMISSED | PKEXEC_NOT_AUTHORIZED => "Authentication cancelled.".into(),
        other => format!("Privileged setup failed with exit code {other}."),
    }
}
