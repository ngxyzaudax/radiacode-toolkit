use radiacode_nuclides::{PeakIdentification, best_match};

pub fn peak_label(identification: &PeakIdentification) -> String {
    best_match(identification)
        .map(|candidate| candidate.display_name.clone())
        .unwrap_or_else(|| format!("{:.0} keV", identification.peak.energy_kev))
}
