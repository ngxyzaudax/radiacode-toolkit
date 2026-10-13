use radiacode_protocol::{DataBufFrame, DecodeWarningKind, Seq};

const MAX_PLAUSIBLE_GAP: u32 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqGap {
    pub expected: u8,
    pub got: u8,
    pub lost: u32,
    pub reset: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DataBufCursor {
    last_seq: Option<Seq>,
    pub total_gaps: u64,
    pub total_lost: u64,
}

impl DataBufCursor {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn observe_frame(&mut self, frame: &DataBufFrame) -> Vec<SeqGap> {
        let mut gaps = decode_seq_gaps(frame);
        if let (Some(last), Some(first)) = (self.last_seq, frame.first_header_seq) {
            push_cross_poll_gap(&mut gaps, last, first);
        }
        if let Some(last) = frame.last_header_seq {
            self.last_seq = Some(last);
        }
        self.account_losses(&gaps);
        gaps
    }

    fn account_losses(&mut self, gaps: &[SeqGap]) {
        for gap in gaps.iter().filter(|gap| !gap.reset) {
            self.total_gaps = self.total_gaps.saturating_add(1);
            self.total_lost = self.total_lost.saturating_add(u64::from(gap.lost));
        }
    }
}

fn decode_seq_gaps(frame: &DataBufFrame) -> Vec<SeqGap> {
    frame
        .warnings
        .iter()
        .filter_map(|warning| match warning.kind {
            DecodeWarningKind::SeqJump { expected, got } => {
                Some(seq_gap(Seq::new(got), Seq::new(expected)))
            }
            _ => None,
        })
        .collect()
}

fn seq_gap(got: Seq, expected: Seq) -> SeqGap {
    let lost = plausible_loss(got, expected);
    SeqGap {
        expected: expected.raw(),
        got: got.raw(),
        lost: lost.unwrap_or(0),
        reset: lost.is_none(),
    }
}

fn plausible_loss(got: Seq, expected: Seq) -> Option<u32> {
    if got == expected {
        return Some(0);
    }
    if !got.is_newer_than(expected) {
        return None;
    }
    got.lost_since(expected)
        .filter(|lost| *lost <= MAX_PLAUSIBLE_GAP)
}

fn push_cross_poll_gap(gaps: &mut Vec<SeqGap>, last: Seq, first: Seq) {
    let expected = last.next();
    if first == expected {
        return;
    }
    let candidate = seq_gap(first, expected);
    let already_reported = gaps
        .iter()
        .any(|gap| gap.expected == candidate.expected && gap.got == candidate.got);
    if !already_reported {
        gaps.push(candidate);
    }
}
