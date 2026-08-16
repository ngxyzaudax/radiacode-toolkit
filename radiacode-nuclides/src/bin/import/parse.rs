use std::collections::HashMap;

use radiacode_nuclides::{DecayBranch, DecayMode, GammaLine, NuclideId, RadiationKind};

use super::{Candidate, MAX_GAMMA_ENERGY, MIN_GAMMA_INTENSITY};

pub fn parse_ground_states(rows: &[HashMap<String, String>]) -> Vec<Candidate> {
    rows.iter().filter_map(parse_ground_row).collect()
}

fn parse_ground_row(row: &HashMap<String, String>) -> Option<Candidate> {
    let z = parse_u8(row.get("z")?)?;
    let n = parse_u16(row.get("n")?)?;
    if z == 0 {
        return None;
    }
    let symbol = row.get("symbol")?.trim().to_string();
    if symbol.is_empty() {
        return None;
    }
    let half_life_text = row
        .get("half_life")
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "stable".to_string());
    let half_life_secs = row
        .get("half_life_sec")
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0);
    let metastable = row
        .get("energy_shift")
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(0);
    let id = NuclideId::new(z, n, metastable);
    let decays = parse_decay_branches(row, id);
    Some(Candidate {
        id,
        symbol,
        mass_number: id.mass_number(),
        half_life_secs,
        half_life_text,
        decays,
        force_include: false,
    })
}

fn parse_decay_branches(row: &HashMap<String, String>, parent: NuclideId) -> Vec<DecayBranch> {
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

fn daughter_from_mode(parent: NuclideId, mode: DecayMode) -> Option<NuclideId> {
    let z = i16::from(parent.z);
    let n = parent.n as i16;
    let (next_z, next_n) = match mode {
        DecayMode::Alpha if z >= 2 && n >= 2 => (z - 2, n - 2),
        DecayMode::BetaMinus => (z + 1, n - 1),
        DecayMode::BetaPlus | DecayMode::ElectronCapture if z >= 1 => (z - 1, n + 1),
        DecayMode::Proton if z >= 1 => (z - 1, n),
        DecayMode::Neutron if n >= 1 => (z, n - 1),
        _ => return None,
    };
    if next_z <= 0 || next_n < 0 {
        return None;
    }
    Some(NuclideId::new(next_z as u8, next_n as u16, 0))
}

pub fn parse_gamma_rows(rows: &[HashMap<String, String>]) -> Vec<GammaLine> {
    rows.iter()
        .filter_map(|row| parse_radiation_row(row, RadiationKind::Gamma))
        .collect()
}

pub fn parse_xray_rows(rows: &[HashMap<String, String>]) -> Vec<GammaLine> {
    rows.iter()
        .filter_map(|row| parse_radiation_row(row, RadiationKind::XRay))
        .collect()
}

fn parse_radiation_row(row: &HashMap<String, String>, kind: RadiationKind) -> Option<GammaLine> {
    let parent_energy = row
        .get("p_energy")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);
    if parent_energy.abs() > 0.001 {
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
    match value.trim().to_ascii_uppercase().as_str() {
        "A" | "ALPHA" => DecayMode::Alpha,
        "B-" | "BM" | "BETA-" => DecayMode::BetaMinus,
        "B+" | "BP" | "BETA+" => DecayMode::BetaPlus,
        "EC" | "EPSILON" => DecayMode::ElectronCapture,
        "IT" | "ISOMERIC" => DecayMode::Isomeric,
        "SF" => DecayMode::SpontaneousFission,
        "P" => DecayMode::Proton,
        "N" => DecayMode::Neutron,
        _ => DecayMode::Unknown,
    }
}

fn parse_u8(value: &str) -> Option<u8> {
    value.trim().parse::<u8>().ok()
}

fn parse_u16(value: &str) -> Option<u16> {
    value.trim().parse::<u16>().ok()
}
