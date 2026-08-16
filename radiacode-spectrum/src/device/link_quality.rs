use egui::Color32;

use crate::theme::ACCENT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkQuality {
    Excellent,
    Good,
    Fair,
    Weak,
    Poor,
}

impl LinkQuality {
    pub fn from_rssi(rssi_dbm: i16) -> Self {
        if rssi_dbm >= -55 {
            Self::Excellent
        } else if rssi_dbm >= -65 {
            Self::Good
        } else if rssi_dbm >= -75 {
            Self::Fair
        } else if rssi_dbm >= -85 {
            Self::Weak
        } else {
            Self::Poor
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Excellent => "Excellent",
            Self::Good => "Good",
            Self::Fair => "Fair",
            Self::Weak => "Weak",
            Self::Poor => "Poor",
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            Self::Excellent => Color32::from_rgb(110, 190, 120),
            Self::Good | Self::Fair => ACCENT,
            Self::Weak => Color32::from_rgb(230, 170, 70),
            Self::Poor => Color32::from_rgb(220, 90, 90),
        }
    }

    pub fn bars(self) -> u8 {
        match self {
            Self::Excellent => 4,
            Self::Good => 3,
            Self::Fair => 2,
            Self::Weak => 1,
            Self::Poor => 0,
        }
    }
}
