use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NuclideId {
    pub z: u8,
    pub n: u16,
    pub metastable: u8,
}

impl NuclideId {
    pub fn new(z: u8, n: u16, metastable: u8) -> Self {
        Self { z, n, metastable }
    }

    pub fn mass_number(self) -> u16 {
        self.z as u16 + self.n
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecayMode {
    Alpha,
    BetaMinus,
    BetaPlus,
    ElectronCapture,
    Isomeric,
    SpontaneousFission,
    Proton,
    Neutron,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayBranch {
    pub mode: DecayMode,
    pub branching_pct: f64,
    pub daughter: NuclideId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RadiationKind {
    Gamma,
    XRay,
}

impl RadiationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Gamma => "γ",
            Self::XRay => "X",
        }
    }
}

fn default_radiation_kind() -> RadiationKind {
    RadiationKind::Gamma
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GammaLine {
    pub energy_kev: f64,
    pub intensity_pct: f64,
    pub decay: DecayMode,
    #[serde(default = "default_radiation_kind")]
    pub kind: RadiationKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nuclide {
    pub id: NuclideId,
    pub symbol: String,
    pub display_name: String,
    pub mass_number: u16,
    pub half_life_secs: Option<f64>,
    pub half_life_text: String,
    pub decays: Vec<DecayBranch>,
    pub gammas: Vec<GammaLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub version: u32,
    pub nuclides: Vec<Nuclide>,
}
