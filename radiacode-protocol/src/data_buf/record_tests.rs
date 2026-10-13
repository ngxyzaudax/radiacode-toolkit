use super::{DeviceTicks, EventId, RecordKind, Seq};

#[test]
fn record_kind_mapping_covers_known_groups() {
    assert_eq!(RecordKind::from_entity_group(0, 2), RecordKind::DoseRateDb);
    assert_eq!(
        RecordKind::from_entity_group(0, 8),
        RecordKind::RawCountRate
    );
    assert_eq!(RecordKind::from_entity_group(1, 2), RecordKind::Waveform16);
    assert_eq!(
        RecordKind::from_entity_group(9, 9),
        RecordKind::Unknown {
            entity: 9,
            group: 9
        }
    );
    assert_eq!(RecordKind::RealTimeData.monitor_source_rank(), 3);
    assert_eq!(RecordKind::DoseRateDb.monitor_source_rank(), 2);
    assert_eq!(RecordKind::Event.monitor_source_rank(), 0);
}

#[test]
fn event_ids_map_known_values_and_keep_unknown_raw() {
    assert_eq!(EventId::from_raw(0), EventId::PowerOff);
    assert_eq!(EventId::from_raw(4), EventId::DoseReset);
    assert_eq!(EventId::from_raw(22), EventId::CountRateOffscale);
    assert_eq!(EventId::from_raw(200), EventId::Unknown(200));
}

#[test]
fn device_ticks_conversions() {
    let ticks = DeviceTicks::new(250);
    assert_eq!(ticks.raw(), 250);
    assert_eq!(ticks.as_duration().as_millis(), 2500);
    assert_eq!(DeviceTicks::new(-5).as_duration().as_millis(), 0);
    let base = std::time::UNIX_EPOCH;
    assert_eq!(
        ticks.wall_clock_time(base),
        base + std::time::Duration::from_millis(2500)
    );
}

#[test]
fn seq_loss_accounting() {
    assert_eq!(Seq::new(5).lost_since(Seq::new(5)), Some(0));
    assert_eq!(Seq::new(7).lost_since(Seq::new(5)), Some(2));
    assert_eq!(Seq::new(3).lost_since(Seq::new(5)), None);
    assert_eq!(Seq::new(255).next(), Seq::new(0));
}
