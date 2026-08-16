#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RowKind {
    Normal,
    GapRecovery { offline_secs: f64, raw_total: u64 },
    LiveSpike { rate_factor: f32 },
}

impl RowKind {
    pub fn storage_tag(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::GapRecovery { .. } => 1,
            Self::LiveSpike { .. } => 2,
        }
    }

    pub fn from_storage_tag(tag: u8, extra: f64, raw_total: u64) -> Self {
        match tag {
            1 => Self::GapRecovery {
                offline_secs: extra,
                raw_total,
            },
            2 => Self::LiveSpike {
                rate_factor: extra as f32,
            },
            _ => Self::Normal,
        }
    }

    pub fn storage_extra(self) -> f64 {
        match self {
            Self::GapRecovery { offline_secs, .. } => offline_secs,
            Self::LiveSpike { rate_factor } => f64::from(rate_factor),
            Self::Normal => 0.0,
        }
    }
}
