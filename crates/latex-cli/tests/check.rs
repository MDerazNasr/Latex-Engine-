#![doc = "Process integration tests for the standalone check command."]

use std::path::PathBuf;
use std::process::Command;

#[test]
fn healthy_pipeline_reports_versions_health_cache_and_limits() {
    let output = command()
        .args(["--worker"])
        .arg(fake_worker())
        .output()
        .expect("CLI should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let report = String::from_utf8(output.stdout).expect("report should be UTF 8");
    for expected in [
        "status=ok\n",
        "protocol=1\n",
        "renderer=mathjax\n",
        "renderer_version=0.1.0\n",
        "sanitizer=svg-allowlist-1\n",
        "rasterizer=resvg-0.48.1-policy-1\n",
        "worker_state=ready\n",
        "restart_count=0\n",
        "last_error=none\n",
        "cache_entries=1\n",
        "cache_hits=0\n",
        "cache_misses=1\n",
        "max_source_bytes=16384\n",
        "max_json_line_bytes=65536\n",
        "max_svg_bytes=2097152\n",
        "max_width_px=4096\n",
        "max_height_px=2048\n",
        "min_scale=0.5\n",
        "max_scale=4\n",
    ] {
        assert!(
            report.contains(expected),
            "missing {expected:?} in {report}"
        );
    }
    assert!(!report.contains("x^2"));
}

#[test]
fn missing_worker_is_a_worker_failure_without_a_report() {
    let output = command()
        .args(["--worker", "/path/that/does/not/exist/worker.js"])
        .output()
        .expect("CLI should run");

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("x^2"));
}

#[test]
fn doctor_runs_the_pipeline_and_reports_redirected_terminal_fallback() {
    let output = doctor_command()
        .args(["--worker"])
        .arg(fake_worker())
        .output()
        .expect("CLI should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let report = String::from_utf8(output.stdout).expect("report should be UTF 8");
    assert!(report.contains("status=ok\n"));
    assert!(report.contains("worker_state=ready\n"));
    assert!(report.contains("terminal_stdout_tty=false\n"));
    assert!(report.contains("terminal_backend=text\n"));
    assert!(report.contains("terminal_fallback=redirected_output\n"));
    assert!(report.contains("terminal_ssh=false\n"));
}

#[test]
fn doctor_pipeline_failure_emits_no_partial_terminal_report() {
    let output = doctor_command()
        .args(["--worker", "/path/that/does/not/exist/worker.js"])
        .output()
        .expect("CLI should run");

    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("terminal_backend"));
}

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_latex-render"));
    command.arg("check");
    command
}

fn doctor_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_latex-render"));
    command.arg("doctor");
    command
}

fn fake_worker() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("CLI crate should be inside the repository")
        .join("crates/latex-render-client/tests/support/fake-worker.mjs")
}
