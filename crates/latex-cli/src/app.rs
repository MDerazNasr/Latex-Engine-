//! Supervised rendering command execution.

use std::io::{self, IsTerminal, Read};

use latex_render_client::{WorkerClient, WorkerClientConfig, WorkerCommand};
use latex_render_core::{MAX_SOURCE_BYTES, RenderRequest, RenderedMath};
use latex_render_svg::{
    RasterLimits, RasterRequest, SvgSanitizerLimits, rasterize_svg, sanitize_svg,
};

use crate::args::{CliCommand, OutputFormat, RenderOptions, WorkerOptions};
use crate::error::{CliError, CliErrorKind};
use crate::output::CommandOutput;
use crate::worker_path::resolve_worker;

pub(crate) async fn execute(command: CliCommand) -> Result<CommandOutput, CliError> {
    match command {
        CliCommand::Render(options) => execute_render(options).await,
        CliCommand::Check(_) => Err(CliError::new(
            CliErrorKind::Internal,
            "The check command is not available in this build",
        )),
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
    let rendered = render_once(&options.worker, request).await?;
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

async fn render_once(
    options: &WorkerOptions,
    request: RenderRequest,
) -> Result<RenderedMath, CliError> {
    let worker = resolve_worker(options)?;
    let command = WorkerCommand::new(&options.node).arg(worker);
    let mut client =
        WorkerClient::start(WorkerClientConfig::new(command)).map_err(CliError::from_render)?;
    let render_result = client.render_request(request).await;
    let shutdown_result = client.shutdown().await;
    match (render_result, shutdown_result) {
        (Ok(rendered), Ok(())) => Ok(rendered),
        (Err(error), _) => Err(CliError::from_render(error)),
        (Ok(_), Err(error)) => Err(CliError::from_render(error)),
    }
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
