use super::model::{GammaSource, SourceCategory, component, member};

pub const INDUSTRIAL_SOURCES: &[GammaSource] = &[
    GammaSource {
        key: "ir192",
        name: "Iridium-192",
        category: SourceCategory::Industrial,
        found_in: "Industrial weld radiography cameras, HDR brachytherapy",
        components: &[component(&[member(77, 115, 0, 1.0)])],
    },
    GammaSource {
        key: "se75",
        name: "Selenium-75",
        category: SourceCategory::Industrial,
        found_in: "Industrial radiography of thin-walled pipes",
        components: &[component(&[member(34, 41, 0, 1.0)])],
    },
    GammaSource {
        key: "yb169",
        name: "Ytterbium-169",
        category: SourceCategory::Industrial,
        found_in: "Portable radiography of small components",
        components: &[component(&[member(70, 99, 0, 1.0)])],
    },
];

pub const FALLOUT_SOURCES: &[GammaSource] = &[
    GammaSource {
        key: "cs134",
        name: "Caesium-134",
        category: SourceCategory::Fallout,
        found_in: "Recent reactor accident fallout alongside Cs-137",
        components: &[component(&[member(55, 79, 0, 1.0)])],
    },
    GammaSource {
        key: "ag110m",
        name: "Silver-110m",
        category: SourceCategory::Fallout,
        found_in: "Reactor coolant activation product, accident fallout",
        components: &[component(&[member(47, 63, 1, 1.0)])],
    },
    GammaSource {
        key: "sb125",
        name: "Antimony-125",
        category: SourceCategory::Fallout,
        found_in: "Fission product in fallout and reprocessing effluent",
        components: &[component(&[member(51, 74, 0, 1.0)])],
    },
    GammaSource {
        key: "ru106",
        name: "Ruthenium-106 (Rh-106)",
        category: SourceCategory::Fallout,
        found_in: "Fission product; 2017 European release, eye-plaque brachytherapy",
        components: &[component(&[member(44, 62, 0, 1.0), member(45, 61, 0, 1.0)])],
    },
    GammaSource {
        key: "ce144",
        name: "Cerium-144 (Pr-144)",
        category: SourceCategory::Fallout,
        found_in: "Fission product in early fallout",
        components: &[component(&[member(58, 86, 0, 1.0), member(59, 85, 0, 1.0)])],
    },
    GammaSource {
        key: "zr95",
        name: "Zirconium-95 (Nb-95)",
        category: SourceCategory::Fallout,
        found_in: "Fission product in fresh fallout",
        components: &[component(&[member(40, 55, 0, 1.0), member(41, 54, 0, 1.0)])],
    },
    GammaSource {
        key: "eu154",
        name: "Europium-154",
        category: SourceCategory::Fallout,
        found_in: "Activation and fission product in reactor components and fuel",
        components: &[component(&[member(63, 91, 0, 1.0)])],
    },
    GammaSource {
        key: "co58",
        name: "Cobalt-58",
        category: SourceCategory::Fallout,
        found_in: "Reactor coolant corrosion product",
        components: &[component(&[member(27, 31, 0, 1.0)])],
    },
];
