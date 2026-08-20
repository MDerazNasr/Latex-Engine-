use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use latex_render_core::RenderFuture;
use latex_render_core::RenderRequest;
use latex_render_core::RenderedMath;
use serde_json::Value;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;

use super::*;

const RECTANGLE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50" viewBox="0 0 100 50" role="img" focusable="false" style="color:#000000"><rect x="0" y="0" width="100" height="50" fill="currentColor"/></svg>"##;

struct FakeRenderer;

impl MathRenderer for FakeRenderer {
    fn render(&self, request: RenderRequest) -> RenderFuture<'_> {
        Box::pin(async move {
            Ok(RenderedMath {
                svg: RECTANGLE.to_vec(),
                width_px: 100,
                height_px: 50,
                baseline_px: Some(35.0),
                accessibility_text: "rendered math".to_owned(),
                cache_key: format!("key-{}", request.source),
            })
        })
    }
}

#[tokio::test]
async fn loop_recovers_from_malformed_request_and_preserves_order() {
    let input = format!(
        "not-json\n{}\n{}\n",
        request_json("first", "Text \\(x\\)."),
        request_json("second", "No math.")
    );
    let mut reader = BufReader::new(input.as_bytes());
    let mut writer = Vec::new();

    serve_daemon_v1(&FakeRenderer, &mut reader, &mut writer)
        .await
        .unwrap();

    let responses = response_values(writer);
    assert_eq!(responses.len(), 3);
    assert_eq!(responses[0]["id"], Value::Null);
    assert_eq!(responses[0]["error"]["code"], "invalid_request");
    assert_eq!(responses[1]["id"], "first");
    assert_eq!(
        responses[1]["result"]["equations"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(responses[2]["id"], "second");
    assert!(
        responses[2]["result"]["equations"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn oversized_line_is_drained_before_next_request() {
    let mut input = vec![b'x'; MAX_DAEMON_REQUEST_LINE_BYTES + 1];
    input.push(b'\n');
    input.extend_from_slice(request_json("next", "No math.").as_bytes());
    input.push(b'\n');
    let mut reader = BufReader::new(input.as_slice());
    let mut writer = Vec::new();

    serve_daemon_v1(&FakeRenderer, &mut reader, &mut writer)
        .await
        .unwrap();

    let responses = response_values(writer);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["error"]["code"], "input_limit_exceeded");
    assert_eq!(responses[1]["id"], "next");
    assert_eq!(responses[1]["ok"], true);
}

#[tokio::test]
async fn final_line_without_newline_is_returned_before_eof() {
    let mut reader = BufReader::new(&b"last-line"[..]);

    let first = read_bounded_line_v1(&mut reader).await.unwrap();
    let second = read_bounded_line_v1(&mut reader).await.unwrap();

    assert!(matches!(first, LineReadV1::Line(line) if line == b"last-line"));
    assert!(matches!(second, LineReadV1::Eof));
}

#[tokio::test]
async fn oversized_response_becomes_small_correlated_error() {
    let response = DaemonResponseV1::success(
        "large".to_owned(),
        vec![crate::daemon_protocol_v1::EquationOutcomeV1::rendered(
            0..1,
            false,
            "A".repeat(MAX_DAEMON_RESPONSE_LINE_BYTES),
            (1, 1),
            None,
            "math".to_owned(),
        )],
    );
    let mut writer = Vec::new();

    write_response_v1(&mut writer, response, Some("large".to_owned()))
        .await
        .unwrap();

    let responses = response_values(writer);
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["id"], "large");
    assert_eq!(responses[0]["error"]["code"], "output_limit_exceeded");
}

#[tokio::test]
async fn short_writes_complete_one_valid_json_line() {
    #[derive(Default)]
    struct ShortWriter {
        bytes: Vec<u8>,
        flushes: usize,
    }

    impl AsyncWrite for ShortWriter {
        fn poll_write(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
            buffer: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            let this = self.get_mut();
            let count = buffer.len().min(3);
            this.bytes.extend_from_slice(&buffer[..count]);
            Poll::Ready(Ok(count))
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<std::io::Result<()>> {
            self.get_mut().flushes += 1;
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    let mut writer = ShortWriter::default();
    let response = DaemonResponseV1::error(None, error("invalid_request", false));

    write_response_v1(&mut writer, response, None)
        .await
        .unwrap();

    assert_eq!(writer.flushes, 1);
    let responses = response_values(writer.bytes);
    assert_eq!(responses[0]["error"]["code"], "invalid_request");
}

#[tokio::test]
async fn writer_failure_is_a_source_free_output_error() {
    struct FailingWriter;

    impl AsyncWrite for FailingWriter {
        fn poll_write(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
            _buffer: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "secret",
            )))
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    let response = DaemonResponseV1::error(None, error("invalid_request", false));
    let cli_error = write_response_v1(&mut FailingWriter, response, None)
        .await
        .unwrap_err();

    assert_eq!(cli_error.kind(), CliErrorKind::Output);
    assert_eq!(
        cli_error.to_string(),
        "Daemon response could not be written"
    );
    assert!(!cli_error.to_string().contains("secret"));
}

#[tokio::test]
async fn loop_allows_runtime_progress_while_input_is_idle() {
    let (mut sender, receiver) = tokio::io::duplex(1024);
    let mut reader = BufReader::new(receiver);
    let mut writer = Vec::new();
    let input = format!("{}\n", request_json("after-idle", "No math."));
    let feeder = tokio::spawn(async move {
        tokio::task::yield_now().await;
        sender.write_all(input.as_bytes()).await.unwrap();
    });

    serve_daemon_v1(&FakeRenderer, &mut reader, &mut writer)
        .await
        .unwrap();
    feeder.await.unwrap();

    let responses = response_values(writer);
    assert_eq!(responses[0]["id"], "after-idle");
    assert_eq!(responses[0]["ok"], true);
}

fn request_json(id: &str, source: &str) -> String {
    serde_json::json!({
        "protocol": 1,
        "id": id,
        "method": "render_message",
        "params": {
            "source": source,
            "inlineDollars": "smart",
            "foreground": "#e6edf3",
            "background": "transparent",
            "scale": 2,
            "maxWidthPx": 1200
        }
    })
    .to_string()
}

fn response_values(output: Vec<u8>) -> Vec<Value> {
    String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
