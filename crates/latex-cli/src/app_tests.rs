use latex_render_client::{WorkerHealth, WorkerState};
use latex_render_core::{CacheStats, RenderErrorCode, RenderLimits};

use crate::app::check_report;

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
