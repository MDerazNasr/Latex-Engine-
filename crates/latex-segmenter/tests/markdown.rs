#![doc = "Markdown code exclusion integration tests."]

mod common;

use common::{assert_lossless, parse};
use latex_segmenter::SegmentKind;

#[test]
fn inline_code_suppresses_math_detection() {
    let input = "Use `$x$` before \\(y\\).";
    let segments = parse(input);

    assert_eq!(
        segments
            .iter()
            .map(|segment| segment.kind)
            .collect::<Vec<_>>(),
        vec![
            SegmentKind::Text,
            SegmentKind::Code,
            SegmentKind::Text,
            SegmentKind::InlineMath,
            SegmentKind::Text,
        ]
    );
    assert_eq!(segments[1].content, "$x$");
    assert_eq!(segments[3].content, "y");
    assert_lossless(input, &segments);
}

#[test]
fn backtick_fence_suppresses_math_detection() {
    let input = "Before\n```tex\n\\[x^2\\]\n```\nAfter \\(y\\).";
    let segments = parse(input);

    assert_eq!(
        segments
            .iter()
            .map(|segment| segment.kind)
            .collect::<Vec<_>>(),
        vec![
            SegmentKind::Text,
            SegmentKind::Code,
            SegmentKind::Text,
            SegmentKind::InlineMath,
            SegmentKind::Text,
        ]
    );
    assert!(segments[1].content.contains("\\[x^2\\]"));
    assert_lossless(input, &segments);
}

#[test]
fn tilde_fence_with_indentation_suppresses_math_detection() {
    let input = "   ~~~\n$$x$$\n   ~~~\n";
    let segments = parse(input);

    assert_eq!(
        segments
            .iter()
            .map(|segment| segment.kind)
            .collect::<Vec<_>>(),
        vec![SegmentKind::Text, SegmentKind::Code, SegmentKind::Text]
    );
    assert_lossless(input, &segments);
}

#[test]
fn fence_marker_inside_a_code_line_does_not_close_the_fence() {
    let input = "```\nlet marker = ```;\n\\(still_code\\)\n```";
    let segments = parse(input);

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].kind, SegmentKind::Code);
    assert!(segments[0].content.contains("\\(still_code\\)"));
    assert_lossless(input, &segments);
}

#[test]
fn incomplete_code_delimiter_falls_back_to_text() {
    for input in ["Before `unfinished", "```\nunfinished"] {
        let segments = parse(input);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].kind, SegmentKind::Text);
        assert_lossless(input, &segments);
    }
}
