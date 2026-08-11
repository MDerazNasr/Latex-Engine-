use std::io::ErrorKind;

use tokio::io::BufReader;

use crate::line_reader::read_bounded_line;

#[tokio::test]
async fn reads_lf_and_crlf_frames() {
    let mut reader = BufReader::new(&b"one\r\ntwo\n"[..]);
    let mut output = Vec::new();

    read_bounded_line(&mut reader, &mut output, 8)
        .await
        .expect("first line should be valid");
    assert_eq!(output, b"one");
    read_bounded_line(&mut reader, &mut output, 8)
        .await
        .expect("second line should be valid");
    assert_eq!(output, b"two");
}

#[tokio::test]
async fn rejects_oversized_and_incomplete_frames() {
    let mut oversized = BufReader::new(&b"12345\n"[..]);
    let mut output = Vec::new();
    let error = read_bounded_line(&mut oversized, &mut output, 4)
        .await
        .expect_err("oversized line should fail");
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(output.len() <= 4);

    let mut incomplete = BufReader::new(&b"no newline"[..]);
    let error = read_bounded_line(&mut incomplete, &mut output, 32)
        .await
        .expect_err("incomplete line should fail");
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
}
