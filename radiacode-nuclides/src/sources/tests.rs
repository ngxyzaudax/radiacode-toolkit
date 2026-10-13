use std::collections::HashSet;

use super::{
    SourceCategory, detectable_lines, filter_sources, gamma_sources, source_by_key,
    source_for_nuclide, source_half_life_secs, source_lines, sources_in_category,
};
use crate::catalog::nuclide_by_id;
use crate::model::NuclideId;

fn strongest_energy(key: &str) -> f64 {
    let source = source_by_key(key).expect("source");
    detectable_lines(source)[0].line.energy_kev
}

fn has_line_near(key: &str, energy_kev: f64) -> bool {
    let source = source_by_key(key).expect("source");
    detectable_lines(source)
        .iter()
        .any(|line| (line.line.energy_kev - energy_kev).abs() < 1.0)
}

#[test]
fn library_keys_are_unique_and_curated() {
    let keys: HashSet<&str> = gamma_sources().map(|source| source.key).collect();
    assert_eq!(keys.len(), gamma_sources().count());
    assert!((35..=45).contains(&keys.len()));
}

#[test]
fn every_member_exists_in_catalog_or_topology() {
    for source in gamma_sources() {
        for member in source.components.iter().flat_map(|part| part.members) {
            let known = nuclide_by_id(member.nuclide).is_some()
                || crate::topology::topology_entry(member.nuclide).is_some();
            assert!(known, "{} member {:?}", source.key, member.nuclide);
        }
    }
}

#[test]
fn every_source_has_detectable_lines_and_half_life() {
    for source in gamma_sources() {
        assert!(!detectable_lines(source).is_empty(), "{}", source.key);
        assert!(source_half_life_secs(source).is_some(), "{}", source.key);
    }
}

#[test]
fn every_category_has_sources() {
    for category in SourceCategory::ALL {
        assert!(
            sources_in_category(category).count() > 0,
            "{}",
            category.label()
        );
    }
}

#[test]
fn signature_lines_match_reference_values() {
    assert!((strongest_energy("cs137") - 661.7).abs() < 0.5);
    assert!((strongest_energy("am241") - 59.5).abs() < 0.5);
    assert!((strongest_energy("k40") - 1460.8).abs() < 0.5);
    assert!(has_line_near("th232-series", 2614.5));
    assert!(has_line_near("th232-series", 583.2));
    assert!(has_line_near("u238-series", 609.3));
    assert!(has_line_near("u238-series", 185.7));
    assert!(has_line_near("refined-uranium", 1001.0));
    assert!(has_line_near("na22", 511.0));
    assert!(has_line_near("ru106", 621.9));
}

#[test]
fn refined_uranium_lacks_radium_daughters() {
    assert!(!has_line_near("refined-uranium", 609.3));
    assert!(!has_line_near("refined-uranium", 351.9));
}

#[test]
fn thallium_lines_follow_branching_ratio() {
    let source = source_by_key("th232-series").expect("thorium");
    let tl208 = source_lines(source)
        .into_iter()
        .find(|line| (line.line.energy_kev - 2614.5).abs() < 0.5)
        .expect("2614 keV");
    assert!((tl208.scaled_intensity_pct - 99.75 * 0.3594).abs() < 0.5);
}

#[test]
fn query_matches_names_items_and_members() {
    assert!(filter_sources("").len() == gamma_sources().count());
    let mantle = filter_sources("lantern");
    assert!(mantle.iter().any(|source| source.key == "th232-series"));
    let bismuth = filter_sources("bi-214");
    assert!(bismuth.iter().any(|source| source.key == "radon-progeny"));
    assert!(filter_sources("no such source").is_empty());
}

#[test]
fn nuclide_lookup_prefers_head_then_member() {
    let cs137 = NuclideId::new(55, 82, 0);
    assert_eq!(
        source_for_nuclide(cs137).map(|source| source.key),
        Some("cs137")
    );
    let bi214 = NuclideId::new(83, 131, 0);
    assert!(source_for_nuclide(bi214).is_some());
    let ra226 = NuclideId::new(88, 138, 0);
    assert_eq!(
        source_for_nuclide(ra226).map(|source| source.key),
        Some("ra226")
    );
    assert!(source_for_nuclide(NuclideId::new(1, 2, 0)).is_none());
}
