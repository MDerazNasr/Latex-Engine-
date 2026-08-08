//! Explicit MathJax SVG element, attribute, and value policy.

use std::collections::HashSet;

use latex_render_core::RenderError;
use quick_xml::XmlVersion;
use quick_xml::events::BytesStart;
use svgtypes::{PathParser, TransformListParser, ViewBox};

use crate::sanitizer::unsafe_svg;

const STRIPPED_METADATA: &[&str] = &["data-c", "data-latex", "data-mjx-texclass", "data-mml-node"];

pub(crate) fn sanitize_element(
    input: &BytesStart<'_>,
    root: bool,
    max_attributes: usize,
) -> Result<BytesStart<'static>, RenderError> {
    let name = std::str::from_utf8(input.name().as_ref())
        .map_err(|_| unsafe_svg("SVG element name is not valid UTF 8"))?
        .to_owned();
    if !matches!(name.as_str(), "svg" | "g" | "path" | "rect") || (root != (name == "svg")) {
        return Err(unsafe_svg("SVG contains a disallowed element"));
    }

    let mut output = BytesStart::new(name.clone());
    let mut seen = HashSet::new();
    let mut count = 0usize;
    for attribute in input.attributes() {
        count = count.saturating_add(1);
        if count > max_attributes {
            return Err(unsafe_svg("SVG element exceeds its attribute limit"));
        }
        let attribute = attribute.map_err(|_| unsafe_svg("SVG contains a malformed attribute"))?;
        let key = std::str::from_utf8(attribute.key.as_ref())
            .map_err(|_| unsafe_svg("SVG attribute name is not valid UTF 8"))?;
        if !seen.insert(key.to_owned()) {
            return Err(unsafe_svg("SVG contains a duplicate attribute"));
        }
        if STRIPPED_METADATA.contains(&key) {
            continue;
        }
        let value = attribute
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(|_| unsafe_svg("SVG attribute value is malformed"))?
            .into_owned();
        validate_attribute(&name, key, &value)?;
        output.push_attribute((key, value.as_str()));
    }
    let required: &[&str] = match name.as_str() {
        "svg" => &[
            "xmlns",
            "width",
            "height",
            "viewBox",
            "role",
            "focusable",
            "style",
        ],
        "path" => &["d"],
        "rect" => &["width", "height", "x", "y"],
        _ => &[],
    };
    if required.iter().any(|key| !seen.contains(*key)) {
        return Err(unsafe_svg("SVG element is missing a required attribute"));
    }
    Ok(output)
}

fn validate_attribute(element: &str, key: &str, value: &str) -> Result<(), RenderError> {
    let valid = match (element, key) {
        ("svg", "xmlns") => value == "http://www.w3.org/2000/svg",
        ("svg", "width" | "height") => valid_positive_length(value, true),
        ("svg", "viewBox") => value.parse::<ViewBox>().is_ok(),
        ("svg", "role") => value == "img",
        ("svg", "focusable") => value == "false",
        ("svg", "style") => valid_root_style(value),
        ("g" | "path" | "rect", "transform") => valid_transform(value),
        ("g" | "path" | "rect", "fill" | "stroke") => valid_paint(value),
        ("g" | "path" | "rect", "stroke-width") => valid_nonnegative_number(value),
        ("path", "d") => valid_path(value),
        ("rect", "width" | "height") => valid_positive_length(value, false),
        ("rect", "x" | "y") => valid_number(value),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(unsafe_svg("SVG contains a disallowed attribute or value"))
    }
}

fn valid_root_style(style: &str) -> bool {
    if style.len() > 256 {
        return false;
    }
    let mut seen = HashSet::new();
    for declaration in style.split(';').filter(|value| !value.trim().is_empty()) {
        let Some((property, value)) = declaration.split_once(':') else {
            return false;
        };
        let property = property.trim();
        let value = value.trim();
        if !seen.insert(property) {
            return false;
        }
        let valid = match property {
            "color" | "background-color" => valid_hex_color(value),
            "vertical-align" => valid_signed_length(value),
            _ => false,
        };
        if !valid {
            return false;
        }
    }
    !seen.is_empty()
}

fn valid_transform(value: &str) -> bool {
    if value.is_empty() || value.len() > 512 {
        return false;
    }
    let mut count = 0usize;
    for token in TransformListParser::from(value) {
        if token.is_err() {
            return false;
        }
        count += 1;
        if count > 64 {
            return false;
        }
    }
    count > 0
}

fn valid_path(value: &str) -> bool {
    if value.is_empty() {
        // Empty paths remain safe because MathJax uses them as spacing glyphs with no geometry.
        return true;
    }
    let mut count = 0usize;
    for segment in PathParser::from(value) {
        if segment.is_err() {
            return false;
        }
        count += 1;
        if count > 100_000 {
            return false;
        }
    }
    count > 0
}

fn valid_paint(value: &str) -> bool {
    matches!(value, "currentColor" | "none") || valid_hex_color(value)
}

fn valid_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}

fn valid_positive_length(value: &str, allow_unit: bool) -> bool {
    parse_length(value, allow_unit).is_some_and(|number| number > 0.0)
}

fn valid_signed_length(value: &str) -> bool {
    parse_length(value, true).is_some()
}

fn parse_length(value: &str, allow_unit: bool) -> Option<f64> {
    let number = if allow_unit {
        ["px", "ex", "em"]
            .iter()
            .find_map(|unit| value.strip_suffix(unit))
            .unwrap_or(value)
    } else {
        value
    };
    if !allow_unit && number != value {
        return None;
    }
    parse_number(number)
}

fn valid_nonnegative_number(value: &str) -> bool {
    parse_number(value).is_some_and(|number| number >= 0.0)
}

fn valid_number(value: &str) -> bool {
    parse_number(value).is_some()
}

fn parse_number(value: &str) -> Option<f64> {
    if value.is_empty() || value.len() > 64 || value.bytes().any(|byte| byte.is_ascii_whitespace())
    {
        return None;
    }
    value
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
}
