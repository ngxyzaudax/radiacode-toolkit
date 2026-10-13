use std::time::{Duration, Instant};

use crate::device::link_health::MonitorLinkHealth;
use crate::device::link_quality::LinkQuality;
use crate::device::refresh_age::refresh_age_label;

#[test]
fn link_quality_thresholds_are_inclusive_at_boundaries() {
    assert_eq!(LinkQuality::from_rssi(-55), LinkQuality::Excellent);
    assert_eq!(LinkQuality::from_rssi(-56), LinkQuality::Good);
    assert_eq!(LinkQuality::from_rssi(-65), LinkQuality::Good);
    assert_eq!(LinkQuality::from_rssi(-66), LinkQuality::Fair);
    assert_eq!(LinkQuality::from_rssi(-75), LinkQuality::Fair);
    assert_eq!(LinkQuality::from_rssi(-76), LinkQuality::Weak);
    assert_eq!(LinkQuality::from_rssi(-85), LinkQuality::Weak);
    assert_eq!(LinkQuality::from_rssi(-86), LinkQuality::Poor);
}

#[test]
fn link_quality_bars_decrease_monotonically() {
    let qualities = [
        LinkQuality::Excellent,
        LinkQuality::Good,
        LinkQuality::Fair,
        LinkQuality::Weak,
        LinkQuality::Poor,
    ];
    let bars: Vec<u8> = qualities.iter().map(|quality| quality.bars()).collect();
    assert_eq!(bars, vec![4, 3, 2, 1, 0]);
    assert_eq!(LinkQuality::Poor.label(), "Poor");
    assert_eq!(LinkQuality::Good.color(), LinkQuality::Fair.color());
}

#[test]
fn link_health_summary_reports_counts_only_when_present() {
    let clean = MonitorLinkHealth::default();
    assert!(!clean.has_issues());
    assert_eq!(clean.summary(), "Link health: clean");
    let noisy = MonitorLinkHealth {
        decode_warnings: 1,
        rejected_records: 2,
        resync_count: 3,
        seq_gaps: 4,
        lost_records: 5,
    };
    assert!(noisy.has_issues());
    assert_eq!(
        noisy.summary(),
        "Link health: 1 decode warnings, 2 rejected, 3 resyncs, 4 seq gaps, 5 lost records"
    );
}

#[test]
fn refresh_age_label_buckets_elapsed_time() {
    assert_eq!(refresh_age_label(None), "never");
    assert_eq!(refresh_age_label(Some(Instant::now())), "just now");
    let seconds_ago = Instant::now().checked_sub(Duration::from_secs(30)).unwrap();
    assert_eq!(refresh_age_label(Some(seconds_ago)), "30s ago");
    let minutes_ago = Instant::now()
        .checked_sub(Duration::from_secs(150))
        .unwrap();
    assert_eq!(refresh_age_label(Some(minutes_ago)), "2m ago");
}
