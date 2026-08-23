use crate::series::is_natural_series_head;
use crate::source_summary::ChainEvidence;

pub fn leading_chain_for_rerank(chains: &[ChainEvidence]) -> Option<&ChainEvidence> {
    chains
        .iter()
        .filter(|chain| is_natural_series_head(chain.head) && chain.matched_members >= 2)
        .max_by(|left, right| {
            left.score
                .partial_cmp(&right.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| right.matched_members.cmp(&left.matched_members))
        })
        .or_else(|| {
            chains
                .iter()
                .filter(|chain| chain.matched_members >= 2)
                .max_by(|left, right| {
                    left.score
                        .partial_cmp(&right.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        })
}
