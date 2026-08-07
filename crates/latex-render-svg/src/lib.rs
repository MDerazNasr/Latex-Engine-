#![doc = "Fail closed SVG allowlist and deterministic PNG rasterization."]

mod policy;
mod rasterizer;
mod sanitizer;

pub use rasterizer::{PngImage, RasterLimits, RasterRequest, rasterize_svg};
pub use sanitizer::{SanitizedSvg, SvgSanitizerLimits, sanitize_svg};

/// The sanitizer policy version used in cache invalidation.
pub const SVG_POLICY_VERSION: u32 = 1;

/// The rasterizer implementation version used in cache invalidation.
pub const RASTERIZER_VERSION: &str = "resvg-0.48.1-policy-1";
