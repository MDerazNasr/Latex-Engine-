use super::*;

const VALID_REQUEST: &str = r##"{
  "protocol": 1,
  "id": "message-42",
  "method": "render_message",
  "params": {
    "source": "Use \\(x^2\\).",
    "inlineDollars": "smart",
    "foreground": "#e6edf3",
    "background": "transparent",
    "scale": 2,
    "maxWidthPx": 1200
  }
}"##;

#[test]
fn valid_request_decodes_to_render_values() {
    let request = decode_request_v1(VALID_REQUEST.as_bytes()).unwrap();

    assert_eq!(
        request,
        ValidatedRenderMessageV1 {
            id: "message-42".to_owned(),
            source: "Use \\(x^2\\).".to_owned(),
            inline_dollars: InlineDollarsV1::Smart,
            foreground: Rgba::opaque(230, 237, 243),
            background: None,
            scale: 2.0,
            max_width_px: 1200,
        }
    );
}

#[test]
fn opaque_background_and_dollar_modes_decode() {
    for (mode, expected) in [
        ("off", InlineDollarsV1::Off),
        ("smart", InlineDollarsV1::Smart),
        ("always", InlineDollarsV1::Always),
    ] {
        let json = VALID_REQUEST
            .replace("\"smart\"", &format!("\"{mode}\""))
            .replace("\"transparent\"", "\"#111827\"");

        let request = decode_request_v1(json.as_bytes()).unwrap();

        assert_eq!(request.inline_dollars, expected);
        assert_eq!(request.background, Some(Rgba::opaque(17, 24, 39)));
    }
}

#[test]
fn malformed_json_has_no_correlation_id() {
    let error = decode_request_v1(br#"{"id":"message-42""#).unwrap_err();

    assert_eq!(
        error,
        DecodeErrorV1 {
            id: None,
            error: super::error("invalid_request", false),
        }
    );
}

#[test]
fn oversized_request_line_is_rejected_before_id_reflection() {
    let json = vec![b' '; MAX_DAEMON_REQUEST_LINE_BYTES + 1];

    let error = decode_request_v1(&json).unwrap_err();

    assert_eq!(
        error,
        DecodeErrorV1 {
            id: None,
            error: super::error("input_limit_exceeded", false),
        }
    );
}

#[test]
fn unknown_field_preserves_safe_correlation_id() {
    let json = VALID_REQUEST.replace("\"protocol\": 1,", "\"protocol\": 1, \"unexpected\": true,");

    let error = decode_request_v1(json.as_bytes()).unwrap_err();

    assert_eq!(error.id.as_deref(), Some("message-42"));
    assert_eq!(error.error, super::error("invalid_request", false));
}

#[test]
fn protocol_mismatch_has_stable_code() {
    let json = VALID_REQUEST.replace("\"protocol\": 1", "\"protocol\": 2");

    let error = decode_request_v1(json.as_bytes()).unwrap_err();

    assert_eq!(error.id.as_deref(), Some("message-42"));
    assert_eq!(error.error, super::error("protocol", false));
}

#[test]
fn invalid_ids_are_not_reflected() {
    for id in ["", "line\\u0000break", &"a".repeat(129)] {
        let json = VALID_REQUEST.replace("message-42", id);

        let error = decode_request_v1(json.as_bytes()).unwrap_err();

        assert_eq!(error.id, None);
        assert_eq!(error.error, super::error("invalid_request", false));
    }
}

#[test]
fn empty_unsafe_and_oversized_sources_are_rejected() {
    for (source, code) in [
        (String::new(), "invalid_request"),
        ("unsafe\\u0000source".to_owned(), "invalid_request"),
        (
            "x".repeat(MAX_DAEMON_MESSAGE_BYTES + 1),
            "input_limit_exceeded",
        ),
    ] {
        let json = VALID_REQUEST.replace("Use \\\\(x^2\\\\).", &source);

        let error = decode_request_v1(json.as_bytes()).unwrap_err();

        assert_eq!(error.id.as_deref(), Some("message-42"));
        assert_eq!(error.error, super::error(code, false));
    }
}

#[test]
fn invalid_colors_scale_and_width_are_rejected() {
    for json in [
        VALID_REQUEST.replace("#e6edf3", "red"),
        VALID_REQUEST.replace("\"scale\": 2", "\"scale\": 99"),
        VALID_REQUEST.replace("\"maxWidthPx\": 1200", "\"maxWidthPx\": 0"),
    ] {
        let error = decode_request_v1(json.as_bytes()).unwrap_err();

        assert_eq!(error.id.as_deref(), Some("message-42"));
        assert_eq!(error.error, super::error("invalid_request", false));
    }
}

#[test]
fn rendered_response_uses_camel_case_without_source() {
    let response = DaemonResponseV1::success(
        "message-42".to_owned(),
        vec![EquationOutcomeV1::rendered(
            4..11,
            false,
            "iVBORw0KGgo".to_owned(),
            (64, 32),
            Some(24.0),
            "x squared".to_owned(),
        )],
    );

    let json = serde_json::to_string(&response).unwrap();

    assert_eq!(
        json,
        r#"{"protocol":1,"id":"message-42","ok":true,"result":{"equations":[{"status":"rendered","startByte":4,"endByte":11,"displayMode":false,"pngBase64":"iVBORw0KGgo","widthPx":64,"heightPx":32,"baselinePx":24.0,"accessibilityText":"x squared"}]}}"#
    );
    assert!(!json.contains("Use"));
    assert!(!json.contains("x^2"));
}

#[test]
fn failed_outcome_and_top_error_have_stable_shapes() {
    let failed = DaemonResponseV1::success(
        "message-42".to_owned(),
        vec![EquationOutcomeV1::failed(
            4..11,
            false,
            super::error("invalid_tex", false),
        )],
    );
    let top = DaemonResponseV1::error(None, super::error("invalid_request", false));

    assert_eq!(
        serde_json::to_string(&failed).unwrap(),
        r#"{"protocol":1,"id":"message-42","ok":true,"result":{"equations":[{"status":"failed","startByte":4,"endByte":11,"displayMode":false,"error":{"code":"invalid_tex","retryable":false}}]}}"#
    );
    assert_eq!(
        serde_json::to_string(&top).unwrap(),
        r#"{"protocol":1,"id":null,"ok":false,"error":{"code":"invalid_request","retryable":false}}"#
    );
}

#[test]
fn all_daemon_limits_match_the_committed_contract() {
    assert_eq!(MAX_DAEMON_REQUEST_LINE_BYTES, 1024 * 1024);
    assert_eq!(MAX_DAEMON_MESSAGE_BYTES, 256 * 1024);
    assert_eq!(MAX_DAEMON_EQUATIONS, 32);
    assert_eq!(MAX_DAEMON_PNG_BYTES, 4 * 1024 * 1024);
    assert_eq!(MAX_DAEMON_TOTAL_PNG_BYTES, 8 * 1024 * 1024);
    assert_eq!(MAX_DAEMON_ACCESSIBILITY_BYTES, 64 * 1024);
    assert_eq!(MAX_DAEMON_TOTAL_ACCESSIBILITY_BYTES, 256 * 1024);
    assert_eq!(MAX_DAEMON_RESPONSE_LINE_BYTES, 12 * 1024 * 1024);
}
