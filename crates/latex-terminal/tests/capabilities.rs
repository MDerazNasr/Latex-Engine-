//! Terminal capability selection tests.

use latex_terminal::FallbackReason;
use latex_terminal::TerminalBackend;
use latex_terminal::TerminalEnvironment;
use latex_terminal::TerminalSupport;
use latex_terminal::detect_terminal_support;

#[test]
fn redirected_and_unknown_terminals_use_text() {
    assert_eq!(
        detect_terminal_support(&TerminalEnvironment::default()),
        fallback(FallbackReason::RedirectedOutput),
    );
    assert_eq!(
        detect_terminal_support(&TerminalEnvironment {
            stdout_is_terminal: true,
            ..TerminalEnvironment::default()
        }),
        fallback(FallbackReason::UnsupportedTerminal),
    );
}

#[test]
fn kitty_and_wezterm_select_direct_transfer() {
    for environment in [
        TerminalEnvironment {
            stdout_is_terminal: true,
            kitty_window: true,
            ..TerminalEnvironment::default()
        },
        TerminalEnvironment {
            stdout_is_terminal: true,
            wezterm: true,
            ..TerminalEnvironment::default()
        },
        TerminalEnvironment {
            stdout_is_terminal: true,
            term_program: Some("ghostty".to_string()),
            ..TerminalEnvironment::default()
        },
    ] {
        assert_eq!(
            detect_terminal_support(&environment),
            supported(TerminalBackend::KittyDirect),
        );
    }
}

#[test]
fn current_iterm2_selects_local_file_transfer() {
    assert_eq!(
        detect_terminal_support(&TerminalEnvironment {
            stdout_is_terminal: true,
            term_program: Some("iTerm.app".to_string()),
            term_program_version: Some("3.6.10".to_string()),
            ..TerminalEnvironment::default()
        }),
        supported(TerminalBackend::KittyLocalFile),
    );
}

#[test]
fn old_or_malformed_iterm2_versions_use_text() {
    for version in [None, Some("3.5.9"), Some("3.6.beta"), Some("3.6.0.1")] {
        assert_eq!(
            detect_terminal_support(&TerminalEnvironment {
                stdout_is_terminal: true,
                term_program: Some("iTerm2".to_string()),
                term_program_version: version.map(str::to_string),
                ..TerminalEnvironment::default()
            }),
            fallback(FallbackReason::Iterm2TooOld),
        );
    }
}

#[test]
fn multiplexers_disable_an_underlying_image_terminal() {
    for environment in [
        TerminalEnvironment {
            stdout_is_terminal: true,
            kitty_window: true,
            tmux: true,
            ..TerminalEnvironment::default()
        },
        TerminalEnvironment {
            stdout_is_terminal: true,
            kitty_window: true,
            zellij: true,
            ..TerminalEnvironment::default()
        },
        TerminalEnvironment {
            stdout_is_terminal: true,
            kitty_window: true,
            screen: true,
            ..TerminalEnvironment::default()
        },
    ] {
        assert_eq!(
            detect_terminal_support(&environment),
            fallback(FallbackReason::Multiplexer),
        );
    }
}

#[test]
fn ssh_disables_only_the_local_file_backend() {
    assert_eq!(
        detect_terminal_support(&TerminalEnvironment {
            stdout_is_terminal: true,
            term_program: Some("iTerm2".to_string()),
            term_program_version: Some("3.6.10".to_string()),
            ssh: true,
            ..TerminalEnvironment::default()
        }),
        fallback(FallbackReason::RemoteFileUnavailable),
    );
    assert_eq!(
        detect_terminal_support(&TerminalEnvironment {
            stdout_is_terminal: true,
            kitty_window: true,
            ssh: true,
            ..TerminalEnvironment::default()
        }),
        supported(TerminalBackend::KittyDirect),
    );
}

#[test]
fn diagnostic_names_are_stable_lowercase_values() {
    assert_eq!(
        TerminalBackend::KittyDirect.diagnostic_name(),
        "kitty_direct"
    );
    assert_eq!(
        TerminalBackend::KittyLocalFile.diagnostic_name(),
        "kitty_local_file"
    );
    assert_eq!(TerminalBackend::Text.diagnostic_name(), "text");
    assert_eq!(
        FallbackReason::RemoteFileUnavailable.diagnostic_name(),
        "remote_file_unavailable"
    );
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
