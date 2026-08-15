use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use radiacode_nuclides::{
    Catalog, DecayBranch, DecayMode, GammaLine, Nuclide, NuclideId, RadiationKind,
};

const API_BASE: &str = "https://nds.iaea.org/relnsd/v1/data";
const USER_AGENT: &str = "radiacode-nuclides/0.1 (research; contact: local)";
const MIN_HALF_LIFE_SECS: f64 = 1.0;
const MIN_GAMMA_INTENSITY: f64 = 0.05;
const MAX_GAMMA_ENERGY: f64 = 4000.0;
const REQUEST_DELAY_MS: u64 = 60;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let limit = import_limit();
    let output = data_path();
    println!("Fetching ground states from IAEA Livechart...");
    let ground_rows = fetch_csv(&format!("{API_BASE}?fields=ground_states&nuclides=all"))?;
    let mut candidates = parse_ground_states(&ground_rows);
    println!("Parsed {} ground-state rows", candidates.len());
    force_chain_members(&mut candidates);
    let selected = select_nuclides(&candidates);
    println!(
        "Selected {} nuclide candidates{}",
        selected.len(),
        limit
            .map(|value| format!(", import limit {value}"))
            .unwrap_or_default()
    );
    let mut nuclides = Vec::new();
    for (index, candidate) in selected.iter().enumerate() {
        if limit.is_some_and(|cap| nuclides.len() >= cap) {
            break;
        }
        if index > 0 {
            thread::sleep(Duration::from_millis(REQUEST_DELAY_MS));
        }
        let slug = nuclide_slug(candidate.id);
        let radiations = fetch_radiations(&slug);
        if radiations.is_empty() && !candidate.force_include {
            continue;
        }
        nuclides.push(build_nuclide(candidate, radiations));
        if index % 50 == 0 {
            println!(
                "Fetched {index}/{} ({slug}) -> {} kept",
                selected.len(),
                nuclides.len()
            );
        }
    }
    nuclides.sort_by(|left, right| {
        left.display_name
            .cmp(&right.display_name)
            .then_with(|| left.mass_number.cmp(&right.mass_number))
    });
    ensure_critical_nuclides(&mut nuclides, &candidates);
    let catalog = Catalog {
        version: 1,
        nuclides,
    };
    fs::write(&output, serde_json::to_vec_pretty(&catalog)?)?;
    println!(
        "Wrote {} nuclides to {}",
        catalog.nuclides.len(),
        output.display()
    );
    Ok(())
}

fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/nuclides.json")
}

fn import_limit() -> Option<usize> {
    std::env::args().skip(1).find_map(|arg| {
        arg.strip_prefix("--limit=")
            .and_then(|value| value.parse::<usize>().ok())
    })
}

#[derive(Clone)]
struct Candidate {
    id: NuclideId,
    symbol: String,
    mass_number: u16,
    half_life_secs: Option<f64>,
    half_life_text: String,
    decays: Vec<DecayBranch>,
    force_include: bool,
}

fn fetch_csv(url: &str) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
    let response = ureq::get(url)
        .header("User-Agent", USER_AGENT)
        .call()?;
    let body = response.into_body().read_to_string()?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(body.as_bytes());
    let headers = reader.headers()?.clone();
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record?;
        let mut map = HashMap::new();
        for (index, field) in record.iter().enumerate() {
            if let Some(name) = headers.get(index) {
                map.insert(name.to_string(), field.to_string());
            }
        }
        rows.push(map);
    }
    Ok(rows)
}

fn parse_ground_states(rows: &[HashMap<String, String>]) -> Vec<Candidate> {
    rows.iter()
        .filter_map(parse_ground_row)
        .collect()
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
    [("decay_1", "decay_1_%"), ("decay_2", "decay_2_%"), ("decay_3", "decay_3_%")]
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

fn fetch_radiations(slug: &str) -> Vec<GammaLine> {
    let gamma_url = format!("{API_BASE}?fields=decay_rads&rad_types=g&nuclides={slug}");
    let xray_url = format!("{API_BASE}?fields=decay_rads&rad_types=x&nuclides={slug}");
    let mut radiations = parse_gamma_rows(&fetch_csv(&gamma_url).unwrap_or_default());
    radiations.extend(parse_xray_rows(&fetch_csv(&xray_url).unwrap_or_default()));
    radiations
}

fn parse_gamma_rows(rows: &[HashMap<String, String>]) -> Vec<GammaLine> {
    rows.iter()
        .filter_map(|row| parse_radiation_row(row, RadiationKind::Gamma))
        .collect()
}

fn parse_xray_rows(rows: &[HashMap<String, String>]) -> Vec<GammaLine> {
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

fn select_nuclides(candidates: &[Candidate]) -> Vec<Candidate> {
    let mut selected = candidates
        .iter()
        .filter(|candidate| {
            candidate.force_include
                || candidate.half_life_secs.is_none()
                || candidate.half_life_secs.unwrap_or(0.0) >= MIN_HALF_LIFE_SECS
        })
        .cloned()
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        let left_rank = fetch_rank(left);
        let right_rank = fetch_rank(right);
        left_rank.cmp(&right_rank)
    });
    selected
}

fn fetch_rank(candidate: &Candidate) -> u8 {
    if candidate.force_include {
        return 0;
    }
    if candidate.half_life_secs.is_none() {
        return 2;
    }
    1
}

