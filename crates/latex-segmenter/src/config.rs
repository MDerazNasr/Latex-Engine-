/// Controls whether single dollar delimiters are interpreted as inline math.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum InlineDollarMode {
    /// Treat every single dollar as ordinary text.
    Off,
    /// Apply conservative currency and whitespace checks.
    #[default]
    Smart,
    /// Treat a single dollar followed by non-whitespace as an opener.
    Always,
}

/// Configuration for a streaming [`crate::Segmenter`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SegmenterConfig {
    /// Controls recognition of `$...$` inline math.
    pub inline_dollars: InlineDollarMode,
}
