use super::library::gamma_sources;
use super::model::GammaSource;
use crate::catalog::nuclide_by_id;

pub fn filter_sources(query: &str) -> Vec<&'static GammaSource> {
    let needle = query.trim().to_lowercase();
    gamma_sources()
        .filter(|source| needle.is_empty() || source_matches(source, &needle))
        .collect()
}

fn source_matches(source: &GammaSource, needle: &str) -> bool {
    [source.name, source.key, source.found_in]
        .iter()
        .any(|text| text.to_lowercase().contains(needle))
        || member_names(source).any(|name| name.to_lowercase().contains(needle))
}

fn member_names(source: &GammaSource) -> impl Iterator<Item = &'static str> + '_ {
    source
        .components
        .iter()
        .flat_map(|component| component.members.iter())
        .filter_map(|member| nuclide_by_id(member.nuclide))
        .map(|nuclide| nuclide.display_name.as_str())
}
