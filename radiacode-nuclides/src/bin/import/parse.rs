use std::collections::HashMap;

use radiacode_nuclides::{DecayBranch, DecayMode, GammaLine, NuclideId, RadiationKind};

use super::{Candidate, MAX_GAMMA_ENERGY, MIN_GAMMA_INTENSITY};

pub fn parse_ground_states(rows: &[HashMap<String, String>]) -> Vec<Candidate> {
    rows.iter().filter_map(parse_ground_row).collect()
}

fn parse_ground_row(row: &HashMap<String, String>) -> Option<Candidate> {
    let energy = row
        .get("energy")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);
    if energy.abs() > 0.001 {
        return None;
    }
    let z = parse_u8(row.get("z")?)?;
    let n = parse_u16(row.get("n")?)?;
    if z == 0 {
        return None;
    }
    let symbol = row.get("symbol")?.trim().to_string();
    if symbol.is_empty() {
        return None;
    }
    let id = NuclideId::new(z, n, 0);
    let decays = parse_decay_branches(row, id);
    Some(Candidate {
        id,
        symbol,
        mass_number: id.mass_number(),
        half_life_secs: parse_half_life_secs(row),
        half_life_text: half_life_text_from_row(row),
        decays,
        force_include: false,
        level_energy_kev: 0.0,
    })
}

pub fn half_life_text_from_row(row: &HashMap<String, String>) -> String {
    row.get("half_life")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "unknown".to_string())
}

fn parse_half_life_secs(row: &HashMap<String, String>) -> Option<f64> {
    row.get("half_life_sec")
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0)
}

pub fn parse_decay_branches(row: &HashMap<String, String>, parent: NuclideId) -> Vec<DecayBranch> {
    [
        ("decay_1", "decay_1_%"),
        ("decay_2", "decay_2_%"),
        ("decay_3", "decay_3_%"),
    ]
    .iter()
    .filter_map(|(mode_key, branch_key)| {
        let mode_text = row.get(*mode_key)?;
        if mode_text.trim().is_empty() {
            return None;
        }
        let mode = parse_decay_mode(mode_text);
        if mode == DecayMode::Unknown {
            return None;
        }
        let branching = row
            .get(*branch_key)
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(100.0);
        let daughter = daughter_from_mode(parent, mode)?;
        Some(DecayBranch {
            mode,
            branching_pct: branching,
            daughter,
        })
    })
    .collect()
}

pub fn daughter_from_mode(parent: NuclideId, mode: DecayMode) -> Option<NuclideId> {
    let z = i16::from(parent.z);
    let n = parent.n as i16;
    let (next_z, next_n) = match mode {
        DecayMode::Alpha if z >= 2 && n >= 2 => (z - 2, n - 2),
        DecayMode::BetaMinus => (z + 1, n - 1),
        DecayMode::BetaPlus | DecayMode::ElectronCapture if z >= 1 => (z - 1, n + 1),
        DecayMode::Isomeric => (z, n),
        DecayMode::Proton if z >= 1 => (z - 1, n),
        DecayMode::Neutron if n >= 1 => (z, n - 1),
        _ => return None,
    };
    if next_z <= 0 || next_n < 0 {
        return None;
    }
    Some(NuclideId::new(next_z as u8, next_n as u16, 0))
}

pub fn parse_radiation_row(
    row: &HashMap<String, String>,
    kind: RadiationKind,
    parent_level_energy_kev: f64,
) -> Option<GammaLine> {
    let parent_energy = row
        .get("p_energy")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);
    if (parent_energy - parent_level_energy_kev).abs() > 0.001 {
        return None;
    }
    let energy = row.get("energy")?.parse::<f64>().ok()?;
    if !energy.is_finite() || energy <= 0.0 || energy > MAX_GAMMA_ENERGY {
        return None;
    }
    let intensity = row
        .get("intensity")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);
    if intensity < MIN_GAMMA_INTENSITY {
        return None;
    }
    let decay = row
        .get("decay")
        .map(|value| parse_decay_mode(value))
        .unwrap_or(DecayMode::Unknown);
    Some(GammaLine {
        energy_kev: energy,
        intensity_pct: intensity,
        decay,
        kind,
    })
}

pub fn parse_decay_mode(value: &str) -> DecayMode {
    let upper = value.trim().to_ascii_uppercase();
    if upper.contains("IT") || upper.contains("ISOMERIC") {
        return DecayMode::Isomeric;
    }
    if upper.contains("SF") {
        return DecayMode::SpontaneousFission;
    }
    if upper == "A" || upper.starts_with("ALPHA") {
        return DecayMode::Alpha;
    }
    if upper.contains("EC") {
        return DecayMode::ElectronCapture;
    }
    if upper.contains("B+") || upper.contains("BP") {
        return DecayMode::BetaPlus;
    }
    if upper.contains("B-") || upper.contains("BM") || upper.starts_with("2B") {
        return DecayMode::BetaMinus;
    }
    if upper == "P" {
        return DecayMode::Proton;
    }
    if upper == "N" {
        return DecayMode::Neutron;
    }
    DecayMode::Unknown
}

pub fn display_name(symbol: &str, mass_number: u16, metastable: u8) -> String {
    match metastable {
        0 => format!("{symbol}-{mass_number}"),
        1 => format!("{symbol}-{mass_number}m"),
        index => format!("{symbol}-{mass_number}m{index}"),
    }
}

pub fn parse_u8(value: &str) -> Option<u8> {
    value.trim().parse::<u8>().ok()
}

pub fn parse_u16(value: &str) -> Option<u16> {
    value.trim().parse::<u16>().ok()
}
