//! Process tests for the terminal lifecycle spike.

use std::process::Command;

const BINARY: &str = env!("CARGO_BIN_EXE_terminal-spike");

#[test]
fn automatic_redirected_output_preserves_source_without_control_sequences() {
    let output = Command::new(BINARY)
        .args(["--png", "/path/that/does/not/exist"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "\\[x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}\\]\n",
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn forced_kitty_run_draws_resizes_deletes_and_restores() {
    let output = Command::new(BINARY)
        .args(["--backend", "kitty", "--hold-ms", "0"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.starts_with("\x1b[?1049h\x1b[2J\x1b[?25l"));
    assert_eq!(stdout.matches("\x1b_Ga=T,t=d").count(), 2);
    assert_eq!(stdout.matches("\x1b_Ga=d,d=I").count(), 2);
    assert!(stdout.ends_with("\x1b[?25h\x1b[?1049l"));
    assert!(output.stderr.is_empty());
}

#[test]
fn forced_iterm2_run_uses_local_file_transport_and_cleans_up() {
    let output = Command::new(BINARY)
        .args(["--backend", "iterm2", "--hold-ms", "0", "--no-resize"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert_eq!(stdout.matches("\x1b_Ga=T,t=f").count(), 1);
    assert_eq!(stdout.matches("\x1b_Ga=d,d=I").count(), 1);
    assert!(stdout.ends_with("\x1b[?25h\x1b[?1049l"));
    assert!(output.stderr.is_empty());
}

#[test]
fn malformed_arguments_fail_without_terminal_output() {
    let output = Command::new(BINARY)
        .args(["--columns", "0"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("dimensions must be nonzero")
    );
}
