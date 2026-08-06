//! Stable errors returned by renderer implementations.

use std::error::Error;
use std::fmt::{Display, Formatter};

/// Machine readable render failure categories.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum RenderErrorCode {
    /// The request does not satisfy the public contract.
    InvalidRequest,
    /// Input exceeded a configured resource bound.
    InputLimitExceeded,
    /// Output exceeded a configured resource bound.
    OutputLimitExceeded,
    /// The renderer rejected the math syntax.
    InvalidTex,
    /// The worker protocol was malformed or incompatible.
    Protocol,
    /// No usable renderer worker is available.
    WorkerUnavailable,
    /// Rendering exceeded its deadline.
    Timeout,
    /// The bounded render queue is full.
    QueueFull,
    /// Work was cancelled because its result is no longer relevant.
    Cancelled,
    /// Generated output violated a safety policy.
    UnsafeOutput,
    /// Rendering failed without a more specific public category.
    RenderFailed,
}

/// A source free error safe to show in diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderError {
    /// Machine readable category.
    pub code: RenderErrorCode,
    /// Stable public explanation that excludes equation source.
    pub message: String,
    /// Whether retrying on a healthy worker may succeed.
    pub retryable: bool,
    /// Optional byte position reported by the renderer.
    pub position: Option<usize>,
}

impl RenderError {
    /// Creates an error without a source position.
    pub fn new(code: RenderErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code,
            message: sanitize_message(message.into()),
            retryable,
            position: None,
        }
    }

    /// Adds a byte position to an error.
    pub const fn with_position(mut self, position: usize) -> Self {
        self.position = Some(position);
        self
    }
}

impl Display for RenderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for RenderError {}

fn sanitize_message(message: String) -> String {
    message
        .chars()
        .map(|character| {
            if character.is_control() {
                '\u{fffd}'
            } else {
                character
            }
        })
        .collect()
}
