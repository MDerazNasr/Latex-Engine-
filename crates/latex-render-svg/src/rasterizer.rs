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
    let options = Options {
        resources_dir: None,
        font_size: 16.0,
        ..Options::default()
    };
    let tree = Tree::from_str(svg.as_str(), &options).map_err(|_| {
        RenderError::new(
            RenderErrorCode::RenderFailed,
            "Sanitized SVG could not be parsed by the rasterizer",
            false,
        )
    })?;
    let mut pixmap = Pixmap::new(request.width_px, request.height_px).ok_or_else(|| {
        RenderError::new(
            RenderErrorCode::OutputLimitExceeded,
            "Raster pixel buffer could not be allocated",
            false,
        )
    })?;
    let size = tree.size();
    let transform = Transform::from_scale(
        request.width_px as f32 / size.width(),
        request.height_px as f32 / size.height(),
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());
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
