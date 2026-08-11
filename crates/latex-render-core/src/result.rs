//! Backend neutral render output.

use crate::{RenderError, RenderErrorCode, RenderLimits};

/// Sanitized vector output and layout metadata for one expression.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderedMath {
    /// Standalone sanitized SVG bytes.
    pub svg: Vec<u8>,
    /// Output width in pixels.
    pub width_px: u32,
    /// Output height in pixels.
    pub height_px: u32,
    /// Optional baseline measured from the top edge.
    pub baseline_px: Option<f32>,
    /// Plain text representation retained for accessibility.
    pub accessibility_text: String,
    /// Stable key covering every rendering input.
    pub cache_key: String,
}

impl RenderedMath {
    /// Validates resource limits and layout metadata.
    pub fn validate(&self, limits: &RenderLimits) -> Result<(), RenderError> {
        if self.svg.is_empty() {
            return Err(invalid("SVG output must not be empty"));
        }
        if self.svg.len() > limits.max_svg_bytes {
            return Err(RenderError::new(
                RenderErrorCode::OutputLimitExceeded,
                format!("SVG exceeds {} UTF 8 bytes", limits.max_svg_bytes),
                false,
            ));
        }
        if std::str::from_utf8(&self.svg).is_err() {
            return Err(RenderError::new(
                RenderErrorCode::UnsafeOutput,
                "SVG output must contain valid UTF 8",
                false,
            ));
        }
        if self.width_px == 0 || self.width_px > limits.max_width_px {
            return Err(invalid("SVG width is outside the configured bounds"));
        }
        if self.height_px == 0 || self.height_px > limits.max_height_px {
            return Err(invalid("SVG height is outside the configured bounds"));
        }
        if self.baseline_px.is_some_and(|baseline| {
            !baseline.is_finite() || baseline < 0.0 || baseline > self.height_px as f32
        }) {
            return Err(invalid("SVG baseline is outside the output height"));
        }
        if self.accessibility_text.chars().any(is_unsafe_control) {
            return Err(RenderError::new(
                RenderErrorCode::UnsafeOutput,
                "Accessibility text contains an unsupported control character",
                false,
            ));
        }
        if self.cache_key.is_empty() || self.cache_key.len() > 128 {
            return Err(invalid("Cache key must contain 1 to 128 bytes"));
        }
        Ok(())
    }

    pub(crate) fn estimated_size_bytes(&self) -> usize {
        self.svg
            .len()
            .saturating_add(self.accessibility_text.len())
            .saturating_add(self.cache_key.len())
    }
}

fn invalid(message: impl Into<String>) -> RenderError {
    RenderError::new(RenderErrorCode::RenderFailed, message, false)
}

fn is_unsafe_control(character: char) -> bool {
    character.is_control() && !matches!(character, '\n' | '\r' | '\t')
}
