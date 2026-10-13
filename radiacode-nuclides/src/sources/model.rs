use crate::model::NuclideId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceCategory {
    Natural,
    Check,
    Medical,
    Industrial,
    Fallout,
}

impl SourceCategory {
    pub const ALL: [SourceCategory; 5] = [
        Self::Natural,
        Self::Check,
        Self::Medical,
        Self::Industrial,
        Self::Fallout,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Natural => "Natural and everyday",
            Self::Check => "Check and calibration",
            Self::Medical => "Medical",
            Self::Industrial => "Industrial",
            Self::Fallout => "Fallout and reactor",
        }
    }

    pub fn prevalence(self) -> f64 {
        match self {
            Self::Natural | Self::Check => 1.0,
            Self::Medical => 0.8,
            Self::Industrial => 0.6,
            Self::Fallout => 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceMember {
    pub nuclide: NuclideId,
    pub weight: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceComponent {
    pub members: &'static [SourceMember],
    pub activity_ratio: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GammaSource {
    pub key: &'static str,
    pub name: &'static str,
    pub category: SourceCategory,
    pub found_in: &'static str,
    pub components: &'static [SourceComponent],
}

impl GammaSource {
    pub fn head(&self) -> NuclideId {
        self.components[0].members[0].nuclide
    }

    pub fn contains(&self, id: NuclideId) -> bool {
        self.components
            .iter()
            .flat_map(|component| component.members.iter())
            .any(|member| member.nuclide == id)
    }
}

pub const fn member(z: u8, n: u16, metastable: u8, weight: f64) -> SourceMember {
    SourceMember {
        nuclide: NuclideId::new(z, n, metastable),
        weight,
    }
}

pub const fn component(members: &'static [SourceMember]) -> SourceComponent {
    SourceComponent {
        members,
        activity_ratio: 1.0,
    }
}
