#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewTab {
    #[default]
    Device,
    Monitor,
    Spectrum,
    Spectrogram,
    Dosimeter,
    Analysis,
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
            Self::Dosimeter => "Dosimeter",
            Self::Analysis => "Analysis",
            Self::Settings => "Settings",
            Self::About => "About",
        }
    }
}
