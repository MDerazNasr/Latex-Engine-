#![doc = "Supervised worker process integration tests."]

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use latex_render_client::{WorkerClient, WorkerClientConfig, WorkerCommand, WorkerState};
use latex_render_core::{CacheLimits, RenderErrorCode, RenderRequest, Rgba};

static NEXT_MARKER: AtomicU64 = AtomicU64::new(1);

#[test]
fn client_start_requires_an_async_runtime() {
    let error = WorkerClient::start(config("healthy", None))
        .err()
        .expect("client should reject a missing runtime");

    assert_eq!(error.code, RenderErrorCode::WorkerUnavailable);
}

#[test]
fn client_rejects_invalid_resource_configuration() {
    let mut invalid_queue = config("healthy", None);
    invalid_queue.queue_capacity = 0;
    let error = WorkerClient::start(invalid_queue)
        .err()
        .expect("zero queue should fail");
    assert_eq!(error.code, RenderErrorCode::InvalidRequest);

    let mut invalid_limit = config("healthy", None);
    invalid_limit.render_limits.max_svg_bytes = 0;
    let error = WorkerClient::start(invalid_limit)
        .err()
        .expect("zero output limit should fail");
    assert_eq!(error.code, RenderErrorCode::InvalidRequest);
}

#[tokio::test]
async fn worker_is_lazy_cached_and_gracefully_stopped() {
    let marker = marker_path("cache");
    let mut client =
        WorkerClient::start(config("healthy-count", Some(&marker))).expect("client should start");
    assert_eq!(client.health().await.state, WorkerState::Idle);

    let first = client
        .render_request(request("x^2"))
        .await
        .expect("first render should succeed");
    let second = client
        .render_request(request("x^2"))
        .await
        .expect("cached render should succeed");

    assert_eq!(first, second);
    assert_eq!(client.health().await.state, WorkerState::Ready);
    let stats = client.cache_stats().await;
    assert_eq!(stats.entries, 1);
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert_eq!(
        fs::read_to_string(&marker).expect("counter should exist"),
        "render\n"
    );

    client.shutdown().await.expect("shutdown should succeed");
    assert_eq!(client.health().await.state, WorkerState::Stopped);
    remove_marker(&marker);
}

#[tokio::test]
async fn invalid_tex_isolated_without_restarting_worker() {
    let mut client = WorkerClient::start(config("invalid-tex", None)).expect("client should start");

    let error = client
        .render_request(request("{"))
        .await
        .expect_err("invalid math should fail");

    assert_eq!(error.code, RenderErrorCode::InvalidTex);
    assert!(!error.message.contains('{'));
    let health = client.health().await;
    assert_eq!(health.state, WorkerState::Ready);
    assert_eq!(health.restart_count, 0);
    client.shutdown().await.expect("shutdown should succeed");
}

#[tokio::test]
async fn crash_is_restarted_once_and_request_recovers() {
    let marker = marker_path("restart");
    let mut client =
        WorkerClient::start(config("crash-once", Some(&marker))).expect("client should start");

    let rendered = client
        .render_request(request("x"))
        .await
        .expect("request should recover after one restart");

    assert_eq!(rendered.accessibility_text, "x");
    assert_eq!(client.health().await.restart_count, 1);
    client.shutdown().await.expect("shutdown should succeed");
    remove_marker(&marker);
}

#[tokio::test]
async fn timeout_kills_worker_then_enters_restart_backoff() {
    let mut config = config("hang", None);
    config.render_timeout = Duration::from_millis(40);
    config.shutdown_timeout = Duration::from_millis(30);
    config.restart_interval = Duration::from_secs(2);
    let mut client = WorkerClient::start(config).expect("client should start");

    let error = client
        .render_request(request("x"))
        .await
        .expect_err("hung worker should time out");
    assert_eq!(error.code, RenderErrorCode::Timeout);
    let health = client.health().await;
    assert_eq!(health.state, WorkerState::Backoff);
    assert_eq!(health.restart_count, 1);

    let started = Instant::now();
    let error = client
        .render_request(request("y"))
        .await
        .expect_err("backoff should reject immediate restart");
    assert_eq!(error.code, RenderErrorCode::WorkerUnavailable);
    assert!(started.elapsed() < Duration::from_millis(100));
    client.shutdown().await.expect("shutdown should succeed");
}

#[tokio::test]
async fn incompatible_handshake_fails_closed_after_one_restart() {
    let mut client =
        WorkerClient::start(config("bad-handshake", None)).expect("client should start");

    let error = client
        .render_request(request("x"))
        .await
        .expect_err("incompatible worker should fail");

    assert_eq!(error.code, RenderErrorCode::Protocol);
    assert_eq!(client.health().await.restart_count, 1);
    client.shutdown().await.expect("shutdown should succeed");
}

