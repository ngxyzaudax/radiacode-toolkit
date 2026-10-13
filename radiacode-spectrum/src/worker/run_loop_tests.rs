use std::sync::Arc;

use super::batch::CoalescedBatch;
use super::epoch::SessionEpoch;
use super::messages::{WorkerCommand, WorkerEvent};
use super::run_loop::{WorkerContext, run_command, run_fetch_batch, worker_loop};
use super::test_fixtures::{Harness, configured_device, count_events};

fn context_from(harness: Harness) -> (WorkerContext, crossbeam_channel::Receiver<WorkerEvent>) {
    let ctx = WorkerContext {
        events: harness.sender,
        session_epoch: Arc::clone(&harness.live_epoch),
        session: harness.session,
    };
    (ctx, harness.events)
}

fn is_busy(expected: bool) -> impl Fn(&WorkerEvent) -> bool {
    move |event| matches!(event, WorkerEvent::Busy(flag) if *flag == expected)
}

#[tokio::test(start_paused = true)]
async fn fetch_batch_wraps_monitor_and_spectrum_in_busy_markers() {
    let (mut ctx, events) = context_from(Harness::connected(configured_device()).await);
    run_fetch_batch(CoalescedBatch::monitor_and_spectrum(), &mut ctx).await;
    let events: Vec<WorkerEvent> = events.try_iter().collect();
    assert!(is_busy(true)(&events[0]));
    assert!(is_busy(false)(events.last().expect("events")));
    assert_eq!(
        count_events(&events, |event| matches!(
            event,
            WorkerEvent::MonitorSample(_)
        )),
        1
    );
    assert_eq!(
        count_events(&events, |event| matches!(event, WorkerEvent::Spectrum(_))),
        1
    );
    assert!(ctx.session.has_device());
}

#[tokio::test(start_paused = true)]
async fn disconnect_command_ends_epoch_and_clears_session() {
    let (mut ctx, events) = context_from(Harness::connected(configured_device()).await);
    ctx.session.monitor_polls = 4;
    let epoch_before = SessionEpoch::capture(&ctx.session_epoch);
    run_command(WorkerCommand::Disconnect, &mut ctx).await;
    let events: Vec<WorkerEvent> = events.try_iter().collect();
    assert!(!epoch_before.active());
    assert!(
        events
            .iter()
            .any(|event| matches!(event, WorkerEvent::Disconnected))
    );
    assert!(!ctx.session.has_device());
    assert_eq!(ctx.session.monitor_polls, 0);
    assert!(is_busy(false)(events.last().expect("events")));
}

#[tokio::test(start_paused = true)]
async fn fetch_monitor_command_runs_single_poll() {
    let (mut ctx, events) = context_from(Harness::connected(configured_device()).await);
    run_command(WorkerCommand::FetchMonitor, &mut ctx).await;
    let events: Vec<WorkerEvent> = events.try_iter().collect();
    assert_eq!(
        count_events(&events, |event| matches!(
            event,
            WorkerEvent::MonitorPollComplete
        )),
        1
    );
    assert_eq!(ctx.session.monitor_polls, 1);
}

#[tokio::test(start_paused = true)]
async fn interval_commands_are_ignored_by_run_command() {
    let (mut ctx, events) = context_from(Harness::connected(configured_device()).await);
    run_command(WorkerCommand::SetCaptureInterval(7.0), &mut ctx).await;
    run_command(WorkerCommand::SetMonitorPollInterval(3), &mut ctx).await;
    let events: Vec<WorkerEvent> = events.try_iter().collect();
    assert_eq!(events.len(), 4);
    assert!(
        events
            .iter()
            .all(|event| matches!(event, WorkerEvent::Busy(_)))
    );
    assert!(ctx.session.has_device());
}

#[tokio::test(start_paused = true)]
async fn worker_loop_processes_commands_until_channel_closes() {
    let (ctx, events) = context_from(Harness::connected(configured_device()).await);
    let (commands, command_rx) = tokio::sync::mpsc::unbounded_channel();
    commands
        .send(WorkerCommand::SetCaptureInterval(2.0))
        .unwrap();
    commands
        .send(WorkerCommand::SetMonitorPollInterval(5))
        .unwrap();
    commands.send(WorkerCommand::FetchMonitor).unwrap();
    commands.send(WorkerCommand::Disconnect).unwrap();
    drop(commands);
    worker_loop(command_rx, ctx).await;
    let events: Vec<WorkerEvent> = events.try_iter().collect();
    assert!(
        count_events(&events, |event| matches!(
            event,
            WorkerEvent::MonitorPollComplete
        )) >= 1
    );
    assert_eq!(
        count_events(&events, |event| matches!(event, WorkerEvent::Disconnected)),
        1
    );
}
