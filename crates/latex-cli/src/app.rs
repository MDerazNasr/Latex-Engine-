//! Supervised rendering command execution.

use std::fmt::Write as _;
use std::io::{self, IsTerminal, Read};

use latex_render_client::{
    WORKER_PROTOCOL_VERSION, WorkerClient, WorkerClientConfig, WorkerCommand, WorkerHealth,
    WorkerState,
};
use latex_render_core::{
    CacheStats, MAX_SOURCE_BYTES, RenderErrorCode, RenderLimits, RenderRequest, RenderedMath, Rgba,
};
use latex_render_svg::{
    RASTERIZER_VERSION, RasterLimits, RasterRequest, SVG_POLICY_VERSION_LABEL, SvgSanitizerLimits,
    rasterize_svg, sanitize_svg,
};
use latex_terminal::{TerminalEnvironment, detect_terminal_support};

use crate::args::{CliCommand, OutputFormat, RenderOptions, WorkerOptions};
use crate::error::{CliError, CliErrorKind};
use crate::output::CommandOutput;
use crate::worker_path::resolve_worker;

pub(crate) async fn execute(command: CliCommand) -> Result<CommandOutput, CliError> {
    match command {
        CliCommand::Render(options) => execute_render(options).await,
        CliCommand::Check(options) => execute_check(options).await,
        CliCommand::Doctor(options) => execute_doctor(options).await,
    }
}

async fn execute_render(options: RenderOptions) -> Result<CommandOutput, CliError> {
    let source = read_source(options.source)?;
    let request = RenderRequest {
        source,
        display_mode: options.display_mode,
        foreground: options.foreground,
        background: options.background,
        scale: options.scale,
        max_width_px: options.max_width_px,
    };
    let rendered = render_once(&options.worker, request).await?.rendered;
    let bytes = match options.format {
        OutputFormat::Svg => rendered.svg,
        OutputFormat::Png => rasterize(rendered).await?,
    };
    Ok(CommandOutput {
        bytes,
        path: options.output,
        force: options.force,
    })
}

async fn execute_check(options: WorkerOptions) -> Result<CommandOutput, CliError> {
    let request = RenderRequest {
        source: "x^2".to_owned(),
        display_mode: false,
        foreground: Rgba::opaque(230, 237, 243),
        background: None,
        scale: 2.0,
        max_width_px: 1200,
    };
    let session = render_once(&options, request).await?;
    if session.health.state != WorkerState::Ready {
        return Err(CliError::new(
            CliErrorKind::Worker,
            "Worker did not remain ready after the check render",
        ));
    }
    let version = session.health.renderer_version.as_deref().ok_or_else(|| {
        CliError::new(
            CliErrorKind::Worker,
            "Worker health omitted its renderer version",
        )
    })?;
    let report = check_report(version, &session.health, session.cache, session.limits)?;
    let png = rasterize(session.rendered).await?;
    if !png.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(CliError::new(
            CliErrorKind::Render,
            "Native raster check did not produce a PNG",
        ));
    }
    Ok(CommandOutput {
        bytes: report.into_bytes(),
        path: None,
        force: false,
    })
}

async fn execute_doctor(options: WorkerOptions) -> Result<CommandOutput, CliError> {
    let checked = execute_check(options).await?;
    let pipeline = String::from_utf8(checked.bytes).map_err(|_| {
        CliError::new(
            CliErrorKind::Internal,
            "Check report did not contain valid UTF 8",
        )
    })?;
    let environment = TerminalEnvironment::from_current_process(io::stdout().is_terminal());
    Ok(CommandOutput {
        bytes: doctor_report(pipeline, &environment)?.into_bytes(),
        path: None,
        force: false,
    })
}

async fn render_once(
    options: &WorkerOptions,
    request: RenderRequest,
) -> Result<RenderSession, CliError> {
    let worker = resolve_worker(options)?;
    let command = WorkerCommand::new(&options.node).arg(worker);
    let config = WorkerClientConfig::new(command);
    let limits = config.render_limits;
    let mut client = WorkerClient::start(config).map_err(CliError::from_render)?;
    let render_result = client.render_request(request).await;
    let health = client.health().await;
    let cache = client.cache_stats().await;
    let shutdown_result = client.shutdown().await;
    match (render_result, shutdown_result) {
        (Ok(rendered), Ok(())) => Ok(RenderSession {
            rendered,
            health,
            cache,
            limits,
        }),
        (Err(error), _) => Err(CliError::from_render(error)),
        (Ok(_), Err(error)) => Err(CliError::from_render(error)),
    }
}

