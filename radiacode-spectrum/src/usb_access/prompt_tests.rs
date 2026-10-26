use std::time::{Duration, Instant};

use radiacode_lib::DeviceEndpoint;
use radiacode_lib::usb::UsbAccessStatus;

use super::prompt::{UsbAccessOutcome, UsbAccessPrompt, prompt_message};

fn usb_endpoint() -> DeviceEndpoint {
    DeviceEndpoint::Usb {
        serial: "RC-103-000001".into(),
    }
}

fn prompt(status: UsbAccessStatus) -> UsbAccessPrompt {
    UsbAccessPrompt::new(usb_endpoint(), status)
}

fn wait_for_outcome(prompt: &mut UsbAccessPrompt) -> Option<UsbAccessOutcome> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if !prompt.installing {
            return None;
        }
        if let Some(outcome) = prompt.poll_install() {
            return Some(outcome);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("install did not finish");
}

#[test]
fn new_prompt_uses_status_message() {
    let prompt = prompt(UsbAccessStatus::RuleMissing);
    assert_eq!(prompt.message, prompt_message(UsbAccessStatus::RuleMissing));
    assert!(!prompt.installing);
}

#[test]
fn successful_install_reports_replug_requirement() {
    let mut prompt = prompt(UsbAccessStatus::RuleMissing);
    let outcome = prompt.apply_install_result(Ok(()), UsbAccessStatus::RuleInstalledNeedReplug);
    assert!(matches!(
        outcome,
        Some(UsbAccessOutcome::Installed {
            need_replug: true,
            ..
        })
    ));
    assert_eq!(prompt.status, UsbAccessStatus::RuleInstalledNeedReplug);
}

#[test]
fn failed_install_surfaces_error_message() {
    let mut prompt = prompt(UsbAccessStatus::RuleMissing);
    let outcome = prompt.apply_install_result(
        Err("Authentication cancelled.".into()),
        UsbAccessStatus::RuleMissing,
    );
    assert!(outcome.is_none());
    assert_eq!(prompt.message, "Authentication cancelled.");
}

#[test]
fn install_thread_result_is_polled_back() {
    let mut prompt = prompt(UsbAccessStatus::RuleMissing);
    prompt.start_install_with(|| Err("denied".into()));
    assert!(prompt.installing);
    assert!(wait_for_outcome(&mut prompt).is_none());
    assert!(!prompt.installing);
    assert_eq!(prompt.message, "denied");
}

#[test]
fn second_start_while_installing_is_ignored() {
    let mut prompt = prompt(UsbAccessStatus::RuleMissing);
    prompt.start_install_with(|| {
        std::thread::sleep(Duration::from_millis(50));
        Err("first".into())
    });
    prompt.start_install_with(|| Err("second".into()));
    wait_for_outcome(&mut prompt);
    assert_eq!(prompt.message, "first");
}
