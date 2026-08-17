use radiacode_nuclides::{PeakIdentification, best_match};

use crate::peak_overlay::peak_label;

pub fn peak_hover_text(identification: &PeakIdentification) -> String {
    let name = peak_label(identification);
    let energy = identification.peak.energy_kev;
    let net_area = identification.peak.counts;
    let mut lines = vec![
        name,
        format!("{energy:.1} keV"),
        format!("Net area: {net_area:.1}"),
    ];
    if let Some(candidate) = best_match(identification) {
        lines.push(format!(
            "{:.1} keV · {:.1}% · Δ{:.2} keV",
            candidate.line_energy_kev, candidate.intensity_pct, candidate.delta_kev
        ));
    }
    lines.join("\n")
}
