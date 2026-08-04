use std::mem;

use crate::candidate::{
    CloseMatch, CodeCandidate, CodeFlavor, MathCandidate, MathCloser, Mode, append_math_content,
    math_close_len,
};
use crate::config::{InlineDollarMode, SegmenterConfig};
use crate::segment::{Segment, SegmentKind, Span};
use crate::syntax::{EnvironmentMatch, ascii_run, first_char, first_char_len, match_environment};

/// Incrementally separates Markdown text, code, and supported LaTeX math.
#[derive(Debug)]
pub struct Segmenter {
    config: SegmenterConfig,
    mode: Mode,
    pending: String,
    text: String,
    text_start: Option<usize>,
    offset: usize,
    line_prefix_spaces: Option<usize>,
    text_trailing_backslashes: usize,
}

impl Default for Segmenter {
    fn default() -> Self {
        Self::new()
    }
}

impl Segmenter {
    /// Creates a segmenter with conservative single-dollar recognition.
    pub fn new() -> Self {
        Self::with_config(SegmenterConfig::default())
    }

    /// Creates a segmenter with explicit parsing behavior.
    pub fn with_config(config: SegmenterConfig) -> Self {
        Self {
            config,
            mode: Mode::Text,
            pending: String::new(),
            text: String::new(),
            text_start: None,
            offset: 0,
            line_prefix_spaces: Some(0),
            text_trailing_backslashes: 0,
        }
    }

    /// Consumes one valid UTF-8 stream chunk and returns stable completed segments.
    pub fn push(&mut self, chunk: &str) -> Vec<Segment> {
        self.pending.push_str(chunk);
        self.process(false)
    }

    /// Finalizes the stream and converts incomplete candidates back to text.
    pub fn finish(&mut self) -> Vec<Segment> {
        let mut output = self.process(true);
        self.consume_unexpected_pending();

        match mem::take(&mut self.mode) {
            Mode::Text => self.flush_text(&mut output),
            Mode::Math(candidate) => {
                push_segment(
                    &mut output,
                    Segment::text(candidate.source, Span::new(candidate.start, self.offset)),
                );
            }
            Mode::Code(candidate) => {
                push_segment(
                    &mut output,
                    Segment::text(candidate.source, Span::new(candidate.start, self.offset)),
                );
            }
        }

        output
    }

    /// Returns an exact source snapshot of a currently open math or code candidate.
    pub fn pending_source(&self) -> Option<String> {
        match &self.mode {
            Mode::Text => None,
            Mode::Math(candidate) => Some(format!("{}{}", candidate.source, self.pending)),
            Mode::Code(candidate) => Some(format!("{}{}", candidate.source, self.pending)),
        }
    }

    /// Returns the semantic kind of the currently open candidate.
    pub fn pending_kind(&self) -> Option<SegmentKind> {
        match &self.mode {
            Mode::Text => None,
            Mode::Math(candidate) => Some(candidate.kind),
            Mode::Code(_) => Some(SegmentKind::Code),
        }
    }

    fn process(&mut self, finishing: bool) -> Vec<Segment> {
        let input = mem::take(&mut self.pending);
        let mut output = Vec::new();
        let mut cursor = 0;

        while cursor < input.len() {
            let remaining = &input[cursor..];
            let mode = mem::take(&mut self.mode);

            match mode {
                Mode::Text => {
                    if !self.process_text(remaining, finishing, &mut cursor, &mut output) {
                        self.mode = Mode::Text;
                        break;
                    }
                }
                Mode::Math(candidate) => {
                    if !self.process_math(candidate, remaining, finishing, &mut cursor, &mut output)
                    {
                        break;
                    }
                }
                Mode::Code(candidate) => {
                    if !self.process_code(candidate, remaining, finishing, &mut cursor, &mut output)
                    {
                        break;
                    }
                }
            }
        }

        self.pending.push_str(&input[cursor..]);
        if matches!(self.mode, Mode::Text) {
            self.flush_text(&mut output);
        }
        output
    }

