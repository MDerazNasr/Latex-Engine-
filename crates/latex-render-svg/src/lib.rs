#![doc = "Fail closed SVG allowlist and deterministic PNG rasterization."]

mod policy;
mod rasterizer;
mod sanitizer;

pub use rasterizer::{
    FittedRasterRequest, PngImage, RasterLimits, RasterRect, RasterRequest, rasterize_svg,
    rasterize_svg_fitted,
};
pub use sanitizer::{SanitizedSvg, SvgSanitizerLimits, sanitize_svg};

/// The sanitizer policy version used in cache invalidation.
pub const SVG_POLICY_VERSION: u32 = 1;

/// The sanitizer policy label used in cache invalidation.
pub const SVG_POLICY_VERSION_LABEL: &str = "svg-allowlist-1";

/// The rasterizer implementation version used in cache invalidation.
pub const RASTERIZER_VERSION: &str = "resvg-0.48.1-policy-1";

/// The fitted rasterizer version used in terminal cache invalidation.
pub const FITTED_RASTERIZER_VERSION: &str = "resvg-0.48.1-fit-policy-1";
