#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewTab {
    #[default]
    Device,
    Monitor,
    Spectrum,
    Spectrogram,
    Analysis,
    Catalogue,
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
            Self::Analysis => "Analysis",
            Self::Catalogue => "Catalogue",
            Self::Settings => "Settings",
            Self::About => "About",
        }
    }
}
