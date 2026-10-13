use crate::udev::{
    RULES, UsbAccessStatus, pkexec_failure_message, rule_grants_access, status_for_denied_access,
};

#[test]
fn bundled_rule_grants_access() {
    assert!(rule_grants_access(RULES));
    assert!(!rule_grants_access("SUBSYSTEM==\"usb\", MODE=\"0600\""));
}

#[test]
fn denied_access_status_depends_on_rule_presence() {
    assert_eq!(
        status_for_denied_access(true),
        UsbAccessStatus::RuleInstalledNeedReplug
    );
    assert_eq!(
        status_for_denied_access(false),
        UsbAccessStatus::RuleMissing
    );
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
