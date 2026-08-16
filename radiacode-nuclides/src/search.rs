use crate::catalog::catalog;
use crate::model::Nuclide;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SearchFilters {
    pub query: String,
    pub min_half_life_secs: Option<f64>,
    pub max_half_life_secs: Option<f64>,
}

pub fn search_nuclides(filters: &SearchFilters) -> Vec<usize> {
    catalog()
        .nuclides
        .iter()
        .enumerate()
        .filter(|(_, nuclide)| matches_filters(nuclide, filters))
        .map(|(index, _)| index)
        .collect()
}

fn matches_filters(nuclide: &Nuclide, filters: &SearchFilters) -> bool {
    query_matches(nuclide, &filters.query) && half_life_matches(nuclide, filters)
}

fn query_matches(nuclide: &Nuclide, query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return true;
    }
    let lower = trimmed.to_ascii_lowercase();
    nuclide.display_name.to_ascii_lowercase().contains(&lower)
        || nuclide.symbol.to_ascii_lowercase().contains(&lower)
        || nuclide.mass_number.to_string().contains(trimmed)
}

fn half_life_matches(nuclide: &Nuclide, filters: &SearchFilters) -> bool {
    let Some(secs) = nuclide.half_life_secs else {
        return filters.min_half_life_secs.is_none() && filters.max_half_life_secs.is_none();
    };
    if let Some(min) = filters.min_half_life_secs
        && secs < min
    {
        return false;
    }
    if let Some(max) = filters.max_half_life_secs
        && secs > max
    {
        return false;
    }
    true
}
