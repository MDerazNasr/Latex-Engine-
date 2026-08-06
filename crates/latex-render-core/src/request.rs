//! Validated backend neutral render requests.

use crate::{RenderError, RenderErrorCode, RenderLimits, Rgba};

/// A complete request for rendering one math fragment.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderRequest {
    /// Original math source without delimiters.
    pub source: String,
    /// Whether the expression should use display layout.
    pub display_mode: bool,
    /// Requested foreground color.
    pub foreground: Rgba,
    /// Optional requested background color.
    pub background: Option<Rgba>,
    /// Requested output scale.
    pub scale: f32,
    /// Maximum output width in pixels.
    pub max_width_px: u32,
}

impl RenderRequest {
    /// Validates the request against shared resource limits.
    pub fn validate(&self, limits: &RenderLimits) -> Result<(), RenderError> {
        if self.source.is_empty() {
            return Err(invalid("Source must not be empty"));
        }
        if self.source.len() > limits.max_source_bytes {
            return Err(RenderError::new(
                RenderErrorCode::InputLimitExceeded,
                format!("Source exceeds {} UTF 8 bytes", limits.max_source_bytes),
                false,
            ));
        }
        if self.source.chars().any(is_unsafe_control) {
            return Err(invalid("Source contains an unsupported control character"));
        }
        if !self.scale.is_finite() || self.scale < limits.min_scale || self.scale > limits.max_scale
        {
            return Err(invalid(format!(
                "Scale must be between {} and {}",
                limits.min_scale, limits.max_scale
            )));
        }
        if self.max_width_px == 0 || self.max_width_px > limits.max_width_px {
            return Err(invalid(format!(
                "Maximum width must be between 1 and {} pixels",
                limits.max_width_px
            )));
        }
        Ok(())
    }
}

fn invalid(message: impl Into<String>) -> RenderError {
    RenderError::new(RenderErrorCode::InvalidRequest, message, false)
}

fn is_unsafe_control(character: char) -> bool {
    character.is_control() && !matches!(character, '\n' | '\r' | '\t')
}
