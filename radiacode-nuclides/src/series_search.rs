use crate::series::chain_series;
use crate::topology::topology_display_name;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChainFilters {
    pub query: String,
}

pub fn search_chains(filters: &ChainFilters) -> Vec<usize> {
    chain_series()
        .iter()
        .enumerate()
        .filter(|(_, series)| matches_query(series, &filters.query))
        .map(|(index, _)| index)
        .collect()
}

fn matches_query(series: &crate::series::ChainSeries, query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return true;
    }
    let lower = trimmed.to_ascii_lowercase();
    series.name.to_ascii_lowercase().contains(&lower)
        || series.family.to_ascii_lowercase().contains(&lower)
        || topology_display_name(series.head)
            .to_ascii_lowercase()
            .contains(&lower)
        || series
            .head
            .mass_number()
            .to_string()
            .contains(trimmed)
}
