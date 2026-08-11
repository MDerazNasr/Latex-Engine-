#![doc = "Streaming Markdown-aware LaTeX math segmentation for Codex clients."]

mod candidate;
mod config;
mod parser;
mod segment;
mod syntax;

pub use config::{InlineDollarMode, SegmenterConfig};
pub use parser::Segmenter;
pub use segment::{Segment, SegmentKind, Span};

/// The segmenter behavior version used by integration adapters and fixtures.
pub const SEGMENTER_VERSION: u32 = 1;
