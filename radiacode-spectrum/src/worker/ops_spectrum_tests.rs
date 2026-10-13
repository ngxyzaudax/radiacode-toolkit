use radiacode_core::Command;
use radiacode_core::test_support::ScriptedReply;
use radiacode_core::test_support::encode::u32_words;

use super::epoch::end_session;
use super::messages::WorkerEvent;
use super::ops_spectrum::{handle_dose_reset, handle_reset, handle_spectrum};
use super::test_fixtures::{
    Harness, configured_device, count_events, epoch_of, is_disconnected, is_error, is_spectrum,
    usb_endpoint,
};

#[tokio::test(start_paused = true)]
async fn spectrum_fetch_retries_transient_errors() {
    let mut harness = Harness::connected(configured_device()).await;
    harness.script(Command::RdVirtString, ScriptedReply::timeout());
    handle_spectrum(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert_eq!(count_events(&events, is_spectrum), 1);
    let WorkerEvent::Spectrum(view) = &events[0] else {
        panic!("unexpected event {:?}", events[0]);
    };
    assert_eq!(view.total_counts, 18);
    assert_eq!(view.counts, vec![5, 6, 7]);
}

#[tokio::test(start_paused = true)]
async fn spectrum_fetch_reports_error_without_reconnect_when_endpoint_unknown() {
    let mut harness = Harness::connected(configured_device()).await;
    for _ in 0..3 {
        harness.script(Command::RdVirtString, ScriptedReply::timeout());
    }
    handle_spectrum(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert_eq!(count_events(&events, is_error), 1);
    assert!(harness.session.has_device());
}

#[tokio::test(start_paused = true)]
async fn connection_loss_without_restore_cache_disconnects() {
    let mut harness = Harness::connected(configured_device()).await;
    harness.script(Command::RdVirtString, ScriptedReply::connection_closed());
    harness.session.session_endpoint = Some(usb_endpoint());
    harness.session.session_restore = None;
    handle_spectrum(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert_eq!(count_events(&events, is_disconnected), 1);
    assert!(!harness.session.has_device());
}

#[tokio::test(start_paused = true)]
async fn connection_loss_after_session_end_drops_device_silently() {
    let mut harness = Harness::connected(configured_device()).await;
    harness.script(Command::RdVirtString, ScriptedReply::connection_closed());
    harness.session.session_endpoint = Some(usb_endpoint());
    let epoch = epoch_of(&harness.live_epoch);
    end_session(&harness.live_epoch);
    handle_spectrum(&harness.sender, &mut harness.session, &epoch).await;
    assert!(harness.drain().is_empty());
    assert!(!harness.session.has_device());
}

#[tokio::test(start_paused = true)]
async fn dose_reset_writes_register_and_confirms() {
    let mut harness = Harness::connected(configured_device()).await;
    let writes_before = harness.handle.device().request_count(Command::WrVirtSfr);
    handle_dose_reset(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert!(matches!(events[0], WorkerEvent::DoseResetComplete));
    let writes = harness.handle.device().request_count(Command::WrVirtSfr);
    assert_eq!(writes, writes_before + 1);
}

#[tokio::test(start_paused = true)]
async fn spectrum_reset_clears_then_refetches() {
    let mut harness = Harness::connected(configured_device()).await;
    harness.script(
        Command::WrVirtString,
        ScriptedReply::Payload(u32_words(&[1])),
    );
    handle_reset(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert_eq!(count_events(&events, is_spectrum), 1);
    assert_eq!(
        harness.handle.device().request_count(Command::WrVirtString),
        1
    );
}
