use super::library_check::CHECK_SOURCES;
use super::library_industrial::{FALLOUT_SOURCES, INDUSTRIAL_SOURCES};
use super::library_medical::MEDICAL_SOURCES;
use super::library_natural::NATURAL_SOURCES;
use super::model::{GammaSource, SourceCategory};
use crate::model::NuclideId;

const LIBRARY: [&[GammaSource]; 5] = [
    NATURAL_SOURCES,
    CHECK_SOURCES,
    MEDICAL_SOURCES,
    INDUSTRIAL_SOURCES,
    FALLOUT_SOURCES,
];

pub fn gamma_sources() -> impl Iterator<Item = &'static GammaSource> {
    LIBRARY.iter().flat_map(|group| group.iter())
}

pub fn sources_in_category(category: SourceCategory) -> impl Iterator<Item = &'static GammaSource> {
    gamma_sources().filter(move |source| source.category == category)
}

pub fn source_by_key(key: &str) -> Option<&'static GammaSource> {
    gamma_sources().find(|source| source.key == key)
}

pub fn source_for_nuclide(id: NuclideId) -> Option<&'static GammaSource> {
    gamma_sources()
        .find(|source| source.head() == id)
        .or_else(|| gamma_sources().find(|source| source.contains(id)))
}
