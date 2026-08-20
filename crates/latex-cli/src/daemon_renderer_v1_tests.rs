use std::sync::Mutex;

use base64::engine::general_purpose;
use latex_render_core::RenderFuture;
use latex_render_core::RenderedMath;

use super::*;

const RECTANGLE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50" viewBox="0 0 100 50" role="img" focusable="false" style="color:#000000"><rect x="0" y="0" width="100" height="50" fill="currentColor"/></svg>"##;

#[derive(Default)]
struct FakeRenderer {
    requests: Mutex<Vec<RenderRequest>>,
}

impl MathRenderer for FakeRenderer {
    fn render(&self, request: RenderRequest) -> RenderFuture<'_> {
        self.requests.lock().unwrap().push(request.clone());
        Box::pin(async move {
            if request.source == "bad" {
                return Err(RenderError::new(
                    RenderErrorCode::InvalidTex,
                    "Invalid math",
                    false,
                ));
            }
            let svg = if request.source == "unsafe" {
                br#"<svg xmlns="http://www.w3.org/2000/svg"><script/></svg>"#.to_vec()
            } else {
                RECTANGLE.to_vec()
            };
            let accessibility_text = if request.source == "verbose" {
                "a".repeat(100)
            } else {
                "rendered math".to_owned()
            };
            Ok(RenderedMath {
                svg,
                width_px: 100,
                height_px: 50,
                baseline_px: Some(35.0),
                accessibility_text,
                cache_key: format!("key-{}", request.source),
            })
        })
    }
}

#[tokio::test]
async fn message_segmentation_renders_math_but_not_code_or_prices() {
    let renderer = FakeRenderer::default();
    let request = request_for("Text \\(x\\), `\\(code\\)`, $19.99, and $$y$$.");

    let outcomes = render_message_v1(&renderer, request).await.unwrap();

    assert_eq!(outcomes.len(), 2);
    assert!(matches!(
        &outcomes[0],
        EquationOutcomeV1::Rendered {
            start_byte: 5,
            end_byte: 10,
            display_mode: false,
            ..
        }
    ));
    assert!(matches!(
        &outcomes[1],
        EquationOutcomeV1::Rendered {
            display_mode: true,
            ..
        }
    ));
    let requests = renderer.requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].source, "x");
    assert!(!requests[0].display_mode);
    assert_eq!(requests[1].source, "y");
    assert!(requests[1].display_mode);
}

#[tokio::test]
async fn smart_and_off_dollar_modes_produce_distinct_results() {
    let renderer = FakeRenderer::default();
    let mut smart = request_for("Value $x$ and $19.99.");
    smart.inline_dollars = InlineDollarsV1::Smart;
    let mut off = smart.clone();
    off.inline_dollars = InlineDollarsV1::Off;

    let smart_outcomes = render_message_v1(&renderer, smart).await.unwrap();
    let off_outcomes = render_message_v1(&renderer, off).await.unwrap();

    assert_eq!(smart_outcomes.len(), 1);
    assert!(off_outcomes.is_empty());
}

#[tokio::test]
async fn one_render_failure_does_not_suppress_later_equations() {
    let renderer = FakeRenderer::default();
    let request = request_for("\\(ok\\) \\(bad\\) \\(after\\)");

    let outcomes = render_message_v1(&renderer, request).await.unwrap();

    assert_eq!(outcomes.len(), 3);
    assert!(matches!(&outcomes[0], EquationOutcomeV1::Rendered { .. }));
    assert!(matches!(
        &outcomes[1],
        EquationOutcomeV1::Failed {
            error: DaemonErrorV1 { code, retryable: false },
            ..
        } if code == "invalid_tex"
    ));
    assert!(matches!(&outcomes[2], EquationOutcomeV1::Rendered { .. }));
    assert_eq!(renderer.requests.lock().unwrap().len(), 3);
}

#[tokio::test]
async fn unsafe_svg_fails_only_its_equation() {
    let renderer = FakeRenderer::default();
    let request = request_for("\\(unsafe\\) \\(ok\\)");

    let outcomes = render_message_v1(&renderer, request).await.unwrap();

    assert!(matches!(
        &outcomes[0],
        EquationOutcomeV1::Failed {
            error: DaemonErrorV1 { code, .. },
            ..
        } if code == "unsafe_output"
    ));
    assert!(matches!(&outcomes[1], EquationOutcomeV1::Rendered { .. }));
}

