#![doc = "Fail closed SVG allowlist and deterministic PNG rasterization."]

/// The sanitizer policy version used in cache invalidation.
pub const SVG_POLICY_VERSION: u32 = 1;

/// The rasterizer implementation version used in cache invalidation.
pub const RASTERIZER_VERSION: &str = "resvg-0.48.1-policy-1";
