use std::fs;
use std::process::Command;

use radiacode_lib::usb::{UDEV_RULE, UDEV_RULE_PATH};

const TEMP_RULE: &str = "/tmp/radiacode-udev.rules";
const PKEXEC_DISMISSED: i32 = 126;
const PKEXEC_NOT_AUTHORIZED: i32 = 127;

pub fn install_access_rule() -> Result<(), String> {
    fs::write(TEMP_RULE, UDEV_RULE).map_err(|error| error.to_string())?;
    let script = format!(
        "cp '{TEMP_RULE}' '{UDEV_RULE_PATH}' && udevadm control --reload && udevadm trigger"
    );
    run_pkexec(&script)
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

#[cfg(test)]
mod tests {
    use super::pkexec_failure_message;
    use radiacode_lib::usb::{UDEV_RULE, rule_grants_access};

    #[test]
    fn bundled_rule_grants_access() {
        assert!(rule_grants_access(UDEV_RULE));
        assert!(!rule_grants_access("SUBSYSTEM==\"usb\", MODE=\"0600\""));
    }

    #[test]
    fn pkexec_exit_codes_map_to_messages() {
        assert_eq!(pkexec_failure_message(126), "Authentication cancelled.");
        assert_eq!(pkexec_failure_message(127), "Authentication cancelled.");
        assert_eq!(
            pkexec_failure_message(1),
            "Privileged setup failed with exit code 1."
        );
    }
}