#[tokio::test]
async fn active_svg_is_rejected_before_cache_and_worker_is_recycled() {
    let mut client = WorkerClient::start(config("unsafe", None)).expect("client should start");

    let error = client
        .render_request(request("x"))
        .await
        .expect_err("active SVG should fail");

    assert_eq!(error.code, RenderErrorCode::UnsafeOutput);
    assert_eq!(client.cache_stats().await.entries, 0);
    assert_eq!(client.health().await.state, WorkerState::Backoff);
    client.shutdown().await.expect("shutdown should succeed");
}

#[tokio::test]
async fn full_queue_fails_fast_without_blocking_callers() {
    let mut config = config("slow", None);
    config.queue_capacity = 1;
    config.render_timeout = Duration::from_secs(1);
    let client = Arc::new(WorkerClient::start(config).expect("client should start"));

    let first_client = Arc::clone(&client);
    let first = tokio::spawn(async move { first_client.render_request(request("a")).await });
    tokio::time::sleep(Duration::from_millis(25)).await;
    let second_client = Arc::clone(&client);
    let second = tokio::spawn(async move { second_client.render_request(request("b")).await });
    tokio::time::sleep(Duration::from_millis(10)).await;

    let error = client
        .render_request(request("c"))
        .await
        .expect_err("full queue should fail");
    assert_eq!(error.code, RenderErrorCode::QueueFull);
    first
        .await
        .expect("first task should join")
        .expect("first render should pass");
    second
        .await
        .expect("second task should join")
        .expect("second render should pass");

    let mut client = Arc::try_unwrap(client)
        .ok()
        .expect("test should own client");
    client.shutdown().await.expect("shutdown should succeed");
}

#[tokio::test]
async fn cancelled_caller_does_not_receive_or_cache_a_stale_result() {
    let client = Arc::new(WorkerClient::start(config("slow", None)).expect("client should start"));
    let cancelled_client = Arc::clone(&client);
    let cancelled =
        tokio::spawn(async move { cancelled_client.render_request(request("old")).await });
    tokio::time::sleep(Duration::from_millis(25)).await;
    cancelled.abort();
    assert!(
        cancelled
            .await
            .expect_err("task should be cancelled")
            .is_cancelled()
    );

    let rendered = client
        .render_request(request("current"))
        .await
        .expect("current request should render");
    assert_eq!(rendered.accessibility_text, "current");
    let stats = client.cache_stats().await;
    assert_eq!(stats.entries, 1);

    let mut client = Arc::try_unwrap(client)
        .ok()
        .expect("test should own client");
    client.shutdown().await.expect("shutdown should succeed");
}

#[tokio::test]
async fn shutdown_cancels_active_work_and_reaps_the_worker() {
    let mut config = config("hang", None);
    config.render_timeout = Duration::from_secs(2);
    config.shutdown_timeout = Duration::from_millis(100);
    let client = Arc::new(WorkerClient::start(config).expect("client should start"));
    let render_client = Arc::clone(&client);
    let render = tokio::spawn(async move { render_client.render_request(request("x")).await });
    tokio::time::sleep(Duration::from_millis(50)).await;
    render.abort();
    let _ = render.await;

    let mut client = Arc::try_unwrap(client)
        .ok()
        .expect("test should own client");
    let started = Instant::now();
    client.shutdown().await.expect("shutdown should succeed");
    assert!(started.elapsed() < Duration::from_millis(250));
    assert_eq!(client.health().await.state, WorkerState::Stopped);
}

fn config(mode: &str, marker: Option<&PathBuf>) -> WorkerClientConfig {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("support")
        .join("fake-worker.mjs");
    let mut command = WorkerCommand::new("node").arg(script).arg(mode);
    if let Some(marker) = marker {
        command = command.arg(marker);
    }
    let mut config = WorkerClientConfig::new(command);
    config.startup_timeout = Duration::from_millis(500);
    config.render_timeout = Duration::from_millis(500);
    config.shutdown_timeout = Duration::from_millis(100);
    config.restart_interval = Duration::from_millis(250);
    config.cache_limits = CacheLimits {
        max_entries: 8,
        max_bytes: 1024 * 1024,
    };
    config
}

fn request(source: &str) -> RenderRequest {
    RenderRequest {
        source: source.to_owned(),
        display_mode: true,
        foreground: Rgba::opaque(230, 237, 243),
        background: None,
        scale: 2.0,
        max_width_px: 1200,
    }
}

fn marker_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "latex-render-client-{label}-{}-{}",
        std::process::id(),
        NEXT_MARKER.fetch_add(1, Ordering::Relaxed)
    ))
}

fn remove_marker(path: &PathBuf) {
    let _ = fs::remove_file(path);
}
