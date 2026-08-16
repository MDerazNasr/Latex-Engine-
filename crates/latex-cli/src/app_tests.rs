use latex_render_client::{WorkerHealth, WorkerState};
use latex_render_core::{CacheStats, RenderErrorCode, RenderLimits};
use latex_terminal::TerminalEnvironment;

use crate::app::{check_report, doctor_report};

#[test]
fn check_report_preserves_recovered_health_without_source() {
    let health = WorkerHealth {
        state: WorkerState::Ready,
        renderer_version: Some("0.1.0".to_owned()),
        restart_count: 1,
        last_error: Some(RenderErrorCode::UnsafeOutput),
    };

    let report = check_report(
        "0.1.0",
        &health,
        CacheStats::default(),
        RenderLimits::default(),
    )
    .expect("report should be constructed");

    assert!(report.contains("worker_state=ready\n"));
    assert!(report.contains("restart_count=1\n"));
    assert!(report.contains("last_error=unsafe_output\n"));
    assert!(!report.contains("x^2"));
}

#[test]
fn doctor_report_appends_stable_source_fallback_facts() {
    let environment = TerminalEnvironment {
        stdout_is_terminal: true,
        term_program: Some("iTerm2".to_owned()),
        term_program_version: Some("3.6.10".to_owned()),
        ssh: true,
        ..TerminalEnvironment::default()
    };
    let report = doctor_report("status=ok\n".to_owned(), &environment)
        .expect("doctor report should be constructed");

    assert!(report.starts_with("status=ok\n"));
    assert!(report.contains("terminal_stdout_tty=true\n"));
    assert!(report.contains("terminal_backend=text\n"));
    assert!(report.contains("terminal_fallback=remote_file_unavailable\n"));
    assert!(report.contains("terminal_ssh=true\n"));
    assert!(report.contains("terminal_tmux=false\n"));
    assert!(report.contains("terminal_zellij=false\n"));
    assert!(report.contains("terminal_screen=false\n"));
}
