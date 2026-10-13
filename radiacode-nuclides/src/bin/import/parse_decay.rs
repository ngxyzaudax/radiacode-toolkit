use radiacode_nuclides::{DecayBranch, DecayMode, GammaLine, NuclideId, RadiationKind};

use super::parse::parse_row_f64;
use super::{CsvRow, MAX_GAMMA_ENERGY, MIN_GAMMA_INTENSITY};

const DEFAULT_BRANCHING_PCT: f64 = 100.0;
const LEVEL_ENERGY_TOLERANCE_KEV: f64 = 0.001;
const GROUND_STATE_DECAY_COLUMNS: [(&str, &str); 3] = [
    ("decay_1", "decay_1_%"),
    ("decay_2", "decay_2_%"),
    ("decay_3", "decay_3_%"),
];

pub fn parse_decay_branches(row: &CsvRow, parent: NuclideId) -> Vec<DecayBranch> {
    GROUND_STATE_DECAY_COLUMNS
        .iter()
        .filter_map(|(mode_key, branch_key)| parse_ground_branch(row, parent, mode_key, branch_key))
        .collect()
}

fn parse_ground_branch(
    row: &CsvRow,
    parent: NuclideId,
    mode_key: &str,
    branch_key: &str,
) -> Option<DecayBranch> {
    let mode = parse_decay_mode(row.get(mode_key)?);
    if mode == DecayMode::Unknown {
        return None;
    }
    Some(DecayBranch {
        mode,
        branching_pct: parse_row_f64(row, branch_key).unwrap_or(DEFAULT_BRANCHING_PCT),
        daughter: daughter_from_mode(parent, mode)?,
    })
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
    row: &CsvRow,
    kind: RadiationKind,
    parent_level_energy_kev: f64,
) -> Option<GammaLine> {
    let parent_energy = parse_row_f64(row, "p_energy").unwrap_or(0.0);
    if (parent_energy - parent_level_energy_kev).abs() > LEVEL_ENERGY_TOLERANCE_KEV {
        return None;
    }
    let energy = parse_row_f64(row, "energy")?;
    if !energy.is_finite() || energy <= 0.0 || energy > MAX_GAMMA_ENERGY {
        return None;
    }
    let intensity = parse_row_f64(row, "intensity").unwrap_or(0.0);
    if intensity < MIN_GAMMA_INTENSITY {
        return None;
    }
    Some(GammaLine {
        energy_kev: energy,
        intensity_pct: intensity,
        decay: row
            .get("decay")
            .map(|value| parse_decay_mode(value))
            .unwrap_or(DecayMode::Unknown),
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
    match upper.as_str() {
        "P" => DecayMode::Proton,
        "N" => DecayMode::Neutron,
        _ => DecayMode::Unknown,
    }
}
