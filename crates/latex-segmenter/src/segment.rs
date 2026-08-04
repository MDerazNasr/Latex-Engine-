/// The byte range occupied by a segment in the complete input stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    /// Inclusive byte offset at which the segment begins.
    pub start: usize,
    /// Exclusive byte offset at which the segment ends.
    pub end: usize,
}

impl Span {
    pub(crate) fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// The semantic role of a parsed segment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentKind {
    /// Ordinary Markdown source that should use the normal presentation path.
    Text,
    /// Inline mathematical source such as `\(x\)` or `$x$`.
    InlineMath,
    /// Display mathematical source such as `\[x\]` or `$$x$$`.
    DisplayMath,
    /// Markdown code whose contents must never be interpreted as math.
    Code,
}

/// A lossless source segment and its renderable inner content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Segment {
    /// The semantic role of this segment.
    pub kind: SegmentKind,
    /// The exact original bytes represented as valid UTF-8.
    pub source: String,
    /// Inner math or code content without delimiters, or the text itself.
    pub content: String,
    /// The segment byte range in the complete input stream.
    pub span: Span,
}

impl Segment {
    pub(crate) fn new(kind: SegmentKind, source: String, content: String, span: Span) -> Self {
        Self {
            kind,
            source,
            content,
            span,
        }
    }

    pub(crate) fn text(source: String, span: Span) -> Self {
        Self::new(SegmentKind::Text, source.clone(), source, span)
    }
}
