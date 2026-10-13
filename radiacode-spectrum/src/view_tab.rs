#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewTab {
    #[default]
    Device,
    Monitor,
    Spectrum,
    Spectrogram,
    Compare,
    PeakId,
    Sources,
    Settings,
    About,
}

impl ViewTab {
    pub fn label(self) -> &'static str {
        match self {
            Self::Device => "Device",
            Self::Monitor => "Monitor",
            Self::Spectrum => "Spectrum",
            Self::Spectrogram => "Spectrogram",
            Self::Compare => "Compare",
            Self::PeakId => "Peak ID",
            Self::Sources => "Sources",
            Self::Settings => "Settings",
            Self::About => "About",
        }
    }
}
