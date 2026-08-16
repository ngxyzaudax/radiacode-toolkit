use crate::catalog::nuclide_by_id;
use crate::model::{DecayMode, NuclideId};

#[derive(Debug, Clone, PartialEq)]
pub struct ChainStep {
    pub nuclide_id: NuclideId,
    pub display_name: String,
    pub half_life_text: String,
    pub decay_mode: Option<DecayMode>,
    pub branching_pct: Option<f64>,
}

pub fn decay_chain(head: NuclideId, max_steps: usize) -> Vec<ChainStep> {
    let mut chain = Vec::new();
    let mut current = Some(head);
    for _ in 0..max_steps {
        let Some(id) = current else {
            break;
        };
        let Some(nuclide) = nuclide_by_id(id) else {
            break;
        };
        if chain.iter().any(|step: &ChainStep| step.nuclide_id == id) {
            break;
        }
        let primary = primary_decay(nuclide.decays.as_slice());
        chain.push(ChainStep {
            nuclide_id: id,
            display_name: nuclide.display_name.clone(),
            half_life_text: nuclide.half_life_text.clone(),
            decay_mode: primary.map(|branch| branch.mode),
            branching_pct: primary.map(|branch| branch.branching_pct),
        });
        current = primary.map(|branch| branch.daughter);
        if nuclide.decays.is_empty() {
            break;
        }
    }
    chain
}

fn primary_decay(decays: &[crate::model::DecayBranch]) -> Option<&crate::model::DecayBranch> {
    decays.iter().max_by(|left, right| {
        left.branching_pct
            .partial_cmp(&right.branching_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

pub fn decay_mode_label(mode: DecayMode) -> &'static str {
    match mode {
        DecayMode::Alpha => "alpha",
        DecayMode::BetaMinus => "beta-",
        DecayMode::BetaPlus => "beta+",
        DecayMode::ElectronCapture => "EC",
        DecayMode::Isomeric => "IT",
        DecayMode::SpontaneousFission => "SF",
        DecayMode::Proton => "p",
        DecayMode::Neutron => "n",
        DecayMode::Unknown => "?",
    }
}

pub fn decay_branch_label(mode: DecayMode) -> &'static str {
    match mode {
        DecayMode::Alpha => "α branch",
        DecayMode::BetaMinus => "β⁻ branch",
        DecayMode::BetaPlus => "β⁺ branch",
        DecayMode::ElectronCapture => "EC branch",
        DecayMode::Isomeric => "IT branch",
        DecayMode::SpontaneousFission => "SF branch",
        DecayMode::Proton => "p branch",
        DecayMode::Neutron => "n branch",
        DecayMode::Unknown => "? branch",
    }
}
