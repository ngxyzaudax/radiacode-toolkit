pub mod branches;
pub mod build;
pub mod bundle;
pub mod cli;
pub mod critical;
pub mod fetch;
pub mod isomer;
pub mod level_index;
pub mod output;
pub mod parse;
pub mod parse_decay;
pub mod pipeline;
pub mod pipeline_isomers;
pub mod pipeline_parents;
pub mod select;
pub mod state;
pub mod validate;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use radiacode_nuclides::{DecayBranch, NuclideId};

pub(crate) const API_BASE: &str = "https://nds.iaea.org/relnsd/v1/data";
pub(crate) const USER_AGENT: &str = "radiacode-nuclides/0.1 (research; contact: local)";
pub(crate) const MIN_HALF_LIFE_SECS: f64 = 1.0;
pub(crate) const MIN_GAMMA_INTENSITY: f64 = 0.05;
pub(crate) const MAX_GAMMA_ENERGY: f64 = 4000.0;

pub(crate) type CsvRow = HashMap<String, String>;
pub(crate) type LevelKey = (u8, u16, u64);
pub(crate) type LevelIndex = HashMap<LevelKey, u8>;

#[derive(Clone)]
pub(crate) struct Candidate {
    pub id: NuclideId,
    pub symbol: String,
    pub mass_number: u16,
    pub half_life_secs: Option<f64>,
    pub half_life_text: String,
    pub decays: Vec<DecayBranch>,
    pub force_include: bool,
    pub level_energy_kev: f64,
}