fn force_chain_members(candidates: &mut [Candidate]) {
    let forced = chain_closure_ids(candidates);
    for candidate in candidates.iter_mut() {
        if forced.contains(&candidate.id) {
            candidate.force_include = true;
        }
    }
}

fn chain_closure_ids(candidates: &[Candidate]) -> HashSet<NuclideId> {
    let by_id: HashMap<NuclideId, &Candidate> = candidates
        .iter()
        .map(|candidate| (candidate.id, candidate))
        .collect();
    let seeds = chain_seed_ids();
    let mut forced = HashSet::new();
    let mut queue: VecDeque<NuclideId> = seeds.into_iter().collect();
    while let Some(id) = queue.pop_front() {
        if !forced.insert(id) {
            continue;
        }
        let Some(candidate) = by_id.get(&id) else {
            continue;
        };
        for branch in &candidate.decays {
            queue.push_back(branch.daughter);
        }
    }
    forced
}

fn chain_seed_ids() -> HashSet<NuclideId> {
    [
        (92, 146),
        (92, 143),
        (90, 144),
        (91, 143),
        (55, 82),
        (19, 21),
        (27, 33),
        (95, 146),
    ]
    .into_iter()
    .map(|(z, n)| NuclideId::new(z, n, 0))
    .collect()
}

fn ensure_critical_nuclides(nuclides: &mut Vec<Nuclide>, candidates: &[Candidate]) {
    let critical = chain_closure_ids(candidates);
    for id in critical {
        if nuclides.iter().any(|nuclide| nuclide.id == id) {
            continue;
        }
        let Some(candidate) = candidates.iter().find(|entry| entry.id == id) else {
            continue;
        };
        thread::sleep(Duration::from_millis(REQUEST_DELAY_MS));
        let slug = nuclide_slug(id);
        let radiations = fetch_radiations(&slug);
        nuclides.push(build_nuclide(candidate, radiations));
        println!("Ensured critical nuclide {slug}");
    }
    nuclides.sort_by(|left, right| {
        left.display_name
            .cmp(&right.display_name)
            .then_with(|| left.mass_number.cmp(&right.mass_number))
    });
}

fn build_nuclide(candidate: &Candidate, mut gammas: Vec<GammaLine>) -> Nuclide {
    gammas.sort_by(|left, right| {
        left.energy_kev
            .partial_cmp(&right.energy_kev)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let display_name = format!("{}-{}", candidate.symbol, candidate.mass_number);
    Nuclide {
        id: candidate.id,
        symbol: candidate.symbol.clone(),
        display_name,
        mass_number: candidate.mass_number,
        half_life_secs: candidate.half_life_secs,
        half_life_text: candidate.half_life_text.clone(),
        decays: candidate.decays.clone(),
        gammas,
    }
}

fn nuclide_slug(id: NuclideId) -> String {
    format!(
        "{}{}",
        id.mass_number(),
        element_symbol(id.z).to_ascii_lowercase()
    )
}

fn element_symbol(z: u8) -> &'static str {
    match z {
        1 => "H",
        2 => "He",
        3 => "Li",
        4 => "Be",
        5 => "B",
        6 => "C",
        7 => "N",
        8 => "O",
        9 => "F",
        10 => "Ne",
        11 => "Na",
        12 => "Mg",
        13 => "Al",
        14 => "Si",
        15 => "P",
        16 => "S",
        17 => "Cl",
        18 => "Ar",
        19 => "K",
        20 => "Ca",
        21 => "Sc",
        22 => "Ti",
        23 => "V",
        24 => "Cr",
        25 => "Mn",
        26 => "Fe",
        27 => "Co",
        28 => "Ni",
        29 => "Cu",
        30 => "Zn",
        31 => "Ga",
        32 => "Ge",
        33 => "As",
        34 => "Se",
        35 => "Br",
        36 => "Kr",
        37 => "Rb",
        38 => "Sr",
        39 => "Y",
        40 => "Zr",
        41 => "Nb",
        42 => "Mo",
        43 => "Tc",
        44 => "Ru",
        45 => "Rh",
        46 => "Pd",
        47 => "Ag",
        48 => "Cd",
        49 => "In",
        50 => "Sn",
        51 => "Sb",
        52 => "Te",
        53 => "I",
        54 => "Xe",
        55 => "Cs",
        56 => "Ba",
        57 => "La",
        58 => "Ce",
        59 => "Pr",
        60 => "Nd",
        61 => "Pm",
        62 => "Sm",
        63 => "Eu",
        64 => "Gd",
        65 => "Tb",
        66 => "Dy",
        67 => "Ho",
        68 => "Er",
        69 => "Tm",
        70 => "Yb",
        71 => "Lu",
        72 => "Hf",
        73 => "Ta",
        74 => "W",
        75 => "Re",
        76 => "Os",
        77 => "Ir",
        78 => "Pt",
        79 => "Au",
        80 => "Hg",
        81 => "Tl",
        82 => "Pb",
        83 => "Bi",
        84 => "Po",
        85 => "At",
        86 => "Rn",
        87 => "Fr",
        88 => "Ra",
        89 => "Ac",
        90 => "Th",
        91 => "Pa",
        92 => "U",
        93 => "Np",
        94 => "Pu",
        95 => "Am",
        _ => "X",
    }
}

fn parse_decay_mode(value: &str) -> DecayMode {
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
