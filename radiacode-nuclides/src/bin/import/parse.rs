use radiacode_nuclides::NuclideId;

use super::parse_decay::parse_decay_branches;
use super::{Candidate, CsvRow};

const GROUND_STATE_TOLERANCE_KEV: f64 = 0.001;

pub fn parse_ground_states(rows: &[CsvRow]) -> Vec<Candidate> {
    rows.iter().filter_map(parse_ground_row).collect()
}

fn parse_ground_row(row: &CsvRow) -> Option<Candidate> {
    let energy = parse_row_f64(row, "energy").unwrap_or(0.0);
    if energy.abs() > GROUND_STATE_TOLERANCE_KEV {
        return None;
    }
    let (z, n) = parse_nuclide_zn(row, "z", "n").filter(|(z, _)| *z != 0)?;
    let symbol = row.get("symbol")?.trim().to_string();
    if symbol.is_empty() {
        return None;
    }
    let id = NuclideId::new(z, n, 0);
    Some(Candidate {
        id,
        symbol,
        mass_number: id.mass_number(),
        half_life_secs: parse_half_life_secs(row),
        half_life_text: half_life_text_from_row(row),
        decays: parse_decay_branches(row, id),
        force_include: false,
        level_energy_kev: 0.0,
    })
}

pub fn half_life_text_from_row(row: &CsvRow) -> String {
    row.get("half_life")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn parse_half_life_secs(row: &CsvRow) -> Option<f64> {
    parse_row_f64(row, "half_life_sec").filter(|value| value.is_finite() && *value > 0.0)
}

pub fn parse_row_f64(row: &CsvRow, key: &str) -> Option<f64> {
    row.get(key)
        .and_then(|value| value.trim().parse::<f64>().ok())
}

pub fn parse_nuclide_zn(row: &CsvRow, z_key: &str, n_key: &str) -> Option<(u8, u16)> {
    let z = parse_u8(row.get(z_key)?)?;
    let n = parse_u16(row.get(n_key)?)?;
    Some((z, n))
}

pub fn parse_parent_level(row: &CsvRow) -> Option<(u8, u16, f64)> {
    let (z, n) = parse_nuclide_zn(row, "p_z", "p_n")?;
    let energy_kev = parse_row_f64(row, "p_energy")?;
    Some((z, n, energy_kev))
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
