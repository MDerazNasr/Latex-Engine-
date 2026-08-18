//! Real renderer and terminal presentation pipeline.

use std::error::Error;
use std::num::NonZeroU32;

use latex_render_client::{WorkerClient, WorkerClientConfig, WorkerCommand};
use latex_render_core::{MAX_WIDTH_PX, RenderRequest, RenderedMath};
use latex_render_svg::{RasterLimits, SanitizedSvg, SvgSanitizerLimits, sanitize_svg};
use latex_terminal::{
    LayoutPolicy, LocalPngStore, MathGeometry, PublishOutcome, TerminalAppearance, TerminalBackend,
    TerminalGeometry, TerminalPresenter, resolve_math_theme,
};

use crate::args::{Arguments, GeometrySpec};

const IMAGE_ID: NonZeroU32 = NonZeroU32::new(0xC0E2).expect("image identifier is nonzero");

pub(crate) struct RenderedEquation {
    svg: SanitizedSvg,
    metrics: RenderedMath,
    display_mode: bool,
}

pub(crate) async fn render_equation(
    arguments: &Arguments,
    geometry: GeometrySpec,
) -> Result<RenderedEquation, Box<dyn Error>> {
    let terminal = terminal_geometry(geometry)?;
    let theme = resolve_math_theme(arguments.theme, TerminalAppearance::default());
    let request = RenderRequest {
        source: arguments.source.clone(),
        display_mode: arguments.display_mode,
        foreground: theme.foreground(),
        background: theme.background(),
        scale: 2.0,
        max_width_px: terminal.width_px().min(MAX_WIDTH_PX),
    };
    let command = WorkerCommand::new(&arguments.node).arg(&arguments.worker);
    let mut client = WorkerClient::start(WorkerClientConfig::new(command))?;
    let rendered = client.render_request(request).await;
    let shutdown = client.shutdown().await;
    let rendered = match (rendered, shutdown) {
        (Ok(rendered), Ok(())) => rendered,
        (Err(error), _) => return Err(error.into()),
        (Ok(_), Err(error)) => return Err(error.into()),
    };
    let svg = sanitize_svg(&rendered.svg, SvgSanitizerLimits::default())?;
    Ok(RenderedEquation {
        svg,
        metrics: rendered,
        display_mode: arguments.display_mode,
    })
}

pub(crate) fn publish_equation(
    presenter: &mut TerminalPresenter,
    store: Option<&mut LocalPngStore>,
    equation: &RenderedEquation,
    geometry: GeometrySpec,
    row: u16,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let terminal = terminal_geometry(geometry)?;
    let math = MathGeometry::new(
        equation.metrics.width_px,
        equation.metrics.height_px,
        equation.metrics.baseline_px,
        equation.display_mode,
    )?;
    let layout = latex_terminal::layout_math(terminal, math, 0, LayoutPolicy::default());
    let job = presenter
        .begin(IMAGE_ID, row, layout)?
        .ok_or("text backend did not create a presentation job")?;
    let raster =
        latex_terminal::rasterize_presentation(&equation.svg, job, RasterLimits::default())?;
    let source = match store {
        Some(store) => store.store_png(raster.png_bytes())?,
        None => raster.direct_source(),
    };
    match presenter.publish(raster, source)? {
        PublishOutcome::Published(command) => Ok(command),
        PublishOutcome::Stale => Err("synchronous smoke frame became stale".into()),
    }
}

pub(crate) fn source_store(
    backend: TerminalBackend,
) -> Result<Option<LocalPngStore>, Box<dyn Error>> {
    match backend {
        TerminalBackend::KittyLocalFile => Ok(Some(LocalPngStore::create(Default::default())?)),
        TerminalBackend::KittyDirect => Ok(None),
        TerminalBackend::Text => Ok(None),
    }
}

fn terminal_geometry(geometry: GeometrySpec) -> Result<TerminalGeometry, Box<dyn Error>> {
    Ok(TerminalGeometry::new(
        geometry.columns,
        geometry.rows,
        geometry.width_px,
        geometry.height_px,
    )?)
}
