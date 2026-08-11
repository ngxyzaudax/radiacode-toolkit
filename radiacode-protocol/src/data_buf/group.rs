#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecordKind {
    RealTimeData,
    RawData,
    DoseRateDb,
    RareData,
    UserData,
    ScheduleData,
    AccelData,
    Event,
    RawCountRate,
    RawDoseRate,
    Waveform8,
    Waveform16,
    Waveform14,
    Unknown { entity: u8, group: u8 },
}

impl RecordKind {
    pub fn from_entity_group(entity: u8, group: u8) -> Self {
        match (entity, group) {
            (0, 0) => Self::RealTimeData,
            (0, 1) => Self::RawData,
            (0, 2) => Self::DoseRateDb,
            (0, 3) => Self::RareData,
            (0, 4) => Self::UserData,
            (0, 5) => Self::ScheduleData,
            (0, 6) => Self::AccelData,
            (0, 7) => Self::Event,
            (0, 8) => Self::RawCountRate,
            (0, 9) => Self::RawDoseRate,
            (1, 1) => Self::Waveform8,
            (1, 2) => Self::Waveform16,
            (1, 3) => Self::Waveform14,
            (entity, group) => Self::Unknown { entity, group },
        }
    }

    pub fn monitor_source_rank(self) -> u8 {
        match self {
            Self::RealTimeData => 3,
            Self::DoseRateDb => 2,
            Self::RawData => 1,
            _ => 0,
        }
    }

    pub fn is_monitor_rates(self) -> bool {
        matches!(self, Self::RealTimeData | Self::RawData | Self::DoseRateDb)
    }
}