    fn process_text(
        &mut self,
        input: &str,
        finishing: bool,
        cursor: &mut usize,
        output: &mut Vec<Segment>,
    ) -> bool {
        if input.starts_with('\\') && self.text_trailing_backslashes % 2 == 0 {
            match match_environment(input, finishing) {
                EnvironmentMatch::Full { opener, closer } => {
                    self.open_math(opener, MathCloser::Literal(closer.into()), true, output);
                    *cursor += opener.len();
                    return true;
                }
                EnvironmentMatch::Incomplete => return false,
                EnvironmentMatch::None => {}
            }

            for (opener, closer, display) in [("\\(", "\\)", false), ("\\[", "\\]", true)] {
                if input.starts_with(opener) {
                    self.open_math(opener, MathCloser::Literal(closer.into()), display, output);
                    *cursor += opener.len();
                    return true;
                }
            }

            if !finishing && input.len() == 1 {
                return false;
            }
        }

        if input.starts_with('$') && self.text_trailing_backslashes % 2 == 0 {
            if input.starts_with("$$") {
                self.open_math("$$", MathCloser::Literal("$$".into()), true, output);
                *cursor += 2;
                return true;
            }
            if !finishing && input.len() == 1 {
                return false;
            }
            if self.can_open_single_dollar(&input[1..]) {
                self.open_math("$", MathCloser::SingleDollar, false, output);
                *cursor += 1;
                return true;
            }
        }

        if input.starts_with('`') {
            let run = ascii_run(input, b'`');
            if !finishing && run == input.len() {
                return false;
            }
            let flavor = if run >= 3 && self.fence_eligible() {
                CodeFlavor::Fence
            } else {
                CodeFlavor::Inline
            };
            self.open_code('`', run, flavor, output);
            *cursor += run;
            return true;
        }

        if input.starts_with('~') && self.fence_eligible() {
            let run = ascii_run(input, b'~');
            if !finishing && run == input.len() {
                return false;
            }
            if run >= 3 {
                self.open_code('~', run, CodeFlavor::Fence, output);
                *cursor += run;
                return true;
            }
        }

        let char_len = first_char_len(input);
        self.append_text(&input[..char_len]);
        *cursor += char_len;
        self.mode = Mode::Text;
        true
    }

    fn process_math(
        &mut self,
        mut candidate: MathCandidate,
        input: &str,
        finishing: bool,
        cursor: &mut usize,
        output: &mut Vec<Segment>,
    ) -> bool {
        match math_close_len(&candidate, input, finishing) {
            CloseMatch::Full(close_len) => {
                let closer = &input[..close_len];
                candidate.source.push_str(closer);
                self.advance_position(closer);
                *cursor += close_len;
                let segment = Segment::new(
                    candidate.kind,
                    candidate.source,
                    candidate.content,
                    Span::new(candidate.start, self.offset),
                );
                push_segment(output, segment);
                self.text_trailing_backslashes = 0;
                self.mode = Mode::Text;
                true
            }
            CloseMatch::Incomplete => {
                self.mode = Mode::Math(candidate);
                false
            }
            CloseMatch::None => {
                let char_len = first_char_len(input);
                let raw = &input[..char_len];
                append_math_content(&mut candidate, raw);
                self.advance_position(raw);
                *cursor += char_len;
                self.mode = Mode::Math(candidate);
                true
            }
        }
    }

    fn process_code(
        &mut self,
        mut candidate: CodeCandidate,
        input: &str,
        finishing: bool,
        cursor: &mut usize,
        output: &mut Vec<Segment>,
    ) -> bool {
        if input.as_bytes().first().copied() == Some(candidate.marker) {
            let run = ascii_run(input, candidate.marker);
            if !finishing && run == input.len() {
                self.mode = Mode::Code(candidate);
                return false;
            }
            let closes = match candidate.flavor {
                CodeFlavor::Inline => run == candidate.marker_len,
                CodeFlavor::Fence => run >= candidate.marker_len && self.fence_eligible(),
            };
            if closes {
                let closer = &input[..run];
                candidate.source.push_str(closer);
                self.advance_position(closer);
                *cursor += run;
                push_segment(
                    output,
                    Segment::new(
                        SegmentKind::Code,
                        candidate.source,
                        candidate.content,
                        Span::new(candidate.start, self.offset),
                    ),
                );
                self.text_trailing_backslashes = 0;
                self.mode = Mode::Text;
                return true;
            }
        }

        let char_len = first_char_len(input);
        let raw = &input[..char_len];
        candidate.source.push_str(raw);
        candidate.content.push_str(raw);
        self.advance_position(raw);
        *cursor += char_len;
        self.mode = Mode::Code(candidate);
        true
    }

