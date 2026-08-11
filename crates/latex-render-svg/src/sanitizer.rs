//! Streaming SVG validation and canonical rewriting.

use latex_render_core::{MAX_SVG_BYTES, RenderError, RenderErrorCode};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;

use crate::policy::sanitize_element;

/// Structural bounds applied while sanitizing one SVG document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SvgSanitizerLimits {
    /// Maximum input and output length in UTF 8 bytes.
    pub max_bytes: usize,
    /// Maximum XML element count.
    pub max_elements: usize,
    /// Maximum element nesting depth.
    pub max_depth: usize,
    /// Maximum attributes on one element.
    pub max_attributes_per_element: usize,
}

impl Default for SvgSanitizerLimits {
    fn default() -> Self {
        Self {
            max_bytes: MAX_SVG_BYTES,
            max_elements: 50_000,
            max_depth: 64,
            max_attributes_per_element: 32,
        }
    }
}

/// SVG bytes that passed the current explicit allowlist policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SanitizedSvg(Vec<u8>);

impl SanitizedSvg {
    /// Returns sanitized standalone SVG bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns sanitized standalone SVG text.
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).expect("sanitized SVG is always valid UTF 8")
    }

    /// Consumes the wrapper and returns its bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

/// Validates and rewrites worker SVG through the MathJax output allowlist.
pub fn sanitize_svg(input: &[u8], limits: SvgSanitizerLimits) -> Result<SanitizedSvg, RenderError> {
    validate_limits(limits)?;
    if input.is_empty() || input.len() > limits.max_bytes {
        return Err(unsafe_svg("SVG input is empty or exceeds its byte limit"));
    }
    let text =
        std::str::from_utf8(input).map_err(|_| unsafe_svg("SVG input must contain valid UTF 8"))?;
    if text.chars().any(is_unsafe_control) {
        return Err(unsafe_svg(
            "SVG input contains an unsupported control character",
        ));
    }

    let mut reader = Reader::from_str(text);
    reader.config_mut().enable_all_checks(true);
    let mut writer = Writer::new(Vec::with_capacity(input.len()));
    let mut depth = 0usize;
    let mut elements = 0usize;
    let mut root_seen = false;
    let mut root_closed = false;

    loop {
        let event = reader
            .read_event()
            .map_err(|_| unsafe_svg("SVG input is not well formed XML"))?;
        match event {
            Event::Start(start) => {
                begin_element(
                    &mut depth,
                    &mut elements,
                    &mut root_seen,
                    root_closed,
                    &start,
                    limits,
                    &mut writer,
                    false,
                )?;
            }
            Event::Empty(start) => {
                begin_element(
                    &mut depth,
                    &mut elements,
                    &mut root_seen,
                    root_closed,
                    &start,
                    limits,
                    &mut writer,
                    true,
                )?;
            }
            Event::End(end) => {
                if depth == 0 {
                    return Err(unsafe_svg("SVG contains an unmatched closing element"));
                }
                writer
                    .write_event(Event::End(end))
                    .map_err(|_| unsafe_svg("SVG output could not be written"))?;
                depth -= 1;
                if depth == 0 {
                    root_closed = true;
                }
            }
            Event::Text(text) if text.as_ref().iter().all(u8::is_ascii_whitespace) => {}
            Event::Eof => break,
            _ => return Err(unsafe_svg("SVG contains a disallowed XML event")),
        }
    }

    if !root_seen || !root_closed || depth != 0 {
        return Err(unsafe_svg("SVG must contain one complete root element"));
    }
    let output = writer.into_inner();
    if output.len() > limits.max_bytes {
        return Err(unsafe_svg("Sanitized SVG exceeds its byte limit"));
    }
    Ok(SanitizedSvg(output))
}

#[allow(clippy::too_many_arguments)]
fn begin_element(
    depth: &mut usize,
    elements: &mut usize,
    root_seen: &mut bool,
    root_closed: bool,
    start: &quick_xml::events::BytesStart<'_>,
    limits: SvgSanitizerLimits,
    writer: &mut Writer<Vec<u8>>,
    empty: bool,
) -> Result<(), RenderError> {
    if root_closed {
        return Err(unsafe_svg("SVG contains content after its root element"));
    }
    *elements = elements.saturating_add(1);
    if *elements > limits.max_elements {
        return Err(unsafe_svg("SVG exceeds its element limit"));
    }
    let next_depth = depth.saturating_add(1);
    if next_depth > limits.max_depth {
        return Err(unsafe_svg("SVG exceeds its nesting depth limit"));
    }
    let root = *depth == 0;
    if root {
        if *root_seen {
            return Err(unsafe_svg("SVG contains more than one root element"));
        }
        *root_seen = true;
    }
    let sanitized = sanitize_element(start, root, limits.max_attributes_per_element)?;
    writer
        .write_event(if empty {
            Event::Empty(sanitized)
        } else {
            Event::Start(sanitized)
        })
        .map_err(|_| unsafe_svg("SVG output could not be written"))?;
    if root && empty {
        return Err(unsafe_svg("SVG root element must not be empty"));
    }
    if !empty {
        *depth = next_depth;
    }
    Ok(())
}

fn validate_limits(limits: SvgSanitizerLimits) -> Result<(), RenderError> {
    if limits.max_bytes == 0
        || limits.max_bytes > MAX_SVG_BYTES
        || limits.max_elements == 0
        || limits.max_depth == 0
        || limits.max_attributes_per_element == 0
    {
        return Err(RenderError::new(
            RenderErrorCode::InvalidRequest,
            "SVG sanitizer limits are invalid",
            false,
        ));
    }
    Ok(())
}

fn is_unsafe_control(character: char) -> bool {
    character.is_control() && !matches!(character, '\n' | '\r' | '\t')
}

pub(crate) fn unsafe_svg(message: impl Into<String>) -> RenderError {
    RenderError::new(RenderErrorCode::UnsafeOutput, message, false)
}
