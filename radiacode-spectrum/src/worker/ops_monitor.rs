use crossbeam_channel::Sender;
use radiacode_lib::config::AlarmLimits;
use radiacode_lib::{DeviceStatus, MonitorPollSample, RadiaCode};
use tracing::{debug, warn};

use super::epoch::SessionEpoch;
use super::messages::WorkerEvent;
use super::ops_reconnect::handle_device_error;
use super::session::WorkerSession;

const ALARM_REFRESH_POLLS: u64 = 120;

pub async fn handle_monitor(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    epoch: &SessionEpoch,
) {
    let Some(mut device) = session.device.take() else {
        warn!("monitor fetch skipped: no active device");
        return;
    };
    let limits = match ensure_alarm_limits(&mut device, session, events).await {
        Ok(limits) => limits,
        Err(error) => {
            session.device = handle_device_error(events, session, device, error, epoch).await;
            return;
        }
    };
    let poll = device
        .poll_monitor(limits.display_units(), &mut session.data_buf_cursor, false)
        .await;
    session.device = match poll {
        Ok((sample, fresh)) => {
            publish_monitor_poll(events, session, sample, fresh, epoch);
            Some(device)
        }
        Err(error) => handle_device_error(events, session, device, error, epoch).await,
    };
}

fn publish_monitor_poll(
    events: &Sender<WorkerEvent>,
    session: &mut WorkerSession,
    sample: MonitorPollSample,
    fresh: DeviceStatus,
    epoch: &SessionEpoch,
) {
    session.link_status = session.link_status.merged(fresh);
    if !epoch.active() {
        return;
    }
    if sample_has_data(&sample) {
        session.monitor_polls = session.monitor_polls.saturating_add(1);
        let _ = events.send(WorkerEvent::MonitorSample(sample));
    } else {
        debug!("monitor data not yet available in databuf");
    }
    let _ = events.send(WorkerEvent::DeviceStatus(session.link_status));
    let _ = events.send(WorkerEvent::MonitorPollComplete);
}

fn sample_has_data(sample: &MonitorPollSample) -> bool {
    !sample.rates.is_empty() || sample.accumulated.is_some()
}

pub fn alarm_refresh_due(cache: Option<&AlarmLimits>, monitor_polls: u64) -> bool {
    cache.is_none() || monitor_polls.is_multiple_of(ALARM_REFRESH_POLLS)
}

async fn ensure_alarm_limits(
    device: &mut RadiaCode,
    session: &mut WorkerSession,
    events: &Sender<WorkerEvent>,
) -> radiacode_lib::Result<AlarmLimits> {
    if alarm_refresh_due(session.alarm_limits.as_ref(), session.monitor_polls) {
        let limits = device.alarm_limits().await?;
        session.alarm_limits = Some(limits);
        let _ = events.send(WorkerEvent::AlarmLimits(limits));
    }
    Ok(session.alarm_limits.expect("alarm limits cached"))
}

#[cfg(test)]
mod tests {
    use super::alarm_refresh_due;
    use crate::worker::test_fixtures::alarm_limits;

    #[test]
    fn refresh_when_cache_missing_or_at_period_boundary() {
        let cached = alarm_limits();
        assert!(alarm_refresh_due(None, 7));
        assert!(alarm_refresh_due(Some(&cached), 0));
        assert!(alarm_refresh_due(Some(&cached), 240));
        assert!(!alarm_refresh_due(Some(&cached), 7));
        assert!(!alarm_refresh_due(Some(&cached), 119));
    }
}
