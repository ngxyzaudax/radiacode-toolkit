use radiacode_protocol::{Command, build_request, framed_request_header, response_matches_request};

#[test]
fn response_matches_request_for_echo_header() {
    let request = build_request(Command::GetVersion, 0x81, &[]);
    let expected = framed_request_header(&request).expect("valid request header");
    let mut response = expected.to_vec();
    response.extend_from_slice(&[0xAA, 0xBB]);
    assert!(response_matches_request(&response, expected));
}

#[test]
fn response_rejects_mismatched_echo_header() {
    let request = build_request(Command::GetVersion, 0x81, &[]);
    let expected = framed_request_header(&request).expect("valid request header");
    let other = build_request(Command::GetSerial, 0x82, &[]);
    let mismatched = framed_request_header(&other).expect("valid other header");
    let mut response = mismatched.to_vec();
    response.push(0x01);
    assert!(!response_matches_request(&response, expected));
}
