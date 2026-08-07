#![doc = "SVG rasterizer integration tests."]

use latex_render_core::RenderErrorCode;
use latex_render_svg::{
    RasterLimits, RasterRequest, SvgSanitizerLimits, rasterize_svg, sanitize_svg,
};
use tiny_skia::Pixmap;

const FIXTURE: &[u8] = include_bytes!("../../../fixtures/terminal/quadratic-formula.svg");

#[test]
fn fixture_rasterizes_to_deterministic_transparent_png() {
    let svg =
        sanitize_svg(FIXTURE, SvgSanitizerLimits::default()).expect("fixture should sanitize");
    let request = RasterRequest {
        width_px: 512,
        height_px: 128,
    };

    let first =
        rasterize_svg(&svg, request, RasterLimits::default()).expect("fixture should rasterize");
    let second =
        rasterize_svg(&svg, request, RasterLimits::default()).expect("repeat should rasterize");

    assert_eq!(first, second);
    assert_eq!(first.width_px, 512);
    assert_eq!(first.height_px, 128);
    assert!(first.bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    let pixmap = Pixmap::decode_png(&first.bytes).expect("PNG should decode");
    assert_eq!(pixmap.width(), 512);
    assert_eq!(pixmap.height(), 128);
    assert!(pixmap.pixels().iter().any(|pixel| pixel.alpha() > 0));
}

#[test]
fn zero_oversized_and_excessive_allocation_requests_fail() {
    let svg =
        sanitize_svg(FIXTURE, SvgSanitizerLimits::default()).expect("fixture should sanitize");
    let cases = [
        RasterRequest {
            width_px: 0,
            height_px: 1,
        },
        RasterRequest {
            width_px: 4097,
            height_px: 1,
        },
        RasterRequest {
            width_px: 1,
            height_px: 2049,
        },
    ];
    for request in cases {
        let error = rasterize_svg(&svg, request, RasterLimits::default())
            .expect_err("invalid dimensions should fail");
        assert_eq!(error.code, RenderErrorCode::OutputLimitExceeded);
    }

    let limits = RasterLimits {
        max_rgba_bytes: 15,
        ..RasterLimits::default()
    };
    let error = rasterize_svg(
        &svg,
        RasterRequest {
            width_px: 2,
            height_px: 2,
        },
        limits,
    )
    .expect_err("allocation bound should fail");
    assert_eq!(error.code, RenderErrorCode::OutputLimitExceeded);
}

#[test]
fn encoded_png_limit_is_enforced() {
    let svg =
        sanitize_svg(FIXTURE, SvgSanitizerLimits::default()).expect("fixture should sanitize");
    let limits = RasterLimits {
        max_png_bytes: 1,
        ..RasterLimits::default()
    };

    let error = rasterize_svg(
        &svg,
        RasterRequest {
            width_px: 64,
            height_px: 32,
        },
        limits,
    )
    .expect_err("PNG limit should fail");

    assert_eq!(error.code, RenderErrorCode::OutputLimitExceeded);
}
