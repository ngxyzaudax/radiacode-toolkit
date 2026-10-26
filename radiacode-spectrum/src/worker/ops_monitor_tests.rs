use super::epoch::end_session;
use super::messages::WorkerEvent;
use super::ops_monitor::handle_monitor;
use super::test_fixtures::{Harness, configured_device, count_events, epoch_of, is_monitor_sample};

fn has_poll_complete(events: &[WorkerEvent]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, WorkerEvent::MonitorPollComplete))
}

#[tokio::test(start_paused = true)]
async fn monitor_poll_publishes_limits_sample_status_and_completion() {
    let mut harness = Harness::connected(configured_device()).await;
    handle_monitor(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert!(matches!(events[0], WorkerEvent::AlarmLimits(limits) if limits.l2_count_rate == 60.0));
    assert_eq!(count_events(&events, is_monitor_sample), 1);
    assert!(matches!(
        events[2],
        WorkerEvent::DeviceStatus(status) if status.battery_percent == Some(87.5)
    ));
    assert!(matches!(events[3], WorkerEvent::MonitorPollComplete));
    assert_eq!(harness.session.monitor_polls, 1);
    assert!(harness.session.alarm_limits.is_some());
    assert!(harness.session.has_device());
}

#[tokio::test(start_paused = true)]
async fn monitor_poll_without_data_reports_status_only() {
    let device = configured_device().with_string(radiacode_lib::protocol::VirtString::DataBuf, &[]);
    let mut harness = Harness::connected(device).await;
    handle_monitor(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert_eq!(count_events(&events, is_monitor_sample), 0);
    assert!(has_poll_complete(&events));
    assert_eq!(harness.session.monitor_polls, 0);
}

#[tokio::test(start_paused = true)]
async fn monitor_poll_after_session_end_is_silent() {
    let mut harness = Harness::connected(configured_device()).await;
    let epoch = epoch_of(&harness.live_epoch);
    end_session(&harness.live_epoch);
    handle_monitor(&harness.sender, &mut harness.session, &epoch).await;
    let events = harness.drain();
    assert_eq!(count_events(&events, is_monitor_sample), 0);
    assert!(!has_poll_complete(&events));
    assert!(harness.session.has_device());
}
