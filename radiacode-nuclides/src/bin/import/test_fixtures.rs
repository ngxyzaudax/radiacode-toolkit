use radiacode_nuclides::NuclideId;

use super::bundle::RadiationBundle;
use super::{Candidate, CsvRow};

pub fn row(pairs: &[(&str, &str)]) -> CsvRow {
    pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

pub fn parent_row(extra: &[(&str, &str)]) -> CsvRow {
    let mut base = row(&[
        ("p_z", "43"),
        ("p_n", "56"),
        ("p_symbol", "Tc"),
        ("p_energy", "142.683"),
        ("half_life_sec", "21624"),
        ("half_life", "6.01 h"),
    ]);
    base.extend(row(extra));
    base
}

pub fn bundle(gamma_rows: Vec<CsvRow>, beta_minus_rows: Vec<CsvRow>) -> RadiationBundle {
    RadiationBundle {
        gamma_rows,
        xray_rows: Vec::new(),
        beta_minus_rows,
        beta_plus_rows: Vec::new(),
    }
}

pub fn candidate(id: NuclideId, level_energy_kev: f64) -> Candidate {
    Candidate {
        id,
        symbol: "Tc".into(),
        mass_number: id.mass_number(),
        half_life_secs: None,
        half_life_text: "unknown".into(),
        decays: Vec::new(),
        force_include: false,
        level_energy_kev,
    }
}
