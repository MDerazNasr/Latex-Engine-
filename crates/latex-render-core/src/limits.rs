//! Shared safety limits for render requests and results.

/// Maximum UTF 8 bytes accepted for one math fragment.
pub const MAX_SOURCE_BYTES: usize = 16 * 1024;

/// Maximum UTF 8 bytes accepted for one worker protocol line.
pub const MAX_JSON_LINE_BYTES: usize = 64 * 1024;

/// Maximum UTF 8 bytes accepted for one SVG result.
pub const MAX_SVG_BYTES: usize = 2 * 1024 * 1024;

/// Maximum output width in pixels.
pub const MAX_WIDTH_PX: u32 = 4096;

/// Maximum output height in pixels.
pub const MAX_HEIGHT_PX: u32 = 2048;

/// Smallest accepted render scale.
pub const MIN_SCALE: f32 = 0.5;

/// Largest accepted render scale.
pub const MAX_SCALE: f32 = 4.0;

/// Limits applied at the backend neutral contract boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderLimits {
    /// Maximum source length in UTF 8 bytes.
    pub max_source_bytes: usize,
    /// Maximum worker protocol line length in UTF 8 bytes.
    pub max_json_line_bytes: usize,
    /// Maximum SVG length in UTF 8 bytes.
    pub max_svg_bytes: usize,
    /// Maximum output width in pixels.
    pub max_width_px: u32,
    /// Maximum output height in pixels.
    pub max_height_px: u32,
    /// Smallest accepted render scale.
    pub min_scale: f32,
    /// Largest accepted render scale.
    pub max_scale: f32,
}

impl Default for RenderLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: MAX_SOURCE_BYTES,
            max_json_line_bytes: MAX_JSON_LINE_BYTES,
            max_svg_bytes: MAX_SVG_BYTES,
            max_width_px: MAX_WIDTH_PX,
            max_height_px: MAX_HEIGHT_PX,
            min_scale: MIN_SCALE,
            max_scale: MAX_SCALE,
        }
    }
}
