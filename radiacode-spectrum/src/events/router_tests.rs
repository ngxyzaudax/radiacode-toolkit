use super::router_fixture::Fixture;
use super::test_fixtures::device_info;
use crate::model::ConnectionState;
use crate::view_tab::ViewTab;
use crate::worker::{WorkerCommand, WorkerEvent};

#[test]
fn initial_connect_syncs_intervals_and_switches_to_monitor() {
    let mut fixture = Fixture::new(ViewTab::Device);
    fixture.state.connection = ConnectionState::Connecting;
    let (outcome, sent) = fixture.dispatch(WorkerEvent::Connected(device_info()));
    assert!(outcome.fetch_spectrum);
    assert!(outcome.sync_capture_interval);
    assert!(outcome.sync_monitor_poll_interval);
    assert_eq!(outcome.switch_tab, Some(ViewTab::Monitor));
    assert!(outcome.remember_device.is_some());
    assert_eq!(sent.len(), 1);
    assert!(matches!(sent[0], WorkerCommand::FetchMonitor));
    assert!(fixture.state.monitor_fetch_pending);
}

#[test]
fn initial_connect_on_config_tab_requests_device_config() {
    let mut fixture = Fixture::new(ViewTab::Settings);
    fixture.state.connection = ConnectionState::Connecting;
    let (outcome, sent) = fixture.dispatch(WorkerEvent::Connected(device_info()));
    assert!(outcome.switch_tab.is_none());
    assert!(
        sent.iter()
            .any(|command| matches!(command, WorkerCommand::FetchDeviceConfig))
    );
}

#[test]
fn reconnect_does_not_resync_or_remember() {
    let mut fixture = Fixture::new(ViewTab::Spectrum);
    fixture.state.connection = ConnectionState::Connected;
    let (outcome, _) = fixture.dispatch(WorkerEvent::Connected(device_info()));
    assert!(outcome.fetch_spectrum);
    assert!(!outcome.sync_capture_interval);
    assert!(outcome.remember_device.is_none());
}

#[test]
fn blocked_session_ignores_connect() {
    let mut fixture = Fixture::new(ViewTab::Device);
    fixture.session_blocked = true;
    let (outcome, sent) = fixture.dispatch(WorkerEvent::Connected(device_info()));
    assert!(!outcome.fetch_spectrum);
    assert!(sent.is_empty());
    assert_eq!(fixture.state.connection, ConnectionState::Disconnected);
}
