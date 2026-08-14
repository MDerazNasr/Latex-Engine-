//! Bounded SVG to PNG rasterization.

use latex_render_core::{MAX_HEIGHT_PX, MAX_WIDTH_PX, RenderError, RenderErrorCode};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};

use crate::SanitizedSvg;

/// Maximum uncompressed RGBA allocation for one raster.
pub const MAX_RGBA_BYTES: usize = MAX_WIDTH_PX as usize * MAX_HEIGHT_PX as usize * 4;

/// Maximum encoded PNG bytes accepted from the rasterizer.
pub const MAX_PNG_BYTES: usize = MAX_RGBA_BYTES + 1024 * 1024;

/// Resource limits for one raster operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RasterLimits {
    /// Maximum output width in pixels.
    pub max_width_px: u32,
    /// Maximum output height in pixels.
    pub max_height_px: u32,
    /// Maximum uncompressed RGBA allocation.
    pub max_rgba_bytes: usize,
    /// Maximum encoded PNG length.
    pub max_png_bytes: usize,
}

impl Default for RasterLimits {
    fn default() -> Self {
        Self {
            max_width_px: MAX_WIDTH_PX,
            max_height_px: MAX_HEIGHT_PX,
            max_rgba_bytes: MAX_RGBA_BYTES,
            max_png_bytes: MAX_PNG_BYTES,
        }
    }
}

/// Requested raster dimensions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RasterRequest {
    /// Output width in pixels.
    pub width_px: u32,
    /// Output height in pixels.
    pub height_px: u32,
}

/// Pixel rectangle reserved for equation content inside a raster canvas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RasterRect {
    /// Horizontal offset from the canvas origin.
    pub x_px: u32,
    /// Vertical offset from the canvas origin.
    pub y_px: u32,
    /// Available content width.
    pub width_px: u32,
    /// Available content height.
    pub height_px: u32,
}

/// Requested canvas and content rectangle for aspect preserving rasterization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FittedRasterRequest {
    /// Transparent canvas width.
    pub canvas_width_px: u32,
    /// Transparent canvas height.
    pub canvas_height_px: u32,
    /// Rectangle that bounds uniformly scaled equation content.
    pub content: RasterRect,
}

/// Encoded transparent PNG and its dimensions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PngImage {
    /// Encoded PNG bytes.
    pub bytes: Vec<u8>,
    /// Image width in pixels.
    pub width_px: u32,
    /// Image height in pixels.
    pub height_px: u32,
}