    fn open_math(
        &mut self,
        opener: &str,
        closer: MathCloser,
        display: bool,
        output: &mut Vec<Segment>,
    ) {
        self.flush_text(output);
        let start = self.offset;
        self.advance_position(opener);
        self.text_trailing_backslashes = 0;
        self.mode = Mode::Math(MathCandidate {
            kind: if display {
                SegmentKind::DisplayMath
            } else {
                SegmentKind::InlineMath
            },
            source: opener.into(),
            content: String::new(),
            closer,
            start,
            trailing_backslashes: 0,
        });
    }

    fn open_code(
        &mut self,
        marker: char,
        marker_len: usize,
        flavor: CodeFlavor,
        output: &mut Vec<Segment>,
    ) {
        self.flush_text(output);
        let start = self.offset;
        let opener = marker.to_string().repeat(marker_len);
        self.advance_position(&opener);
        self.text_trailing_backslashes = 0;
        self.mode = Mode::Code(CodeCandidate {
            source: opener,
            content: String::new(),
            marker: marker as u8,
            marker_len,
            flavor,
            start,
        });
    }

    fn can_open_single_dollar(&self, remainder: &str) -> bool {
        let Some(next) = first_char(remainder) else {
            return false;
        };
        if next.is_whitespace() {
            return false;
        }
        match self.config.inline_dollars {
            InlineDollarMode::Off => false,
            InlineDollarMode::Smart => !next.is_ascii_digit(),
            InlineDollarMode::Always => true,
        }
    }

    fn append_text(&mut self, raw: &str) {
        if self.text.is_empty() {
            self.text_start = Some(self.offset);
        }
        self.text.push_str(raw);
        self.advance_position(raw);
        for character in raw.chars() {
            if character == '\\' {
                self.text_trailing_backslashes += 1;
            } else {
                self.text_trailing_backslashes = 0;
            }
        }
    }

    fn flush_text(&mut self, output: &mut Vec<Segment>) {
        if self.text.is_empty() {
            return;
        }
        let source = mem::take(&mut self.text);
        let start = self.text_start.take().unwrap_or(self.offset - source.len());
        push_segment(output, Segment::text(source, Span::new(start, self.offset)));
    }

    fn advance_position(&mut self, raw: &str) {
        self.offset += raw.len();
        for character in raw.chars() {
            match character {
                '\n' => self.line_prefix_spaces = Some(0),
                ' ' => {
                    if let Some(spaces) = self.line_prefix_spaces {
                        self.line_prefix_spaces = (spaces < 3).then_some(spaces + 1);
                    }
                }
                _ => self.line_prefix_spaces = None,
            }
        }
    }

    fn fence_eligible(&self) -> bool {
        self.line_prefix_spaces.is_some()
    }

    fn consume_unexpected_pending(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let pending = mem::take(&mut self.pending);
        match &mut self.mode {
            Mode::Text => self.append_text(&pending),
            Mode::Math(candidate) => {
                candidate.source.push_str(&pending);
                candidate.content.push_str(&pending);
                self.advance_position(&pending);
            }
            Mode::Code(candidate) => {
                candidate.source.push_str(&pending);
                candidate.content.push_str(&pending);
                self.advance_position(&pending);
            }
        }
    }
}

fn push_segment(output: &mut Vec<Segment>, segment: Segment) {
    if let Some(previous) = output.last_mut()
        && previous.kind == SegmentKind::Text
        && segment.kind == SegmentKind::Text
        && previous.span.end == segment.span.start
    {
        previous.source.push_str(&segment.source);
        previous.content.push_str(&segment.content);
        previous.span.end = segment.span.end;
        return;
    }
    output.push(segment);
}
