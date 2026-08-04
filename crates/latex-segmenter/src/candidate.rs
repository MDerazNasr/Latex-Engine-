use crate::segment::SegmentKind;

#[derive(Debug, Default)]
pub(crate) enum Mode {
    #[default]
    Text,
    Math(MathCandidate),
    Code(CodeCandidate),
}

#[derive(Clone, Debug)]
pub(crate) enum MathCloser {
    Literal(String),
    SingleDollar,
}

#[derive(Debug)]
pub(crate) struct MathCandidate {
    pub(crate) kind: SegmentKind,
    pub(crate) source: String,
    pub(crate) content: String,
    pub(crate) closer: MathCloser,
    pub(crate) start: usize,
    pub(crate) trailing_backslashes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CodeFlavor {
    Inline,
    Fence,
}

#[derive(Debug)]
pub(crate) struct CodeCandidate {
    pub(crate) source: String,
    pub(crate) content: String,
    pub(crate) marker: u8,
    pub(crate) marker_len: usize,
    pub(crate) flavor: CodeFlavor,
    pub(crate) start: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CloseMatch {
    Full(usize),
    Incomplete,
    None,
}

pub(crate) fn math_close_len(
    candidate: &MathCandidate,
    input: &str,
    finishing: bool,
) -> CloseMatch {
    if candidate.trailing_backslashes % 2 != 0 {
        return CloseMatch::None;
    }

    match &candidate.closer {
        MathCloser::SingleDollar => {
            if input.starts_with('$')
                && !candidate.content.is_empty()
                && !candidate.content.ends_with(char::is_whitespace)
            {
                CloseMatch::Full(1)
            } else {
                CloseMatch::None
            }
        }
        MathCloser::Literal(closer) => {
            if input.starts_with(closer) {
                CloseMatch::Full(closer.len())
            } else if !finishing && closer.starts_with(input) {
                CloseMatch::Incomplete
            } else {
                CloseMatch::None
            }
        }
    }
}

pub(crate) fn append_math_content(candidate: &mut MathCandidate, raw: &str) {
    candidate.source.push_str(raw);
    candidate.content.push_str(raw);
    for character in raw.chars() {
        if character == '\\' {
            candidate.trailing_backslashes += 1;
        } else {
            candidate.trailing_backslashes = 0;
        }
    }
}
