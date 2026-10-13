use crate::model::DecayMode;

pub fn decay_mode_label(mode: DecayMode) -> &'static str {
    match mode {
        DecayMode::Alpha => "alpha",
        DecayMode::BetaMinus => "beta-",
        DecayMode::BetaPlus => "beta+",
        DecayMode::ElectronCapture => "EC",
        DecayMode::Isomeric => "IT",
        DecayMode::SpontaneousFission => "SF",
        DecayMode::Proton => "p",
        DecayMode::Neutron => "n",
        DecayMode::Unknown => "?",
    }
}
