#![doc = "Process tests for the end to end terminal smoke tool."]

use std::path::PathBuf;
use std::process::Command;

#[test]
fn automatic_redirected_output_preserves_source_without_starting_worker() {
    let output = command()
        .args(["--worker", "/path/that/does/not/exist/worker.js", "x+y"])
        .output()
        .expect("smoke tool should run");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"x+y\n");
    assert!(output.stderr.is_empty());
    assert!(!output.stdout.contains(&0x1b));
}

#[test]
fn forced_kitty_runs_real_pipeline_resize_and_cleanup() {
    let output = command()
        .args([
            "--backend",
            "kitty",
            "--geometry",
            "80x24@800x480",
            "--resize-geometry",
            "60x24@600x480",
            "--worker",
        ])
        .arg(fake_worker())
        .args(["--hold-ms", "0", "x+y"])
        .output()
        .expect("smoke tool should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let output = String::from_utf8(output.stdout).expect("protocol should be UTF 8");
    assert!(output.starts_with("\x1b[?1049h\x1b[2J\x1b[?25l"));
    assert_eq!(output.matches("a=T,t=d,f=100").count(), 2);
    assert!(output.matches("a=d,d=I,i=49378").count() >= 2);
    assert!(output.ends_with("\x1b[?25h\x1b[?1049l"));
    assert!(!output.contains("x+y"));
}

#[test]
fn forced_iterm2_uses_correlated_local_file_and_restores_screen() {
    let output = command()
        .args([
            "--backend",
            "iterm2",
            "--geometry",
            "80x24@800x480",
            "--worker",
        ])
        .arg(fake_worker())
        .args(["--hold-ms", "0", "x+y"])
        .output()
        .expect("smoke tool should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let output = String::from_utf8(output.stdout).expect("protocol should be UTF 8");
    assert!(output.contains("a=T,t=f,f=100"));
    assert!(output.contains("c="));
    assert!(output.contains("r="));
    assert!(output.ends_with("\x1b[?25h\x1b[?1049l"));
}

#[test]
fn worker_failure_keeps_source_and_never_opens_the_screen() {
    let output = command()
        .args([
            "--backend",
            "kitty",
            "--geometry",
            "80x24@800x480",
            "--worker",
            "/path/that/does/not/exist/worker.js",
            "x+y",
        ])
        .output()
        .expect("smoke tool should run");

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"x+y\n");
    assert!(!output.stdout.contains(&0x1b));
    assert!(!output.stderr.is_empty());
}

#[test]
fn malformed_arguments_emit_no_terminal_bytes() {
    let output = command()
        .args(["--backend", "kitty", "--geometry", "invalid"])
        .output()
        .expect("smoke tool should run");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
    assert!(!output.stderr.contains(&0x1b));
}

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_latex-terminal-smoke"))
}

fn fake_worker() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("tool should be inside the repository")
        .join("crates/latex-render-client/tests/support/fake-worker.mjs")
}