pub(crate) fn check_report(
    renderer_version: &str,
    health: &WorkerHealth,
    cache: CacheStats,
    limits: RenderLimits,
) -> Result<String, CliError> {
    let mut report = String::new();
    writeln!(report, "status=ok").map_err(report_error)?;
    writeln!(report, "protocol={WORKER_PROTOCOL_VERSION}").map_err(report_error)?;
    writeln!(report, "renderer=mathjax").map_err(report_error)?;
    writeln!(report, "renderer_version={renderer_version}").map_err(report_error)?;
    writeln!(report, "sanitizer={SVG_POLICY_VERSION_LABEL}").map_err(report_error)?;
    writeln!(report, "rasterizer={RASTERIZER_VERSION}").map_err(report_error)?;
    writeln!(report, "worker_state={}", state_name(health.state)).map_err(report_error)?;
    writeln!(report, "restart_count={}", health.restart_count).map_err(report_error)?;
    writeln!(report, "last_error={}", error_name(health.last_error)).map_err(report_error)?;
    writeln!(report, "cache_entries={}", cache.entries).map_err(report_error)?;
    writeln!(report, "cache_bytes={}", cache.bytes).map_err(report_error)?;
    writeln!(report, "cache_hits={}", cache.hits).map_err(report_error)?;
    writeln!(report, "cache_misses={}", cache.misses).map_err(report_error)?;
    writeln!(report, "max_source_bytes={}", limits.max_source_bytes).map_err(report_error)?;
    writeln!(report, "max_json_line_bytes={}", limits.max_json_line_bytes).map_err(report_error)?;
    writeln!(report, "max_svg_bytes={}", limits.max_svg_bytes).map_err(report_error)?;
    writeln!(report, "max_width_px={}", limits.max_width_px).map_err(report_error)?;
    writeln!(report, "max_height_px={}", limits.max_height_px).map_err(report_error)?;
    writeln!(report, "min_scale={}", limits.min_scale).map_err(report_error)?;
    writeln!(report, "max_scale={}", limits.max_scale).map_err(report_error)?;
    Ok(report)
}

pub(crate) fn doctor_report(
    mut pipeline: String,
    environment: &TerminalEnvironment,
) -> Result<String, CliError> {
    let support = detect_terminal_support(environment);
    let fallback = support
        .fallback_reason
        .map(|reason| reason.diagnostic_name())
        .unwrap_or("none");
    writeln!(
        pipeline,
        "terminal_stdout_tty={}",
        environment.stdout_is_terminal
    )
    .map_err(report_error)?;
    writeln!(
        pipeline,
        "terminal_backend={}",
        support.backend.diagnostic_name()
    )
    .map_err(report_error)?;
    writeln!(pipeline, "terminal_fallback={fallback}").map_err(report_error)?;
    writeln!(pipeline, "terminal_ssh={}", environment.ssh).map_err(report_error)?;
    writeln!(pipeline, "terminal_tmux={}", environment.tmux).map_err(report_error)?;
    writeln!(pipeline, "terminal_zellij={}", environment.zellij).map_err(report_error)?;
    writeln!(pipeline, "terminal_screen={}", environment.screen).map_err(report_error)?;
    Ok(pipeline)
}

fn report_error(_: std::fmt::Error) -> CliError {
    CliError::new(
        CliErrorKind::Internal,
        "Diagnostic report could not be constructed",
    )
}

fn state_name(state: WorkerState) -> &'static str {
    match state {
        WorkerState::Idle => "idle",
        WorkerState::Starting => "starting",
        WorkerState::Ready => "ready",
        WorkerState::Backoff => "backoff",
        WorkerState::Stopped => "stopped",
    }
}

fn error_name(error: Option<RenderErrorCode>) -> &'static str {
    match error {
        None => "none",
        Some(RenderErrorCode::InvalidRequest) => "invalid_request",
        Some(RenderErrorCode::InputLimitExceeded) => "input_limit_exceeded",
        Some(RenderErrorCode::OutputLimitExceeded) => "output_limit_exceeded",
        Some(RenderErrorCode::InvalidTex) => "invalid_tex",
        Some(RenderErrorCode::Protocol) => "protocol",
        Some(RenderErrorCode::WorkerUnavailable) => "worker_unavailable",
        Some(RenderErrorCode::Timeout) => "timeout",
        Some(RenderErrorCode::QueueFull) => "queue_full",
        Some(RenderErrorCode::Cancelled) => "cancelled",
        Some(RenderErrorCode::UnsafeOutput) => "unsafe_output",
        Some(RenderErrorCode::RenderFailed) => "render_failed",
        Some(_) => "unknown",
    }
}

struct RenderSession {
    rendered: RenderedMath,
    health: WorkerHealth,
    cache: CacheStats,
    limits: RenderLimits,
}

async fn rasterize(rendered: RenderedMath) -> Result<Vec<u8>, CliError> {
    let svg = sanitize_svg(&rendered.svg, SvgSanitizerLimits::default())
        .map_err(CliError::from_render)?;
    let request = RasterRequest {
        width_px: rendered.width_px,
        height_px: rendered.height_px,
    };
    tokio::task::spawn_blocking(move || rasterize_svg(&svg, request, RasterLimits::default()))
        .await
        .map_err(|_| CliError::new(CliErrorKind::Internal, "Raster task ended without a result"))?
        .map(|image| image.bytes)
        .map_err(CliError::from_render)
}

fn read_source(argument: Option<String>) -> Result<String, CliError> {
    if let Some(source) = argument {
        return Ok(source);
    }
    let stdin = io::stdin();
    if stdin.is_terminal() {
        return Err(CliError::new(
            CliErrorKind::Usage,
            "TeX source is required when stdin is a terminal",
        ));
    }
    let mut bytes = Vec::new();
    stdin
        .lock()
        .take(MAX_SOURCE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| CliError::new(CliErrorKind::Usage, "Standard input could not be read"))?;
    if bytes.len() > MAX_SOURCE_BYTES {
        return Err(CliError::new(
            CliErrorKind::Usage,
            format!("TeX source exceeds {MAX_SOURCE_BYTES} UTF 8 bytes"),
        ));
    }
    String::from_utf8(bytes).map_err(|_| {
        CliError::new(
            CliErrorKind::Usage,
            "Standard input must contain valid UTF 8",
        )
    })
}
