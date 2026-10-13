use super::model::{GammaSource, SourceCategory, component, member};

pub const MEDICAL_SOURCES: &[GammaSource] = &[
    GammaSource {
        key: "tc99m",
        name: "Technetium-99m",
        category: SourceCategory::Medical,
        found_in: "Bone, cardiac and thyroid scans; patients for about a day after the scan",
        components: &[component(&[member(43, 56, 1, 1.0)])],
    },
    GammaSource {
        key: "f18",
        name: "Fluorine-18",
        category: SourceCategory::Medical,
        found_in: "PET scans (FDG); patients for several hours after the scan",
        components: &[component(&[member(9, 9, 0, 1.0)])],
    },
    GammaSource {
        key: "i131",
        name: "Iodine-131",
        category: SourceCategory::Medical,
        found_in: "Thyroid therapy patients, hospital waste water, reactor accident fallout",
        components: &[component(&[member(53, 78, 0, 1.0)])],
    },
    GammaSource {
        key: "i123",
        name: "Iodine-123",
        category: SourceCategory::Medical,
        found_in: "Thyroid imaging and DaTscan patients",
        components: &[component(&[member(53, 70, 0, 1.0)])],
    },
    GammaSource {
        key: "i125",
        name: "Iodine-125",
        category: SourceCategory::Medical,
        found_in: "Prostate brachytherapy seeds, laboratory radioimmunoassay kits",
        components: &[component(&[member(53, 72, 0, 1.0)])],
    },
    GammaSource {
        key: "ga67",
        name: "Gallium-67",
        category: SourceCategory::Medical,
        found_in: "Infection and tumour imaging patients",
        components: &[component(&[member(31, 36, 0, 1.0)])],
    },
    GammaSource {
        key: "in111",
        name: "Indium-111",
        category: SourceCategory::Medical,
        found_in: "Octreotide and white blood cell imaging patients",
        components: &[component(&[member(49, 62, 0, 1.0)])],
    },
    GammaSource {
        key: "tl201",
        name: "Thallium-201",
        category: SourceCategory::Medical,
        found_in: "Cardiac stress test patients",
        components: &[component(&[member(81, 120, 0, 1.0)])],
    },
    GammaSource {
        key: "lu177",
        name: "Lutetium-177",
        category: SourceCategory::Medical,
        found_in: "PSMA and DOTATATE radioligand therapy patients",
        components: &[component(&[member(71, 106, 0, 1.0)])],
    },
    GammaSource {
        key: "xe133",
        name: "Xenon-133",
        category: SourceCategory::Medical,
        found_in: "Lung ventilation scans, reactor off-gas",
        components: &[component(&[member(54, 79, 0, 1.0)])],
    },
];
