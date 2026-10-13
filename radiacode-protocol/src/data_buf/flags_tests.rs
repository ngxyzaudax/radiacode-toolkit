use super::flags::{EventId, StatusFlags};

const KNOWN_EVENTS: [EventId; 23] = [
    EventId::PowerOff,
    EventId::PowerOn,
    EventId::LowBatteryShutdown,
    EventId::ChangeDeviceParams,
    EventId::DoseReset,
    EventId::UserEvent,
    EventId::BatteryEmptyAlarm,
    EventId::ChargeStart,
    EventId::ChargeStop,
    EventId::DoseRateAlarm1,
    EventId::DoseRateAlarm2,
    EventId::DoseRateOffscale,
    EventId::DoseAlarm1,
    EventId::DoseAlarm2,
    EventId::DoseOffscale,
    EventId::TemperatureTooLow,
    EventId::TemperatureTooHigh,
    EventId::TextMessage,
    EventId::MemorySnapshot,
    EventId::SpectrumReset,
    EventId::CountRateAlarm1,
    EventId::CountRateAlarm2,
    EventId::CountRateOffscale,
];

#[test]
fn every_known_event_id_decodes_from_its_raw_value() {
    for (raw, expected) in KNOWN_EVENTS.iter().enumerate() {
        assert_eq!(EventId::from_raw(raw as u8), *expected);
    }
}

#[test]
fn unknown_event_ids_are_preserved() {
    assert_eq!(EventId::from_raw(23), EventId::Unknown(23));
    assert_eq!(EventId::from_raw(255), EventId::Unknown(255));
}

#[test]
fn status_flags_round_trip_raw_value() {
    assert_eq!(StatusFlags::new(0xBEEF).raw(), 0xBEEF);
    assert_eq!(StatusFlags::new(0), StatusFlags::new(0));
}
