use radiacode_nuclides::{Confidence, VerdictKind};

use crate::peak_overlay::verdict::verdict_color;
use crate::theme::{ACCENT, MUTED};

#[test]
fn inconclusive_verdicts_are_muted_regardless_of_confidence() {
    assert_eq!(
        verdict_color(Confidence::High, VerdictKind::Inconclusive),
        MUTED
    );
}

#[test]
fn only_high_confidence_verdicts_use_accent() {
    assert_eq!(verdict_color(Confidence::High, VerdictKind::Chain), ACCENT);
    assert_eq!(verdict_color(Confidence::Medium, VerdictKind::Chain), MUTED);
    assert_eq!(verdict_color(Confidence::Low, VerdictKind::Chain), MUTED);
}
