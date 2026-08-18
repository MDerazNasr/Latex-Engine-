#![doc = "SVG rasterizer integration tests."]

use latex_render_core::RenderErrorCode;
use latex_render_svg::{
    FittedRasterRequest, RasterLimits, RasterRect, RasterRequest, SvgSanitizerLimits,
    rasterize_svg, rasterize_svg_fitted, sanitize_svg, validate_png,
};
use tiny_skia::Pixmap;

const FIXTURE: &[u8] = include_bytes!("../../../fixtures/terminal/quadratic-formula.svg");
const PNG_FIXTURE: &[u8] = include_bytes!("../../../fixtures/terminal/quadratic-formula.png");
const RECTANGLE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50" viewBox="0 0 100 50" role="img" focusable="false" style="color:#000000"><rect x="0" y="0" width="100" height="50" fill="currentColor"/></svg>"##;

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
    assert_eq!(
        validate_png(&first.bytes, RasterLimits::default()),
        Ok(request)
    );
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

#[test]
fn fitted_raster_is_deterministic_and_preserves_transparent_padding() {
    let svg =
        sanitize_svg(RECTANGLE, SvgSanitizerLimits::default()).expect("rectangle should sanitize");
    let request = FittedRasterRequest {
        canvas_width_px: 100,
        canvas_height_px: 100,
        content: RasterRect {
            x_px: 20,
            y_px: 10,
            width_px: 60,
            height_px: 80,
        },
    };

    let first = rasterize_svg_fitted(&svg, request, RasterLimits::default())
        .expect("fitted raster should render");
    let second =
        rasterize_svg_fitted(&svg, request, RasterLimits::default()).expect("repeat should render");
    let pixmap = Pixmap::decode_png(&first.bytes).expect("PNG should decode");

    assert_eq!(first, second);
    assert_eq!(first.width_px, 100);
    assert_eq!(first.height_px, 100);
    assert_eq!(alpha_bounds(&pixmap), Some((20, 35, 79, 64)));
    assert_eq!(pixmap.pixel(0, 0).expect("corner should exist").alpha(), 0);
}

#[test]
fn fitted_raster_does_not_upscale_content() {
    let svg =
        sanitize_svg(RECTANGLE, SvgSanitizerLimits::default()).expect("rectangle should sanitize");
    let image = rasterize_svg_fitted(
        &svg,
        FittedRasterRequest {
            canvas_width_px: 220,
            canvas_height_px: 120,
            content: RasterRect {
                x_px: 10,
                y_px: 10,
                width_px: 200,
                height_px: 100,
            },
        },
        RasterLimits::default(),
    )
    .expect("fitted raster should render");
    let pixmap = Pixmap::decode_png(&image.bytes).expect("PNG should decode");

    assert_eq!(alpha_bounds(&pixmap), Some((60, 35, 159, 84)));
}

#[test]
fn invalid_fitted_content_rectangles_fail_closed() {
    let svg =
        sanitize_svg(RECTANGLE, SvgSanitizerLimits::default()).expect("rectangle should sanitize");
    let cases = [
        RasterRect {
            x_px: 0,
            y_px: 0,
            width_px: 0,
            height_px: 1,
        },
        RasterRect {
            x_px: 90,
            y_px: 0,
            width_px: 11,
            height_px: 1,
        },
        RasterRect {
            x_px: u32::MAX,
            y_px: 0,
            width_px: 2,
            height_px: 1,
        },
    ];

    for content in cases {
        let error = rasterize_svg_fitted(
            &svg,
            FittedRasterRequest {
                canvas_width_px: 100,
                canvas_height_px: 100,
                content,
            },
            RasterLimits::default(),
        )
        .expect_err("invalid content should fail");
        assert_eq!(error.code, RenderErrorCode::OutputLimitExceeded);
    }
}

#[test]
fn malformed_and_oversized_png_bytes_fail_before_publication() {
    let malformed = png_header(1, 1);
    let error = validate_png(&malformed, RasterLimits::default())
        .expect_err("invalid CRC and missing image data should fail");
    assert_eq!(error.code, RenderErrorCode::UnsafeOutput);

    let oversized = png_header(4097, 1);
    let error = validate_png(&oversized, RasterLimits::default())
        .expect_err("oversized header should fail before decode");
    assert_eq!(error.code, RenderErrorCode::OutputLimitExceeded);

    let limits = RasterLimits {
        max_png_bytes: 8,
        ..RasterLimits::default()
    };
    let error = validate_png(PNG_FIXTURE, limits).expect_err("encoded byte limit should apply");
    assert_eq!(error.code, RenderErrorCode::OutputLimitExceeded);
}

fn alpha_bounds(pixmap: &Pixmap) -> Option<(u32, u32, u32, u32)> {
    let mut bounds: Option<(u32, u32, u32, u32)> = None;
    for y in 0..pixmap.height() {
        for x in 0..pixmap.width() {
            if pixmap.pixel(x, y).is_some_and(|pixel| pixel.alpha() > 0) {
                bounds = Some(match bounds {
                    Some((min_x, min_y, max_x, max_y)) => {
                        (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
                    }
                    None => (x, y, x, y),
                });
            }
        }
    }
    bounds
}

fn png_header(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(33);
    bytes.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    bytes.extend_from_slice(&13_u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
    bytes.extend_from_slice(&[0; 4]);
    bytes
}
