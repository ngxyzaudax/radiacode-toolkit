use crate::buffer::BytesBuffer;

use super::super::group::RecordKind;
use super::super::seq::Seq;

pub(super) const HEADER_LEN: usize = 7;
const MAX_RESYNCS_PER_FRAME: u32 = 4;
const UNKNOWN_TAIL_LEN: usize = 32;

pub(super) fn unknown_record_tail(data: &[u8], remaining: usize) -> Vec<u8> {
    let tail_start = data.len().saturating_sub(remaining);
    let tail_end = tail_start.saturating_add(UNKNOWN_TAIL_LEN).min(data.len());
    data[tail_start..tail_end].to_vec()
}

pub(super) fn resync_allowed(resync_count: u32) -> bool {
    resync_count < MAX_RESYNCS_PER_FRAME
}

pub(super) fn try_resync(buffer: &mut BytesBuffer, expected_seq: Seq) -> bool {
    let Some(skip) = next_header_offset(buffer.data(), expected_seq) else {
        return false;
    };
    buffer.skip(skip).is_ok()
}

fn next_header_offset(remaining: &[u8], expected_seq: Seq) -> Option<usize> {
    if remaining.len() < HEADER_LEN {
        return None;
    }
    (1..remaining.len().saturating_sub(HEADER_LEN - 1))
        .find(|&skip| header_starts_at(remaining, skip, expected_seq))
}

fn header_starts_at(remaining: &[u8], skip: usize, expected_seq: Seq) -> bool {
    remaining[skip] == expected_seq.raw()
        && !matches!(
            RecordKind::from_entity_group(remaining[skip + 1], remaining[skip + 2]),
            RecordKind::Unknown { .. }
        )
}
