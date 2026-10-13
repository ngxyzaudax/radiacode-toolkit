use radiacode_protocol::decode_data_buf;

use crate::data_buf_cursor::DataBufCursor;
use crate::test_support::encode::{rare_record, real_time_record};

fn realtime(seq: u8) -> Vec<u8> {
    real_time_record(seq, 0, 5.0, 0.001)
}

#[test]
fn ignores_non_rate_records_between_polls() {
    let mut cursor = DataBufCursor::default();
    let gaps1 = cursor.observe_frame(&decode_data_buf(&realtime(10)));
    assert!(gaps1.is_empty());
    let mut bytes = rare_record(11, 0, 50.0, 20.0);
    bytes.extend_from_slice(&realtime(12));
    let gaps2 = cursor.observe_frame(&decode_data_buf(&bytes));
    assert!(gaps2.is_empty(), "unexpected gaps: {gaps2:?}");
}

#[test]
fn reports_real_gap_between_polls_and_accumulates_totals() {
    let mut cursor = DataBufCursor::default();
    cursor.observe_frame(&decode_data_buf(&realtime(10)));
    let gaps = cursor.observe_frame(&decode_data_buf(&realtime(12)));
    assert_eq!(gaps.len(), 1);
    assert_eq!((gaps[0].expected, gaps[0].got, gaps[0].lost), (11, 12, 1));
    assert!(!gaps[0].reset);
    assert_eq!((cursor.total_gaps, cursor.total_lost), (1, 1));
}

#[test]
fn reports_gap_inside_single_frame_once() {
    let mut cursor = DataBufCursor::default();
    cursor.observe_frame(&decode_data_buf(&realtime(1)));
    let mut bytes = realtime(2);
    bytes.extend_from_slice(&realtime(5));
    let gaps = cursor.observe_frame(&decode_data_buf(&bytes));
    assert_eq!(gaps.len(), 1);
    assert_eq!((gaps[0].expected, gaps[0].got, gaps[0].lost), (3, 5, 2));
}

#[test]
fn cross_poll_gap_is_not_duplicated_when_frame_already_reports_it() {
    let mut cursor = DataBufCursor::default();
    cursor.observe_frame(&decode_data_buf(&realtime(1)));
    let gaps = cursor.observe_frame(&decode_data_buf(&realtime(4)));
    assert_eq!(gaps.len(), 1);
    assert_eq!(cursor.total_lost, 2);
}

#[test]
fn reset_after_cursor_clear_has_no_cross_poll_gap() {
    let mut cursor = DataBufCursor::default();
    cursor.observe_frame(&decode_data_buf(&realtime(200)));
    cursor.reset();
    let gaps = cursor.observe_frame(&decode_data_buf(&realtime(5)));
    assert!(gaps.is_empty(), "unexpected gaps after reset: {gaps:?}");
}

#[test]
fn backwards_sequence_reports_reset_not_loss() {
    let mut cursor = DataBufCursor::default();
    cursor.observe_frame(&decode_data_buf(&realtime(200)));
    let gaps = cursor.observe_frame(&decode_data_buf(&realtime(5)));
    assert_eq!(gaps.len(), 1);
    assert!(gaps[0].reset);
    assert_eq!(gaps[0].lost, 0);
    assert_eq!(cursor.total_gaps, 0);
}

#[test]
fn implausibly_large_forward_jump_is_treated_as_reset() {
    let mut cursor = DataBufCursor::default();
    cursor.observe_frame(&decode_data_buf(&realtime(0)));
    let gaps = cursor.observe_frame(&decode_data_buf(&realtime(100)));
    assert_eq!(gaps.len(), 1);
    assert!(gaps[0].reset);
    assert_eq!(cursor.total_lost, 0);
}

#[test]
fn wrapping_sequence_counts_loss_across_boundary() {
    let mut cursor = DataBufCursor::default();
    cursor.observe_frame(&decode_data_buf(&realtime(254)));
    let gaps = cursor.observe_frame(&decode_data_buf(&realtime(1)));
    assert_eq!(gaps.len(), 1);
    assert_eq!((gaps[0].expected, gaps[0].got, gaps[0].lost), (255, 1, 2));
}
