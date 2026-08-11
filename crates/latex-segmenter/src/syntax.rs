pub(crate) const ENVIRONMENTS: &[(&str, &str)] = &[
    ("\\begin{equation}", "\\end{equation}"),
    ("\\begin{equation*}", "\\end{equation*}"),
    ("\\begin{align}", "\\end{align}"),
    ("\\begin{align*}", "\\end{align*}"),
    ("\\begin{gather}", "\\end{gather}"),
    ("\\begin{gather*}", "\\end{gather*}"),
    ("\\begin{multline}", "\\end{multline}"),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnvironmentMatch {
    Full {
        opener: &'static str,
        closer: &'static str,
    },
    Incomplete,
    None,
}

pub(crate) fn match_environment(input: &str, finishing: bool) -> EnvironmentMatch {
    for &(opener, closer) in ENVIRONMENTS {
        if input.starts_with(opener) {
            return EnvironmentMatch::Full { opener, closer };
        }
    }

    if !finishing
        && ENVIRONMENTS
            .iter()
            .any(|(opener, _)| opener.starts_with(input))
    {
        return EnvironmentMatch::Incomplete;
    }

    EnvironmentMatch::None
}

pub(crate) fn ascii_run(input: &str, marker: u8) -> usize {
    input
        .as_bytes()
        .iter()
        .take_while(|&&byte| byte == marker)
        .count()
}

pub(crate) fn first_char_len(input: &str) -> usize {
    input.chars().next().map(char::len_utf8).unwrap_or_default()
}

pub(crate) fn first_char(input: &str) -> Option<char> {
    input.chars().next()
}