#[tokio::test]
async fn rendered_outcome_contains_decodable_bounded_png() {
    let renderer = FakeRenderer::default();
    let request = request_for("\\(x\\)");

    let outcomes = render_message_v1(&renderer, request).await.unwrap();

    let EquationOutcomeV1::Rendered {
        png_base64,
        width_px,
        height_px,
        baseline_px,
        accessibility_text,
        ..
    } = &outcomes[0]
    else {
        panic!("expected rendered outcome");
    };
    let png = general_purpose::STANDARD.decode(png_base64).unwrap();
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(png.len() <= MAX_DAEMON_PNG_BYTES);
    assert_eq!((*width_px, *height_px), (100, 50));
    assert_eq!(*baseline_px, Some(35.0));
    assert_eq!(accessibility_text, "rendered math");
}

#[tokio::test]
async fn equation_limit_rejects_message_before_rendering() {
    let renderer = FakeRenderer::default();
    let request = request_for(&"\\(x\\) ".repeat(3));
    let limits = DaemonRenderLimitsV1 {
        max_equations: 2,
        ..DaemonRenderLimitsV1::default()
    };

    let error = render_message_with_limits_v1(&renderer, request, limits)
        .await
        .unwrap_err();

    assert_eq!(
        error,
        crate::daemon_protocol_v1::error("input_limit_exceeded", false)
    );
    assert!(renderer.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn aggregate_png_limit_stops_later_render_work() {
    let renderer = FakeRenderer::default();
    let request = request_for("\\(one\\) \\(two\\)");
    let limits = DaemonRenderLimitsV1 {
        max_total_png_bytes: 1,
        ..DaemonRenderLimitsV1::default()
    };

    let outcomes = render_message_with_limits_v1(&renderer, request, limits)
        .await
        .unwrap();

    assert_eq!(outcomes.len(), 2);
    assert!(outcomes.iter().all(|outcome| matches!(
        outcome,
        EquationOutcomeV1::Failed {
            error: DaemonErrorV1 { code, .. },
            ..
        } if code == "output_limit_exceeded"
    )));
    assert_eq!(renderer.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn accessibility_limits_fail_before_response_serialization() {
    let renderer = FakeRenderer::default();
    let request = request_for("\\(verbose\\) \\(later\\)");
    let limits = DaemonRenderLimitsV1 {
        max_accessibility_bytes: 20,
        ..DaemonRenderLimitsV1::default()
    };

    let outcomes = render_message_with_limits_v1(&renderer, request, limits)
        .await
        .unwrap();

    assert!(matches!(
        &outcomes[0],
        EquationOutcomeV1::Failed {
            error: DaemonErrorV1 { code, .. },
            ..
        } if code == "output_limit_exceeded"
    ));
    assert!(matches!(&outcomes[1], EquationOutcomeV1::Rendered { .. }));
}

#[tokio::test]
async fn aggregate_accessibility_limit_stops_later_render_work() {
    let renderer = FakeRenderer::default();
    let request = request_for("\\(one\\) \\(two\\)");
    let limits = DaemonRenderLimitsV1 {
        max_total_accessibility_bytes: 1,
        ..DaemonRenderLimitsV1::default()
    };

    let outcomes = render_message_with_limits_v1(&renderer, request, limits)
        .await
        .unwrap();

    assert!(outcomes.iter().all(|outcome| matches!(
        outcome,
        EquationOutcomeV1::Failed {
            error: DaemonErrorV1 { code, .. },
            ..
        } if code == "output_limit_exceeded"
    )));
    assert_eq!(renderer.requests.lock().unwrap().len(), 1);
}

fn request_for(source: &str) -> ValidatedRenderMessageV1 {
    ValidatedRenderMessageV1 {
        id: "message-42".to_owned(),
        source: source.to_owned(),
        inline_dollars: InlineDollarsV1::Smart,
        foreground: latex_render_core::Rgba::opaque(230, 237, 243),
        background: None,
        scale: 2.0,
        max_width_px: 1200,
    }
}
