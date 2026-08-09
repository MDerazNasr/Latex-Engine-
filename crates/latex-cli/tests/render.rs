#![doc = "Process integration tests for the standalone render command."]

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FILE: AtomicU64 = AtomicU64::new(1);

#[test]
fn svg_renders_to_clean_stdout() {
    let output = command()
        .args(worker_arguments())
        .args(["x^2"])
        .output()
        .expect("CLI should run");

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(output.stderr.is_empty());
    assert!(output.stdout.starts_with(b"<svg "));
    assert!(output.stdout.ends_with(b"</svg>"));
    assert!(!contains(&output.stdout, b"x^2"));
}

#[test]
fn redirected_stdin_is_preserved_and_rendered() {
    let mut child = command()
        .args(worker_arguments())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("CLI should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(b"x+y\n")
        .expect("stdin should accept source");
    let output = child.wait_with_output().expect("CLI should exit");

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(output.stdout.starts_with(b"<svg "));
    assert!(output.stderr.is_empty());
}

#[test]
fn png_renders_to_stdout_through_the_native_boundary() {
    let output = command()
        .args(worker_arguments())
        .args(["--format", "png", "x^2"])
        .output()
        .expect("CLI should run");

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(output.stderr.is_empty());
    assert!(output.stdout.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(output.stdout.len() > 8);
}

#[test]
fn new_output_is_published_only_after_rendering_completes() {
    let path = temporary_file("svg");
    assert!(!path.exists(), "temporary target should be unused");

    let output = command()
        .args(worker_arguments())
        .args(["--output"])
        .arg(&path)
        .arg("x^2")
        .output()
        .expect("CLI should run");

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert!(
        fs::read(&path)
            .expect("SVG should be readable")
            .starts_with(b"<svg ")
    );
    assert!(!temporary_sibling_exists(&path));

    fs::remove_file(path).expect("temporary output should be removed");
}

#[test]
fn existing_output_is_preserved_until_force_is_explicit() {
    let path = temporary_file("png");
    fs::write(&path, b"original").expect("fixture should be created");

    let refused = command()
        .args(worker_arguments())
        .args(["--format", "png", "--output"])
        .arg(&path)
        .arg("x^2")
        .output()
        .expect("CLI should run");
    assert_eq!(refused.status.code(), Some(5));
    assert_eq!(fs::read(&path).expect("fixture should remain"), b"original");

    let replaced = command()
        .args(worker_arguments())
        .args(["--format", "png", "--output"])
        .arg(&path)
        .args(["--force", "x^2"])
        .output()
        .expect("CLI should run");
    assert!(replaced.status.success(), "{}", stderr(&replaced));
    assert!(
        fs::read(&path)
            .expect("PNG should be readable")
            .starts_with(b"\x89PNG\r\n\x1a\n")
    );

    fs::remove_file(path).expect("temporary output should be removed");
}

#[test]
fn environment_worker_path_is_supported() {
    let output = command()
        .env("LATEX_RENDER_WORKER", fake_worker())
        .arg("x^2")
        .output()
        .expect("CLI should run");

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(output.stdout.starts_with(b"<svg "));
}

#[test]
fn missing_worker_and_empty_source_use_stable_exit_codes() {
    let missing = command()
        .args(["--worker", "/path/that/does/not/exist/worker.js", "x^2"])
        .output()
        .expect("CLI should run");
    assert_eq!(missing.status.code(), Some(3));
    assert!(!contains(&missing.stderr, b"x^2"));

    let empty = command()
        .args(worker_arguments())
        .arg("")
        .output()
        .expect("CLI should run");
    assert_eq!(empty.status.code(), Some(2));
    assert!(!contains(&empty.stderr, b"x^2"));
}

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_latex-render"));
    command.arg("render");
    command
}

fn worker_arguments() -> Vec<std::ffi::OsString> {
    vec!["--worker".into(), fake_worker().into_os_string()]
}

fn fake_worker() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("CLI crate should be inside the repository")
        .join("crates/latex-render-client/tests/support/fake-worker.mjs")
}

fn temporary_file(extension: &str) -> PathBuf {
    let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "latex-render-cli-{}-{id}.{extension}",
        std::process::id()
    ))
}

fn temporary_sibling_exists(path: &std::path::Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return true;
    };
    let prefix = format!("{file_name}.latex-render.{}.", std::process::id());
    fs::read_dir(path.parent().expect("temporary file should have a parent"))
        .expect("temporary directory should be readable")
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .any(|name| name.starts_with(&prefix) && name.ends_with(".tmp"))
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|candidate| candidate == needle)
}
