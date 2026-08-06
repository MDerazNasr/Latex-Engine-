//! Object safe asynchronous renderer boundary.

use std::future::Future;
use std::pin::Pin;

use crate::{RenderError, RenderRequest, RenderedMath};

/// A boxed render operation that can complete away from the caller thread.
pub type RenderFuture<'a> =
    Pin<Box<dyn Future<Output = Result<RenderedMath, RenderError>> + Send + 'a>>;

/// A backend that converts one validated math fragment into sanitized SVG.
pub trait MathRenderer: Send + Sync {
    /// Renders one request without blocking the caller thread.
    fn render(&self, request: RenderRequest) -> RenderFuture<'_>;
}
