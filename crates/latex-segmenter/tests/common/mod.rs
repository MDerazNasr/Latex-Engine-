#![allow(dead_code)]

use latex_segmenter::{Segment, SegmentKind, Segmenter, SegmenterConfig};

pub fn parse(input: &str) -> Vec<Segment> {
    parse_with_config(input, SegmenterConfig::default())
}

pub fn parse_with_config(input: &str, config: SegmenterConfig) -> Vec<Segment> {
    let mut segmenter = Segmenter::with_config(config);
    let mut output = segmenter.push(input);
    output.extend(segmenter.finish());
    normalize(output)
}

pub fn parse_chunks(input: &str, boundaries: &[usize]) -> Vec<Segment> {
    let mut segmenter = Segmenter::new();
    let mut output = Vec::new();
    let mut start = 0;
    for &end in boundaries {
        output.extend(segmenter.push(&input[start..end]));
        start = end;
    }
    output.extend(segmenter.push(&input[start..]));
    output.extend(segmenter.finish());
    normalize(output)
}

pub fn normalize(segments: Vec<Segment>) -> Vec<Segment> {
    let mut normalized: Vec<Segment> = Vec::new();
    for segment in segments {
        if let Some(previous) = normalized.last_mut()
            && previous.kind == SegmentKind::Text
            && segment.kind == SegmentKind::Text
            && previous.span.end == segment.span.start
        {
            previous.source.push_str(&segment.source);
            previous.content.push_str(&segment.content);
            previous.span.end = segment.span.end;
        } else {
            normalized.push(segment);
        }
    }
    normalized
}

pub fn assert_lossless(input: &str, segments: &[Segment]) {
    let reconstructed: String = segments
        .iter()
        .map(|segment| segment.source.as_str())
        .collect();
    assert_eq!(reconstructed, input);

    let mut expected_start = 0;
    for segment in segments {
        assert_eq!(segment.span.start, expected_start);
        assert_eq!(segment.span.end - segment.span.start, segment.source.len());
        expected_start = segment.span.end;
    }
    assert_eq!(expected_start, input.len());
}
