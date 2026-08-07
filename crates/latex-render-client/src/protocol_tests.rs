use latex_render_core::{RenderErrorCode, RenderLimits, Rgba};

use crate::WorkerClientConfig;
use crate::config::WorkerCommand;
use crate::protocol::{decode_ready, decode_response, encode_request};

#[test]
fn request_encoding_matches_the_versioned_camel_case_contract() {
    let request = request("x^2");
    let encoded =
        encode_request("eq-42", &request, &RenderLimits::default()).expect("request should encode");
    let value: serde_json::Value =
        serde_json::from_slice(&encoded).expect("encoded request should be JSON");

    assert_eq!(value["protocol"], 1);
    assert_eq!(value["id"], "eq-42");
    assert_eq!(value["method"], "render");
    assert_eq!(value["params"]["displayMode"], true);
    assert_eq!(value["params"]["foreground"], "#e6edf3");
    assert_eq!(value["params"]["background"], "transparent");
    assert!(value["params"].get("display_mode").is_none());
}

#[test]
fn request_encoding_rejects_partial_alpha() {
    let mut request = request("x");
    request.foreground = Rgba::new(1, 2, 3, 127);

    let error = encode_request("eq-1", &request, &RenderLimits::default())
        .expect_err("partial alpha should fail");
    assert_eq!(error.code, RenderErrorCode::InvalidRequest);
}

#[test]
fn ready_handshake_requires_exact_identity_and_capabilities() {
    let config = config();
    let valid = ready("0.1.0", "[\"svg\"]", "[\"inline\",\"display\"]");
    assert_eq!(
        decode_ready(valid.as_bytes(), &config).as_deref(),
        Ok("0.1.0")
    );

    let wrong_version = ready("0.2.0", "[\"svg\"]", "[\"inline\",\"display\"]");
    assert_eq!(
        decode_ready(wrong_version.as_bytes(), &config)
            .expect_err("version mismatch should fail")
            .code,
        RenderErrorCode::Protocol
    );

    let missing_mode = ready("0.1.0", "[\"svg\"]", "[\"display\"]");
    assert_eq!(
        decode_ready(missing_mode.as_bytes(), &config)
            .expect_err("missing mode should fail")
            .code,
        RenderErrorCode::Protocol
    );
}

#[test]
fn ready_handshake_accepts_a_stricter_client_limit() {
    let mut config = config();
    config.render_limits.max_svg_bytes = 1024;
    let valid = ready("0.1.0", "[\"svg\"]", "[\"inline\",\"display\"]");

    assert_eq!(
        decode_ready(valid.as_bytes(), &config).as_deref(),
        Ok("0.1.0")
    );
}

#[test]
fn response_decoder_correlates_and_validates_success() {
    let line = br#"{"protocol":1,"id":"eq-1","ok":true,"result":{"svgUtf8":"<svg></svg>","widthPx":10,"heightPx":5,"baselinePx":4,"accessibilityText":"x"}}"#;
    let rendered = decode_response(line, "eq-1", "v1:key".to_owned(), &RenderLimits::default())
        .expect("response should decode");

    assert_eq!(rendered.svg, b"<svg></svg>");
    assert_eq!(rendered.cache_key, "v1:key");

    let error = decode_response(line, "eq-2", "v1:key".to_owned(), &RenderLimits::default())
        .expect_err("wrong correlation should fail");
    assert_eq!(error.code, RenderErrorCode::Protocol);
}

#[test]
fn response_decoder_maps_worker_errors_without_controls() {
    let line = br#"{"protocol":1,"id":"eq-1","ok":false,"error":{"code":"INVALID_TEX","message":"bad\u001bvalue","retryable":false,"position":7}}"#;
    let error = decode_response(line, "eq-1", "v1:key".to_owned(), &RenderLimits::default())
        .expect_err("worker error should be returned");

    assert_eq!(error.code, RenderErrorCode::InvalidTex);
    assert_eq!(error.position, Some(7));
    assert!(!error.message.contains('\u{001b}'));
}

#[test]
fn response_decoder_rejects_invalid_success_as_unsafe_output() {
    let line = br#"{"protocol":1,"id":"eq-1","ok":true,"result":{"svgUtf8":"<svg></svg>","widthPx":0,"heightPx":5,"baselinePx":4,"accessibilityText":"x"}}"#;
    let error = decode_response(line, "eq-1", "v1:key".to_owned(), &RenderLimits::default())
        .expect_err("invalid output should fail");

    assert_eq!(error.code, RenderErrorCode::UnsafeOutput);
}

fn request(source: &str) -> latex_render_core::RenderRequest {
    latex_render_core::RenderRequest {
        source: source.to_owned(),
        display_mode: true,
        foreground: Rgba::opaque(230, 237, 243),
        background: None,
        scale: 2.0,
        max_width_px: 1200,
    }
}

fn config() -> WorkerClientConfig {
    WorkerClientConfig::new(WorkerCommand::new("node"))
}

fn ready(version: &str, formats: &str, display_modes: &str) -> String {
    format!(
        r#"{{"protocol":1,"type":"ready","renderer":{{"name":"mathjax","version":"{version}"}},"capabilities":{{"formats":{formats},"displayModes":{display_modes}}},"limits":{{"maxSourceBytes":16384,"maxJsonLineBytes":65536,"maxSvgBytes":2097152,"maxWidthPx":4096,"maxHeightPx":2048,"minScale":0.5,"maxScale":4}}}}"#
    )
}
