use crate::layout::safe_span;

pub struct DetailLayoutConfig {
    pub chain_share: f32,
    pub chain_max_share: f32,
    pub preview_min_height: f32,
    pub chain_min_height: f32,
    pub section_gap: f32,
}

pub struct DetailLayout {
    pub chain_height: f32,
    pub preview_height: f32,
}

pub fn is_tight_layout(remaining: f32, tight_height: f32) -> bool {
    remaining < tight_height
}

pub fn detail_layout(
    remaining: f32,
    chain_collapsed: bool,
    config: DetailLayoutConfig,
) -> DetailLayout {
    let chain_fraction = if chain_collapsed {
        0.0
    } else {
        config.chain_share
    };
    let chain_height = if chain_collapsed {
        0.0
    } else {
        safe_span(remaining * chain_fraction, 0.0, config.chain_min_height)
            .min(remaining * config.chain_max_share)
    };
    let section_gap = if chain_collapsed {
        0.0
    } else {
        config.section_gap
    };
    let preview_height = safe_span(
        remaining - chain_height - section_gap,
        0.0,
        config.preview_min_height,
    );
    DetailLayout {
        chain_height,
        preview_height,
    }
}
