#![doc = "Release performance gate for the independent renderer."]

mod metrics;

#[cfg(test)]
mod metrics_tests;

use std::ffi::OsString;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use latex_render_client::{WorkerClient, WorkerClientConfig, WorkerCommand, WorkerState};
use latex_render_core::{RenderError, RenderRequest, Rgba};
use latex_segmenter::Segmenter;
use metrics::{milliseconds, percentile_95};

const SIMPLE_SAMPLES: usize = 60;
const COMPLEX_SAMPLES: usize = 30;
const CACHED_SAMPLES: usize = 1_000;
const SEGMENTER_SAMPLES: usize = 10_000;

const FIRST_RENDER_TARGET: Duration = Duration::from_millis(1_500);
const SIMPLE_TARGET: Duration = Duration::from_millis(150);
const COMPLEX_TARGET: Duration = Duration::from_millis(500);
const CACHED_TARGET: Duration = Duration::from_millis(10);
const SEGMENTER_TARGET: Duration = Duration::from_millis(5);

#[tokio::main(flavor = "current_thread")]
async fn main() {
    match run().await {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("latex-bench: {error}");
            std::process::exit(2);
        }
    }
}

async fn run() -> Result<bool, BenchError> {
    if cfg!(debug_assertions) {
        return Err(BenchError::message(
            "Run with cargo run --release -p latex-bench",
        ));
    }
    let worker = worker_path()?;
    let node = std::env::var_os("LATEX_RENDER_NODE").unwrap_or_else(|| OsString::from("node"));
    if node.is_empty() {
        return Err(BenchError::message("LATEX_RENDER_NODE must not be empty"));
    }
    let command = WorkerCommand::new(PathBuf::from(node)).arg(worker);
    let mut client = WorkerClient::start(WorkerClientConfig::new(command))?;

    let started = Instant::now();
    client.render_request(request("x^2", false)).await?;
    let first_render = started.elapsed();

    let mut simple = measure_renders(&client, SIMPLE_SAMPLES, |index| {
        request(format!("x_{{{index}}}^2+1"), false)
    })
    .await?;
    let mut complex = measure_renders(&client, COMPLEX_SAMPLES, |index| {
        request(
            format!(
                r"\sum_{{i=1}}^n \frac{{(-1)^i}}{{i^2}} + \begin{{pmatrix}}1&0\\0&1\end{{pmatrix}} + {index}"
            ),
            true,
        )
    })
    .await?;

    let cached_request = request("E=mc^2", false);
    client.render_request(cached_request.clone()).await?;
    let mut cached = Vec::with_capacity(CACHED_SAMPLES);
    for _ in 0..CACHED_SAMPLES {
        let started = Instant::now();
        black_box(client.render_request(cached_request.clone()).await?);
        cached.push(started.elapsed());
    }
    let health = client.health().await;
    client.shutdown().await?;
    if health.state != WorkerState::Ready {
        return Err(BenchError::message(
            "Worker did not remain ready during the benchmark",
        ));
    }

    let mut segmenter = measure_segmenter();
    let summary = Summary {
        first_render,
        simple_p95: require_percentile(&mut simple)?,
        complex_p95: require_percentile(&mut complex)?,
        cached_p95: require_percentile(&mut cached)?,
        segmenter_p95: require_percentile(&mut segmenter)?,
    };
    summary.print(health.renderer_version.as_deref().unwrap_or("unknown"));
    Ok(summary.passed())
}

async fn measure_renders(
    client: &WorkerClient,
    count: usize,
    source: impl Fn(usize) -> RenderRequest,
) -> Result<Vec<Duration>, BenchError> {
    let mut samples = Vec::with_capacity(count);
    for index in 0..count {
        let started = Instant::now();
        black_box(client.render_request(source(index)).await?);
        samples.push(started.elapsed());
    }
    Ok(samples)
}

fn measure_segmenter() -> Vec<Duration> {
    let mut samples = Vec::with_capacity(SEGMENTER_SAMPLES);
    for _ in 0..SEGMENTER_SAMPLES {
        let mut segmenter = Segmenter::new();
        let started = Instant::now();
        black_box(segmenter.push("The result is \\(x^2+1\\) and remains stable."));
        samples.push(started.elapsed());
    }
    samples
}

fn request(source: impl Into<String>, display_mode: bool) -> RenderRequest {
    RenderRequest {
        source: source.into(),
        display_mode,
        foreground: Rgba::opaque(230, 237, 243),
        background: None,
        scale: 2.0,
        max_width_px: 1200,
    }
}

fn worker_path() -> Result<PathBuf, BenchError> {
    let mut arguments = std::env::args_os().skip(1);
    let worker = arguments.next().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("benchmark crate should be inside the repository")
            .join("renderer/mathjax-worker/dist/src/server.js")
    });
    if arguments.next().is_some() {
        return Err(BenchError::message(
            "Accepts at most one MathJax worker path",
        ));
    }
    if !worker.is_file() {
        return Err(BenchError::message(
            "Build the MathJax worker or provide its server.js path",
        ));
    }
    Ok(worker)
}

fn require_percentile(samples: &mut [Duration]) -> Result<Duration, BenchError> {
    percentile_95(samples).ok_or_else(|| BenchError::message("Benchmark sample set was empty"))
}

struct Summary {
    first_render: Duration,
    simple_p95: Duration,
    complex_p95: Duration,
    cached_p95: Duration,
    segmenter_p95: Duration,
}

impl Summary {
    fn passed(&self) -> bool {
        self.first_render <= FIRST_RENDER_TARGET
            && self.simple_p95 <= SIMPLE_TARGET
            && self.complex_p95 <= COMPLEX_TARGET
            && self.cached_p95 <= CACHED_TARGET
            && self.segmenter_p95 <= SEGMENTER_TARGET
    }

    fn print(&self, renderer_version: &str) {
        println!(
            "benchmark_status={}",
            if self.passed() { "ok" } else { "failed" }
        );
        println!("renderer_version={renderer_version}");
        println!("simple_samples={SIMPLE_SAMPLES}");
        println!("complex_samples={COMPLEX_SAMPLES}");
        println!("cached_samples={CACHED_SAMPLES}");
        println!("segmenter_samples={SEGMENTER_SAMPLES}");
        print_metric("first_render", self.first_render, FIRST_RENDER_TARGET);
        print_metric("uncached_simple_p95", self.simple_p95, SIMPLE_TARGET);
        print_metric("complex_p95", self.complex_p95, COMPLEX_TARGET);
        print_metric("cached_p95", self.cached_p95, CACHED_TARGET);
        print_metric("segmenter_delta_p95", self.segmenter_p95, SEGMENTER_TARGET);
    }
}

fn print_metric(name: &str, value: Duration, target: Duration) {
    println!("{name}_ms={:.3}", milliseconds(value));
    println!("{name}_target_ms={:.3}", milliseconds(target));
    println!("{name}_passed={}", value <= target);
}

struct BenchError(String);

impl BenchError {
    fn message(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl From<RenderError> for BenchError {
    fn from(error: RenderError) -> Self {
        Self(format!("{:?}: {}", error.code, error.message))
    }
}

impl std::fmt::Display for BenchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
