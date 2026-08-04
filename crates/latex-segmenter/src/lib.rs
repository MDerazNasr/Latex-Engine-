#![doc = "Streaming Markdown-aware LaTeX math segmentation for Codex clients."]

/// The segmenter crate protocol version.
pub const SEGMENTER_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use super::SEGMENTER_VERSION;

    #[test]
    fn protocol_version_starts_at_one() {
        assert_eq!(SEGMENTER_VERSION, 1);
    }
}
