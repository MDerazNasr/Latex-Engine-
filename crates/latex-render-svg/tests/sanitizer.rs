#![doc = "SVG sanitizer integration tests."]

use latex_render_core::RenderErrorCode;
use latex_render_svg::{SvgSanitizerLimits, sanitize_svg};

const FIXTURE: &[u8] = include_bytes!("../../../fixtures/terminal/quadratic-formula.svg");

#[test]
fn mathjax_fixture_is_allowed_and_metadata_is_removed() {
    let sanitized =
        sanitize_svg(FIXTURE, SvgSanitizerLimits::default()).expect("MathJax fixture should pass");
    let output = sanitized.as_str();

    assert!(output.starts_with("<svg "));
    assert!(output.ends_with("</svg>"));
    assert!(!output.contains("data-latex"));
    assert!(!output.contains("data-mml-node"));
    assert!(!output.contains(r"\frac"));
    assert!(output.len() < FIXTURE.len());
}

#[test]
fn active_elements_and_attributes_fail_closed() {
    let cases = [
        root("<script></script>"),
        root("<foreignObject></foreignObject>"),
        root(r#"<image href="file:///etc/passwd"/>"#),
        root(r##"<use href="#glyph"/>"##),
        root(r#"<path d="M0 0" onclick="alert(1)"/>"#),
        root(r#"<path d="M0 0" style="fill:url(https://example.com/x)"/>"#),
        root(r#"<path d="M0 0" data-unknown="value"/>"#),
    ];

    for value in cases {
        assert_unsafe(value.as_bytes());
    }
}

#[test]
fn declarations_entities_and_non_whitespace_text_are_rejected() {
    assert_unsafe(br#"<!DOCTYPE svg [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><svg></svg>"#);
    assert_unsafe(root("secret text").as_bytes());
    assert_unsafe(root("<![CDATA[value]]>").as_bytes());
    assert_unsafe(root("<?worker value?>").as_bytes());
}

#[test]
fn empty_malformed_oversized_and_control_input_are_rejected() {
    assert_unsafe(b"");
    assert_unsafe(b"<svg>");
    assert_unsafe(b"\xff\xfe");
    assert_unsafe(root("\u{001b}").as_bytes());

    let limits = SvgSanitizerLimits {
        max_bytes: 8,
        ..SvgSanitizerLimits::default()
    };
    let error = sanitize_svg(root("").as_bytes(), limits).expect_err("oversized input should fail");
    assert_eq!(error.code, RenderErrorCode::UnsafeOutput);
}

#[test]
fn structural_limits_are_enforced() {
    let depth = SvgSanitizerLimits {
        max_depth: 2,
        ..SvgSanitizerLimits::default()
    };
    let error =
        sanitize_svg(root("<g><g></g></g>").as_bytes(), depth).expect_err("deep SVG should fail");
    assert_eq!(error.code, RenderErrorCode::UnsafeOutput);

    let elements = SvgSanitizerLimits {
        max_elements: 2,
        ..SvgSanitizerLimits::default()
    };
    let error =
        sanitize_svg(root("<g/><g/>").as_bytes(), elements).expect_err("element count should fail");
    assert_eq!(error.code, RenderErrorCode::UnsafeOutput);

    let attributes = SvgSanitizerLimits {
        max_attributes_per_element: 6,
        ..SvgSanitizerLimits::default()
    };
    let error =
        sanitize_svg(root("").as_bytes(), attributes).expect_err("attribute count should fail");
    assert_eq!(error.code, RenderErrorCode::UnsafeOutput);
}

#[test]
fn malformed_geometry_and_css_values_are_rejected() {
    let cases = [
        document_with_style("color: url(https://example.com/x)"),
        document_with_style("unknown: #ffffff"),
        document_with_path("M not-a-path"),
        root(r#"<g transform="url(file:///tmp/x)"></g>"#),
        root(r#"<rect width="-1" height="2" x="0" y="0"/>"#),
    ];

    for value in cases {
        assert_unsafe(value.as_bytes());
    }
}

#[test]
fn missing_required_geometry_attributes_are_rejected() {
    assert_unsafe(br#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#);
    assert_unsafe(root("<path/>").as_bytes());
    assert_unsafe(root(r#"<rect width="1" height="1"/>"#).as_bytes());
}

#[test]
fn invalid_sanitizer_configuration_is_rejected() {
    let limits = SvgSanitizerLimits {
        max_depth: 0,
        ..SvgSanitizerLimits::default()
    };
    let error = sanitize_svg(FIXTURE, limits).expect_err("zero depth should fail");
    assert_eq!(error.code, RenderErrorCode::InvalidRequest);
}

fn assert_unsafe(input: &[u8]) {
    let error =
        sanitize_svg(input, SvgSanitizerLimits::default()).expect_err("SVG should be rejected");
    assert_eq!(error.code, RenderErrorCode::UnsafeOutput);
}

fn root(content: &str) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="10px" height="10px" role="img" focusable="false" viewBox="0 0 10 10" style="color: #ffffff;">{content}</svg>"##
    )
}

fn document_with_style(style: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="10px" height="10px" role="img" focusable="false" viewBox="0 0 10 10" style="{style}"><path d="M0 0"/></svg>"#
    )
}

fn document_with_path(path: &str) -> String {
    root(&format!(r#"<path d="{path}"/>"#))
}
