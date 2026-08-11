use std::env;

const ITERM2_KITTY_MINIMUM: (u64, u64, u64) = (3, 6, 0);

/// Image transport selected for the current terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalBackend {
    /// Transmit PNG bytes through the Kitty graphics protocol.
    KittyDirect,
    /// Ask a local Kitty compatible terminal to read a PNG file.
    KittyLocalFile,
    /// Preserve source text without writing image control sequences.
    Text,
}

/// Reason automatic detection selected the text fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackReason {
    /// Standard output is not connected to a terminal.
    RedirectedOutput,
    /// The active multiplexer cannot safely preserve image placement.
    Multiplexer,
    /// The detected iTerm2 version predates Kitty graphics support.
    Iterm2TooOld,
    /// No supported image protocol was detected.
    UnsupportedTerminal,
}

/// Process facts used by deterministic capability detection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TerminalEnvironment {
    /// Whether standard output is connected to a terminal.
    pub stdout_is_terminal: bool,
    /// Value of `TERM` when present.
    pub term: Option<String>,
    /// Value of `TERM_PROGRAM` when present.
    pub term_program: Option<String>,
    /// Value of `TERM_PROGRAM_VERSION` when present.
    pub term_program_version: Option<String>,
    /// Whether Kitty exposes a window identifier.
    pub kitty_window: bool,
    /// Whether WezTerm exposes its executable or version marker.
    pub wezterm: bool,
    /// Whether tmux is active.
    pub tmux: bool,
    /// Whether Zellij is active.
    pub zellij: bool,
}

impl TerminalEnvironment {
    /// Captures terminal variables while accepting an explicit terminal status.
    pub fn from_current_process(stdout_is_terminal: bool) -> Self {
        Self {
            stdout_is_terminal,
            term: value("TERM"),
            term_program: value("TERM_PROGRAM"),
            term_program_version: value("TERM_PROGRAM_VERSION"),
            kitty_window: env::var_os("KITTY_WINDOW_ID").is_some(),
            wezterm: env::var_os("WEZTERM_EXECUTABLE").is_some()
                || env::var_os("WEZTERM_VERSION").is_some(),
            tmux: env::var_os("TMUX").is_some() || env::var_os("TMUX_PANE").is_some(),
            zellij: env::var_os("ZELLIJ").is_some()
                || env::var_os("ZELLIJ_SESSION_NAME").is_some()
                || env::var_os("ZELLIJ_VERSION").is_some(),
        }
    }
}

/// Result of deterministic terminal capability detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSupport {
    /// Selected image or text backend.
    pub backend: TerminalBackend,
    /// Explanation when the selected backend is text.
    pub fallback_reason: Option<FallbackReason>,
}

/// Selects a safe backend without sending an active terminal probe.
pub fn detect_terminal_support(environment: &TerminalEnvironment) -> TerminalSupport {
    if !environment.stdout_is_terminal {
        return fallback(FallbackReason::RedirectedOutput);
    }
    if environment.tmux || environment.zellij {
        return fallback(FallbackReason::Multiplexer);
    }
    if environment.kitty_window
        || environment.wezterm
        || field_contains(environment.term.as_deref(), "kitty")
        || field_contains(environment.term.as_deref(), "wezterm")
        || field_contains(environment.term.as_deref(), "ghostty")
        || field_contains(environment.term_program.as_deref(), "kitty")
        || field_contains(environment.term_program.as_deref(), "wezterm")
        || field_contains(environment.term_program.as_deref(), "ghostty")
    {
        return supported(TerminalBackend::KittyDirect);
    }
    if field_contains(environment.term_program.as_deref(), "iterm") {
        return if version_at_least(
            environment.term_program_version.as_deref(),
            ITERM2_KITTY_MINIMUM,
        ) {
            supported(TerminalBackend::KittyLocalFile)
        } else {
            fallback(FallbackReason::Iterm2TooOld)
        };
    }
    fallback(FallbackReason::UnsupportedTerminal)
}

fn value(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}

fn field_contains(value: Option<&str>, needle: &str) -> bool {
    value.is_some_and(|value| value.to_ascii_lowercase().contains(needle))
}

fn version_at_least(version: Option<&str>, minimum: (u64, u64, u64)) -> bool {
    parse_version(version).is_some_and(|version| version >= minimum)
}

fn parse_version(version: Option<&str>) -> Option<(u64, u64, u64)> {
    let mut parts = version?.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    (parts.next().is_none()).then_some((major, minor, patch))
}

fn supported(backend: TerminalBackend) -> TerminalSupport {
    TerminalSupport {
        backend,
        fallback_reason: None,
    }
}

fn fallback(reason: FallbackReason) -> TerminalSupport {
    TerminalSupport {
        backend: TerminalBackend::Text,
        fallback_reason: Some(reason),
    }
}
