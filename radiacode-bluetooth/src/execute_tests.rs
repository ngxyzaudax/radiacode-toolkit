use std::time::Duration;

use futures::StreamExt;
use futures::stream;
use radiacode_protocol::{
    Command, Error, build_request, framed_request_header, response_matches_request,
};
use tokio::time::Instant;

use crate::drain::NotificationStream;
use crate::execute::await_matching_response;
use crate::test_streams::{framed_response, notification};

fn expected_header() -> [u8; 4] {
    let request = build_request(Command::GetVersion, 0x81, &[]);
    framed_request_header(&request).expect("valid request header")
}

fn other_header() -> [u8; 4] {
    let request = build_request(Command::GetSerial, 0x82, &[]);
    framed_request_header(&request).expect("valid other header")
}

fn response_payload(header: [u8; 4], body: &[u8]) -> Vec<u8> {
    let mut payload = header.to_vec();
    payload.extend_from_slice(body);
    payload
}

fn stream_of(chunks: Vec<Vec<u8>>) -> NotificationStream {
    stream::iter(chunks.into_iter().map(notification)).boxed()
}

fn far_deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}

#[test]
fn response_matches_request_for_echo_header() {
    let response = response_payload(expected_header(), &[0xAA, 0xBB]);
    assert!(response_matches_request(&response, expected_header()));
}

#[test]
fn response_rejects_mismatched_echo_header() {
    let mismatched = response_payload(other_header(), &[0x01]);
    assert!(!response_matches_request(&mismatched, expected_header()));
}

#[tokio::test]
async fn chunked_response_is_reassembled() {
    let frame = framed_response(&response_payload(expected_header(), &[1, 2, 3, 4, 5, 6]));
    let (head, tail) = frame.split_at(7);
    let mut notifications = stream_of(vec![head.to_vec(), tail.to_vec()]);
    let response = await_matching_response(&mut notifications, expected_header(), far_deadline())
        .await
        .expect("assembled response");
    assert_eq!(response.size(), 10);
}

#[tokio::test]
async fn unrelated_frames_are_discarded_before_match() {
    let stale = framed_response(&response_payload(other_header(), &[9]));
    let wanted = framed_response(&response_payload(expected_header(), &[7]));
    let mut notifications = stream_of(vec![stale, wanted]);
    let response = await_matching_response(&mut notifications, expected_header(), far_deadline())
        .await
        .expect("matching response");
    assert_eq!(response.size(), 5);
}

#[tokio::test]
async fn closed_stream_reports_connection_closed() {
    let mut notifications = stream_of(Vec::new());
    let outcome =
        await_matching_response(&mut notifications, expected_header(), far_deadline()).await;
    assert!(matches!(outcome, Err(Error::ConnectionClosed)));
}

#[tokio::test(start_paused = true)]
async fn silent_stream_times_out_at_deadline() {
    let mut notifications: NotificationStream = stream::pending().boxed();
    let deadline = Instant::now() + Duration::from_millis(50);
    let outcome = await_matching_response(&mut notifications, expected_header(), deadline).await;
    assert!(matches!(outcome, Err(Error::Timeout)));
}
