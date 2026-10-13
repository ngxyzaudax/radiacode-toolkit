use super::model::{SourceMember, member};

pub const NATURAL_U235_ACTIVITY_RATIO: f64 = 0.046;

pub const URANIUM_238_CHAIN: &[SourceMember] = &[
    member(92, 146, 0, 1.0),
    member(90, 144, 0, 1.0),
    member(91, 143, 1, 1.0),
    member(91, 143, 0, 0.0016),
    member(92, 142, 0, 1.0),
    member(90, 140, 0, 1.0),
    member(88, 138, 0, 1.0),
    member(86, 136, 0, 1.0),
    member(84, 134, 0, 1.0),
    member(82, 132, 0, 0.9998),
    member(83, 131, 0, 1.0),
    member(84, 130, 0, 0.9998),
    member(82, 128, 0, 1.0),
    member(83, 127, 0, 1.0),
    member(84, 126, 0, 1.0),
];

pub const THORIUM_232_CHAIN: &[SourceMember] = &[
    member(90, 142, 0, 1.0),
    member(88, 140, 0, 1.0),
    member(89, 139, 0, 1.0),
    member(90, 138, 0, 1.0),
    member(88, 136, 0, 1.0),
    member(86, 134, 0, 1.0),
    member(84, 132, 0, 1.0),
    member(82, 130, 0, 1.0),
    member(83, 129, 0, 1.0),
    member(84, 128, 0, 0.6406),
    member(81, 127, 0, 0.3594),
];

pub const URANIUM_235_CHAIN: &[SourceMember] = &[
    member(92, 143, 0, 1.0),
    member(90, 141, 0, 1.0),
    member(91, 140, 0, 1.0),
    member(89, 138, 0, 1.0),
    member(90, 137, 0, 0.9862),
    member(87, 136, 0, 0.0138),
    member(88, 135, 0, 1.0),
    member(86, 133, 0, 1.0),
    member(84, 131, 0, 1.0),
    member(82, 129, 0, 1.0),
    member(83, 128, 0, 1.0),
    member(81, 126, 0, 0.99724),
    member(84, 127, 0, 0.00276),
];

pub const REFINED_URANIUM_238: &[SourceMember] = URANIUM_238_CHAIN.split_at(5).0;
pub const REFINED_URANIUM_235: &[SourceMember] = URANIUM_235_CHAIN.split_at(2).0;
pub const RADIUM_226_CHAIN: &[SourceMember] = URANIUM_238_CHAIN.split_at(6).1;
pub const RADON_222_PROGENY: &[SourceMember] = URANIUM_238_CHAIN.split_at(7).1.split_at(5).0;
