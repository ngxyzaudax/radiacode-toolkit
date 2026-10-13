use std::collections::HashMap;
use std::sync::OnceLock;

use super::library::gamma_sources;
use super::model::{GammaSource, SourceComponent, SourceMember};
use crate::catalog::nuclide_by_id;
use crate::chain_lines::AttributedLine;
use crate::topology::topology_half_life_secs;

pub const MIN_VISIBLE_ENERGY_KEV: f64 = 20.0;
pub const MIN_VISIBLE_INTENSITY_PCT: f64 = 0.5;

static DETECTABLE_LINES: OnceLock<HashMap<&'static str, Vec<AttributedLine>>> = OnceLock::new();

pub fn source_lines(source: &GammaSource) -> Vec<AttributedLine> {
    let mut lines: Vec<AttributedLine> =
        source.components.iter().flat_map(component_lines).collect();
    lines.sort_by(|left, right| {
        right
            .scaled_intensity_pct
            .total_cmp(&left.scaled_intensity_pct)
    });
    lines
}

pub fn detectable_lines(source: &GammaSource) -> &'static [AttributedLine] {
    DETECTABLE_LINES
        .get_or_init(build_detectable_lines)
        .get(source.key)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

pub fn is_detectable(line: &AttributedLine) -> bool {
    line.line.energy_kev >= MIN_VISIBLE_ENERGY_KEV
        && line.scaled_intensity_pct >= MIN_VISIBLE_INTENSITY_PCT
}

pub fn source_half_life_secs(source: &GammaSource) -> Option<f64> {
    topology_half_life_secs(source.head())
        .or_else(|| nuclide_by_id(source.head()).and_then(|nuclide| nuclide.half_life_secs))
}

fn build_detectable_lines() -> HashMap<&'static str, Vec<AttributedLine>> {
    gamma_sources()
        .map(|source| {
            let lines = source_lines(source)
                .into_iter()
                .filter(is_detectable)
                .collect();
            (source.key, lines)
        })
        .collect()
}

fn component_lines(component: &SourceComponent) -> Vec<AttributedLine> {
    component
        .members
        .iter()
        .flat_map(|member| member_lines(member, component.activity_ratio))
        .collect()
}

fn member_lines(member: &SourceMember, activity_ratio: f64) -> Vec<AttributedLine> {
    let Some(nuclide) = nuclide_by_id(member.nuclide) else {
        return Vec::new();
    };
    let scale = member.weight * activity_ratio;
    nuclide
        .gammas
        .iter()
        .map(|line| AttributedLine {
            line: line.clone(),
            source: member.nuclide,
            source_name: nuclide.display_name.clone(),
            scaled_intensity_pct: line.intensity_pct * scale,
        })
        .collect()
}
