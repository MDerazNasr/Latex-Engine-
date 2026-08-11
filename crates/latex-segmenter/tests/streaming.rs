#![doc = "Streaming invariance and recovery integration tests."]

mod common;

use common::{assert_lossless, parse, parse_chunks};
use latex_segmenter::{SegmentKind, Segmenter};

#[test]
fn every_single_split_matches_whole_input() {
    let corpus = [
        "Text \\(x + 1\\) tail",
        "Display \\[α + β\\] tail",
        "Price $19.99 and $x$",
        "```tex\n$$x$$\n``` then \\(y\\)",
        "\\begin{align}x&=1\\end{align}",
        "Escaped \\\\(x\\) and \\(y\\)",
    ];

    for input in corpus {
        let expected = parse(input);
        for boundary in input
            .char_indices()
            .map(|(index, _)| index)
            .chain(std::iter::once(input.len()))
        {
            let actual = parse_chunks(input, &[boundary]);
            assert_eq!(actual, expected, "input {input:?}, boundary {boundary}");
            assert_lossless(input, &actual);
        }
    }
}

#[test]
fn delimiter_can_arrive_one_character_at_a_time() {
    let input = "The result is \\(x^2 + 1\\).";
    let boundaries: Vec<usize> = input
        .char_indices()
        .skip(1)
        .map(|(index, _)| index)
        .collect();

    let actual = parse_chunks(input, &boundaries);
    assert_eq!(actual, parse(input));
    assert_lossless(input, &actual);
}

#[test]
fn stable_prose_is_emitted_while_math_is_pending() {
    let mut segmenter = Segmenter::new();

    let initial = segmenter.push("Before \\(x");
    assert_eq!(initial.len(), 1);
    assert_eq!(initial[0].source, "Before ");
    assert_eq!(segmenter.pending_kind(), Some(SegmentKind::InlineMath));
    assert_eq!(segmenter.pending_source(), Some("\\(x".into()));

    let completed = segmenter.push(" + 1\\) after");
    assert_eq!(completed[0].kind, SegmentKind::InlineMath);
    assert_eq!(completed[0].content, "x + 1");
    assert_eq!(completed[1].source, " after");
    assert_eq!(segmenter.pending_kind(), None);
}

#[test]
fn pending_snapshot_includes_a_split_closing_delimiter() {
    let mut segmenter = Segmenter::new();

    segmenter.push("\\(x\\");

    assert_eq!(segmenter.pending_source(), Some("\\(x\\".into()));
    assert_eq!(segmenter.pending_kind(), Some(SegmentKind::InlineMath));
}

#[test]
fn empty_chunks_and_empty_finalization_are_no_ops() {
    let mut segmenter = Segmenter::new();

    assert!(segmenter.push("").is_empty());
    assert!(segmenter.finish().is_empty());
    assert!(segmenter.finish().is_empty());
}

#[test]
fn incomplete_math_falls_back_to_lossless_text() {
    let input = "Before \\(x + 1";
    let segments = parse(input);

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].kind, SegmentKind::Text);
    assert_lossless(input, &segments);
}

#[test]
fn split_environment_name_is_recognized() {
    let input = "\\begin{equation}x=1\\end{equation}";
    let boundaries = [1, 7, 14, 22, 31];
    let segments = parse_chunks(input, &boundaries);

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].kind, SegmentKind::DisplayMath);
    assert_eq!(segments[0].content, "x=1");
    assert_lossless(input, &segments);
}
