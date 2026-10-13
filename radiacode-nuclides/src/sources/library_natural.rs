use super::chains::{
    NATURAL_U235_ACTIVITY_RATIO, RADIUM_226_CHAIN, RADON_222_PROGENY, REFINED_URANIUM_235,
    REFINED_URANIUM_238, THORIUM_232_CHAIN, URANIUM_235_CHAIN, URANIUM_238_CHAIN,
};
use super::model::{GammaSource, SourceCategory, SourceComponent, component, member};

pub const NATURAL_SOURCES: &[GammaSource] = &[
    GammaSource {
        key: "k40",
        name: "Potassium-40",
        category: SourceCategory::Natural,
        found_in: "Potassium salt substitutes, fertilizer, bananas, granite, concrete, the human body",
        components: &[component(&[member(19, 21, 0, 1.0)])],
    },
    GammaSource {
        key: "u238-series",
        name: "Uranium ore (U-238 series)",
        category: SourceCategory::Natural,
        found_in: "Uranium minerals (pitchblende, autunite, torbernite), soil, granite, building materials",
        components: &[
            component(URANIUM_238_CHAIN),
            SourceComponent {
                members: URANIUM_235_CHAIN,
                activity_ratio: NATURAL_U235_ACTIVITY_RATIO,
            },
        ],
    },
    GammaSource {
        key: "refined-uranium",
        name: "Refined uranium",
        category: SourceCategory::Natural,
        found_in: "Uranium glass, Fiestaware and other uranium glazes, depleted uranium, yellowcake",
        components: &[
            component(REFINED_URANIUM_238),
            SourceComponent {
                members: REFINED_URANIUM_235,
                activity_ratio: NATURAL_U235_ACTIVITY_RATIO,
            },
        ],
    },
    GammaSource {
        key: "th232-series",
        name: "Thorium (Th-232 series)",
        category: SourceCategory::Natural,
        found_in: "Gas lantern mantles, thoriated TIG electrodes, monazite sand, thoriated lenses",
        components: &[component(THORIUM_232_CHAIN)],
    },
    GammaSource {
        key: "ra226",
        name: "Radium-226",
        category: SourceCategory::Natural,
        found_in: "Luminous radium dials and gauges, old medical applicators, radium check sources",
        components: &[component(RADIUM_226_CHAIN)],
    },
    GammaSource {
        key: "radon-progeny",
        name: "Radon progeny (Pb-214, Bi-214)",
        category: SourceCategory::Natural,
        found_in: "Rain-out after showers, HVAC and vacuum filters, basements, mine air",
        components: &[component(RADON_222_PROGENY)],
    },
    GammaSource {
        key: "lu176",
        name: "Lutetium-176",
        category: SourceCategory::Natural,
        found_in: "LYSO and LSO scintillator crystals, lutetium metal and oxide samples",
        components: &[component(&[member(71, 105, 0, 1.0)])],
    },
    GammaSource {
        key: "be7",
        name: "Beryllium-7",
        category: SourceCategory::Natural,
        found_in: "Cosmogenic; rainwater residue and air filters",
        components: &[component(&[member(4, 3, 0, 1.0)])],
    },
];
