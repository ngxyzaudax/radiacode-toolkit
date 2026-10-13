use std::time::Instant;

use super::{CloseAction, CloseState, SHUTDOWN_DISCONNECT_TIMEOUT, ShutdownSequence};

fn awaiting_disconnect(started: Instant) -> ShutdownSequence {
    let mut shutdown = ShutdownSequence::new();
    shutdown.state = CloseState::AwaitingDisconnect;
    shutdown.started = Some(started);
    shutdown
}

#[test]
fn close_request_while_closing_is_ignored() {
    let mut shutdown = ShutdownSequence::new();
    shutdown.state = CloseState::Closing;
    let action = shutdown.on_close_request(true, true);
    assert!(matches!(action, CloseAction::None));
    assert_eq!(shutdown.state, CloseState::Closing);
}

#[test]
fn no_close_request_is_a_noop() {
    let mut shutdown = ShutdownSequence::new();
    assert!(matches!(
        shutdown.on_close_request(false, true),
        CloseAction::None
    ));
    assert!(!shutdown.active());
}

#[test]
fn close_without_device_completes_immediately() {
    let mut shutdown = ShutdownSequence::new();
    let action = shutdown.on_close_request(true, false);
    assert!(matches!(action, CloseAction::CompleteClose));
    assert_eq!(shutdown.state, CloseState::Closing);
}

#[test]
fn close_with_device_requests_disconnect_first() {
    let mut shutdown = ShutdownSequence::new();
    let action = shutdown.on_close_request(true, true);
    assert!(matches!(action, CloseAction::DisconnectDevice));
    assert_eq!(shutdown.state, CloseState::AwaitingDisconnect);
    assert!(shutdown.active());
    assert!(!shutdown.closing());
    assert!(matches!(shutdown.advance_close(), CloseAction::None));
}

#[test]
fn closing_reports_closing() {
    let mut shutdown = ShutdownSequence::new();
    assert!(!shutdown.closing());
    shutdown.on_close_request(true, false);
    assert!(shutdown.closing());
}

#[test]
fn second_close_request_while_awaiting_disconnect_completes_shutdown() {
    let mut shutdown = awaiting_disconnect(Instant::now());
    let action = shutdown.on_close_request(true, false);
    assert!(matches!(action, CloseAction::CompleteClose));
    assert_eq!(shutdown.state, CloseState::Closing);
}

#[test]
fn disconnected_event_completes_shutdown_on_next_advance() {
    let mut shutdown = awaiting_disconnect(Instant::now());
    shutdown.on_disconnected();
    let action = shutdown.advance_close();
    assert!(matches!(action, CloseAction::CompleteClose));
    assert_eq!(shutdown.state, CloseState::Closing);
}

#[test]
fn disconnected_event_outside_shutdown_is_ignored() {
    let mut shutdown = ShutdownSequence::new();
    shutdown.on_disconnected();
    assert!(!shutdown.after_disconnect);
    assert!(matches!(shutdown.advance_close(), CloseAction::None));
}

#[test]
fn disconnect_timeout_forces_close() {
    let expired = Instant::now()
        .checked_sub(SHUTDOWN_DISCONNECT_TIMEOUT)
        .expect("instant arithmetic");
    let mut shutdown = awaiting_disconnect(expired);
    assert!(matches!(
        shutdown.advance_close(),
        CloseAction::CompleteClose
    ));
    assert_eq!(shutdown.state, CloseState::Closing);
}

#[test]
fn awaiting_without_start_time_closes_defensively() {
    let mut shutdown = ShutdownSequence::new();
    shutdown.state = CloseState::AwaitingDisconnect;
    assert!(matches!(
        shutdown.advance_close(),
        CloseAction::CompleteClose
    ));
}
