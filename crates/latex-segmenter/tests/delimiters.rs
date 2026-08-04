#![doc = "Delimiter and environment integration tests."]

mod common;

use common::{assert_lossless, parse, parse_with_config};
use latex_segmenter::{InlineDollarMode, SegmentKind, SegmenterConfig, Span};

#[test]
fn separates_inline_and_display_math() {
    let input = "Inline \\(x + 1\\), display \\[y^2\\], and $$z^3$$.";
    let segments = parse(input);

    assert_eq!(
        segments
            .iter()
            .map(|segment| segment.kind)
            .collect::<Vec<_>>(),
        vec![
            SegmentKind::Text,
            SegmentKind::InlineMath,
            SegmentKind::Text,
            SegmentKind::DisplayMath,
            SegmentKind::Text,
            SegmentKind::DisplayMath,
            SegmentKind::Text,
        ]
    );
    assert_eq!(segments[1].source, "\\(x + 1\\)");
    assert_eq!(segments[1].content, "x + 1");
    assert_eq!(segments[3].content, "y^2");
    assert_eq!(segments[5].content, "z^3");
    assert_lossless(input, &segments);
}

#[test]
fn recognizes_supported_display_environments() {
    let cases = [
        ("equation", "x=1"),
        ("equation*", "x=2"),
        ("align", "x&=3"),
        ("align*", "x&=4"),
        ("gather", "x=5"),
        ("gather*", "x=6"),
        ("multline", "x=7"),
    ];

    for (environment, content) in cases {
        let input = format!("\\begin{{{environment}}}{content}\\end{{{environment}}}");
        let segments = parse(&input);
        assert_eq!(segments.len(), 1, "environment {environment}");
        assert_eq!(segments[0].kind, SegmentKind::DisplayMath);
        assert_eq!(segments[0].content, content);
        assert_lossless(&input, &segments);
    }
}

#[test]
fn smart_dollars_render_math_but_not_prices() {
    let input = "Price $19.99, range $5–$10, and math $x^2$.";
    let segments = parse(input);

    assert_eq!(
        segments
            .iter()
            .map(|segment| segment.kind)
            .collect::<Vec<_>>(),
        vec![
            SegmentKind::Text,
            SegmentKind::InlineMath,
            SegmentKind::Text
        ]
    );
    assert_eq!(segments[1].content, "x^2");
    assert_lossless(input, &segments);
}

#[test]
fn single_dollars_can_be_disabled_or_forced() {
    let off = parse_with_config(
        "$x$",
        SegmenterConfig {
            inline_dollars: InlineDollarMode::Off,
        },
    );
    assert_eq!(off.len(), 1);
    assert_eq!(off[0].kind, SegmentKind::Text);

    let always = parse_with_config(
        "$5$",
        SegmenterConfig {
            inline_dollars: InlineDollarMode::Always,
        },
    );
    assert_eq!(always.len(), 1);
    assert_eq!(always[0].kind, SegmentKind::InlineMath);
    assert_eq!(always[0].content, "5");
}

#[test]
fn escaped_delimiters_remain_text() {
    let input = "Escaped \\\\(x\\) and \\$x$ but real \\(y\\).";
    let segments = parse(input);

    assert_eq!(
        segments
            .iter()
            .map(|segment| segment.kind)
            .collect::<Vec<_>>(),
        vec![
            SegmentKind::Text,
            SegmentKind::InlineMath,
            SegmentKind::Text
        ]
    );
    assert_eq!(segments[1].content, "y");
    assert_lossless(input, &segments);
}

#[test]
fn spans_are_utf8_byte_offsets() {
    let input = "π \\(α + β\\)";
    let segments = parse(input);

    assert_eq!(segments[0].span, Span { start: 0, end: 3 });
    assert_eq!(
        segments[1].span,
        Span {
            start: 3,
            end: input.len()
        }
    );
    assert_lossless(input, &segments);
}
