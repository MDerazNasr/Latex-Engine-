#![doc = "Render contract integration tests."]

mod common;

use latex_render_core::{
    MathRenderer, RenderError, RenderErrorCode, RenderFuture, RenderLimits, RenderRequest, Rgba,
};

use common::{request, result};

#[test]
fn default_request_and_result_satisfy_contract() {
    let limits = RenderLimits::default();

    assert_eq!(request("x^2").validate(&limits), Ok(()));
    assert_eq!(result("v1:key").validate(&limits), Ok(()));
}

#[test]
fn request_rejects_empty_oversized_and_control_input() {
    let limits = RenderLimits {
        max_source_bytes: 4,
        ..RenderLimits::default()
    };

    assert_code(
        request("").validate(&limits),
        RenderErrorCode::InvalidRequest,
    );
    assert_code(
        request("ééé").validate(&limits),
        RenderErrorCode::InputLimitExceeded,
    );
    assert_code(
        request("x\u{0007}").validate(&limits),
        RenderErrorCode::InvalidRequest,
    );
}

#[test]
fn request_rejects_invalid_scale_and_width() {
    let limits = RenderLimits::default();
    let mut value = request("x");
    value.scale = f32::NAN;
    assert_code(value.validate(&limits), RenderErrorCode::InvalidRequest);

    let mut value = request("x");
    value.max_width_px = 0;
    assert_code(value.validate(&limits), RenderErrorCode::InvalidRequest);
}

#[test]
fn result_rejects_invalid_dimensions_baseline_and_accessibility_text() {
    let limits = RenderLimits::default();
    let mut value = result("v1:key");
    value.height_px = 0;
    assert_code(value.validate(&limits), RenderErrorCode::RenderFailed);

    let mut value = result("v1:key");
    value.baseline_px = Some(f32::INFINITY);
    assert_code(value.validate(&limits), RenderErrorCode::RenderFailed);

    let mut value = result("v1:key");
    value.accessibility_text = "unsafe\u{001b}".to_owned();
    assert_code(value.validate(&limits), RenderErrorCode::UnsafeOutput);
}

#[test]
fn public_errors_replace_unsafe_control_characters() {
    let error = RenderError::new(RenderErrorCode::Protocol, "bad\u{001b}message\nnext", false);

    assert_eq!(error.message, "bad\u{fffd}message\u{fffd}next");
    assert!(!error.message.contains('\u{001b}'));
}

#[test]
fn result_rejects_non_utf8_svg_bytes() {
    let mut value = result("v1:key");
    value.svg = vec![0xff, 0xfe];

    assert_code(
        value.validate(&RenderLimits::default()),
        RenderErrorCode::UnsafeOutput,
    );
}

#[test]
fn colors_only_format_when_fully_opaque() {
    assert_eq!(
        Rgba::opaque(10, 11, 12).to_rgb_hex().as_deref(),
        Some("#0a0b0c")
    );
    assert_eq!(Rgba::new(10, 11, 12, 127).to_rgb_hex(), None);
    assert!(Rgba::new(10, 11, 12, 0).is_transparent());
}

#[test]
fn renderer_contract_can_be_used_as_a_trait_object() {
    struct Stub;

    impl MathRenderer for Stub {
        fn render(&self, request: RenderRequest) -> RenderFuture<'_> {
            Box::pin(async move {
                let mut output = result("v1:stub");
                output.accessibility_text = request.source;
                Ok(output)
            })
        }
    }

    let renderer: &dyn MathRenderer = &Stub;
    let future = renderer.render(request("x"));
    drop(future);
}

fn assert_code(result: Result<(), RenderError>, expected: RenderErrorCode) {
    assert_eq!(result.expect_err("validation should fail").code, expected);
}
