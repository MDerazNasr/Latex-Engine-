#![doc = "Backend neutral contracts for rendering untrusted math fragments."]

mod cache;
mod cache_key;
mod color;
mod error;
mod limits;
mod renderer;
mod request;
mod result;

pub use cache::{CacheInsert, CacheLimits, CacheStats, RenderCache};
pub use cache_key::{CacheKeyContext, derive_cache_key};
pub use color::Rgba;
pub use error::{RenderError, RenderErrorCode};
pub use limits::{
    MAX_HEIGHT_PX, MAX_JSON_LINE_BYTES, MAX_SCALE, MAX_SOURCE_BYTES, MAX_SVG_BYTES, MAX_WIDTH_PX,
    MIN_SCALE, RenderLimits,
};
pub use renderer::{MathRenderer, RenderFuture};
pub use request::RenderRequest;
pub use result::RenderedMath;

/// The stable public contract version used in cache and adapter boundaries.
pub const RENDER_CORE_VERSION: u32 = 1;