/// Rasterizes sanitized SVG without external resources or system fonts.
pub fn rasterize_svg(
    svg: &SanitizedSvg,
    request: RasterRequest,
    limits: RasterLimits,
) -> Result<PngImage, RenderError> {
    validate_request(request, limits)?;
    let tree = parse_tree(svg)?;
    let mut pixmap = allocate_pixmap(request)?;
    let size = tree.size();
    let transform = Transform::from_scale(
        request.width_px as f32 / size.width(),
        request.height_px as f32 / size.height(),
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    encode_pixmap(pixmap, request, limits)
}

/// Rasterizes sanitized SVG uniformly inside a transparent canvas rectangle.
pub fn rasterize_svg_fitted(
    svg: &SanitizedSvg,
    request: FittedRasterRequest,
    limits: RasterLimits,
) -> Result<PngImage, RenderError> {
    let canvas = RasterRequest {
        width_px: request.canvas_width_px,
        height_px: request.canvas_height_px,
    };
    validate_request(canvas, limits)?;
    validate_content_rect(request)?;

    let tree = parse_tree(svg)?;
    let mut pixmap = allocate_pixmap(canvas)?;
    let tree_size = tree.size();
    let scale = 1.0_f32.min(
        (request.content.width_px as f32 / tree_size.width())
            .min(request.content.height_px as f32 / tree_size.height()),
    );
    let rendered_width = tree_size.width() * scale;
    let rendered_height = tree_size.height() * scale;
    let translate_x =
        request.content.x_px as f32 + (request.content.width_px as f32 - rendered_width) / 2.0;
    let translate_y =
        request.content.y_px as f32 + (request.content.height_px as f32 - rendered_height) / 2.0;
    let transform = Transform::from_row(scale, 0.0, 0.0, scale, translate_x, translate_y);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    encode_pixmap(pixmap, canvas, limits)
}

/// Validates bounded PNG bytes and returns their decoded dimensions.
pub fn validate_png(png: &[u8], limits: RasterLimits) -> Result<RasterRequest, RenderError> {
    if png.len() > limits.max_png_bytes {
        return Err(RenderError::new(
            RenderErrorCode::OutputLimitExceeded,
            "Encoded PNG exceeds its byte limit",
            false,
        ));
    }
    let request = png_header_dimensions(png)?;
    validate_request(request, limits)?;
    let pixmap = Pixmap::decode_png(png).map_err(|_| invalid_png())?;
    if pixmap.width() != request.width_px || pixmap.height() != request.height_px {
        return Err(invalid_png());
    }
    Ok(request)
}

fn parse_tree(svg: &SanitizedSvg) -> Result<Tree, RenderError> {
    let options = Options {
        resources_dir: None,
        font_size: 16.0,
        ..Options::default()
    };
    Tree::from_str(svg.as_str(), &options).map_err(|_| {
        RenderError::new(
            RenderErrorCode::RenderFailed,
            "Sanitized SVG could not be parsed by the rasterizer",
            false,
        )
    })
}

fn allocate_pixmap(request: RasterRequest) -> Result<Pixmap, RenderError> {
    Pixmap::new(request.width_px, request.height_px).ok_or_else(|| {
        RenderError::new(
            RenderErrorCode::OutputLimitExceeded,
            "Raster pixel buffer could not be allocated",
            false,
        )
    })
}

fn encode_pixmap(
    pixmap: Pixmap,
    request: RasterRequest,
    limits: RasterLimits,
) -> Result<PngImage, RenderError> {
    let bytes = pixmap.encode_png().map_err(|_| {
        RenderError::new(
            RenderErrorCode::RenderFailed,
            "Raster output could not be encoded as PNG",
            false,
        )
    })?;
    if bytes.len() > limits.max_png_bytes {
        return Err(RenderError::new(
            RenderErrorCode::OutputLimitExceeded,
            "Encoded PNG exceeds its byte limit",
            false,
        ));
    }
    Ok(PngImage {
        bytes,
        width_px: request.width_px,
        height_px: request.height_px,
    })
}

fn validate_content_rect(request: FittedRasterRequest) -> Result<(), RenderError> {
    let right = request.content.x_px.checked_add(request.content.width_px);
    let bottom = request.content.y_px.checked_add(request.content.height_px);
    if request.content.width_px == 0
        || request.content.height_px == 0
        || right.is_none_or(|edge| edge > request.canvas_width_px)
        || bottom.is_none_or(|edge| edge > request.canvas_height_px)
    {
        return Err(RenderError::new(
            RenderErrorCode::OutputLimitExceeded,
            "Raster content rectangle falls outside its canvas",
            false,
        ));
    }
    Ok(())
}

fn png_header_dimensions(png: &[u8]) -> Result<RasterRequest, RenderError> {
    const PNG_HEADER_BYTES: usize = 33;
    const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
    if png.len() < PNG_HEADER_BYTES
        || !png.starts_with(PNG_SIGNATURE)
        || png[8..12] != [0, 0, 0, 13]
        || &png[12..16] != b"IHDR"
    {
        return Err(invalid_png());
    }
    Ok(RasterRequest {
        width_px: u32::from_be_bytes(png[16..20].try_into().expect("width slice is exact")),
        height_px: u32::from_be_bytes(png[20..24].try_into().expect("height slice is exact")),
    })
}

fn invalid_png() -> RenderError {
    RenderError::new(
        RenderErrorCode::UnsafeOutput,
        "Raster PNG is malformed",
        false,
    )
}

fn validate_request(request: RasterRequest, limits: RasterLimits) -> Result<(), RenderError> {
    let rgba_bytes = (request.width_px as usize)
        .checked_mul(request.height_px as usize)
        .and_then(|pixels| pixels.checked_mul(4));
    if request.width_px == 0
        || request.width_px > limits.max_width_px
        || request.height_px == 0
        || request.height_px > limits.max_height_px
        || rgba_bytes.is_none_or(|bytes| bytes > limits.max_rgba_bytes)
        || limits.max_png_bytes == 0
    {
        return Err(RenderError::new(
            RenderErrorCode::OutputLimitExceeded,
            "Raster dimensions exceed the configured limits",
            false,
        ));
    }
    Ok(())
}
