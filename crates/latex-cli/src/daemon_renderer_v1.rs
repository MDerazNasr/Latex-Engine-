//! Version 1 message segmentation and bounded PNG rendering.

use base64::Engine as _;
use base64::engine::general_purpose;
use latex_render_core::MathRenderer;
use latex_render_core::RenderError;
use latex_render_core::RenderErrorCode;
use latex_render_core::RenderLimits;
use latex_render_core::RenderRequest;
use latex_render_svg::RasterLimits;
use latex_render_svg::RasterRequest;
use latex_render_svg::SvgSanitizerLimits;
use latex_render_svg::rasterize_svg;
use latex_render_svg::sanitize_svg;
use latex_segmenter::InlineDollarMode;
use latex_segmenter::Segment;
use latex_segmenter::SegmentKind;
use latex_segmenter::Segmenter;
use latex_segmenter::SegmenterConfig;

use crate::daemon_protocol_v1::DaemonErrorV1;
use crate::daemon_protocol_v1::EquationOutcomeV1;
use crate::daemon_protocol_v1::InlineDollarsV1;
use crate::daemon_protocol_v1::MAX_DAEMON_EQUATIONS;
use crate::daemon_protocol_v1::MAX_DAEMON_PNG_BYTES;
use crate::daemon_protocol_v1::MAX_DAEMON_TOTAL_PNG_BYTES;
use crate::daemon_protocol_v1::ValidatedRenderMessageV1;
use crate::daemon_protocol_v1::error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DaemonRenderLimitsV1 {
    max_equations: usize,
    max_png_bytes: usize,
    max_total_png_bytes: usize,
}

impl Default for DaemonRenderLimitsV1 {
    fn default() -> Self {
        Self {
            max_equations: MAX_DAEMON_EQUATIONS,
            max_png_bytes: MAX_DAEMON_PNG_BYTES,
            max_total_png_bytes: MAX_DAEMON_TOTAL_PNG_BYTES,
        }
    }
}

pub(crate) async fn render_message_v1(
    renderer: &(impl MathRenderer + ?Sized),
    request: ValidatedRenderMessageV1,
) -> Result<Vec<EquationOutcomeV1>, DaemonErrorV1> {
    render_message_with_limits_v1(renderer, request, DaemonRenderLimitsV1::default()).await
}

async fn render_message_with_limits_v1(
    renderer: &(impl MathRenderer + ?Sized),
    request: ValidatedRenderMessageV1,
    limits: DaemonRenderLimitsV1,
) -> Result<Vec<EquationOutcomeV1>, DaemonErrorV1> {
    let equations = segment_equations(&request);
    if equations.len() > limits.max_equations {
        return Err(error("input_limit_exceeded", false));
    }

    let mut outcomes = Vec::with_capacity(equations.len());
    let mut total_png_bytes = 0usize;
    let mut aggregate_limit_reached = false;
    for segment in equations {
        if aggregate_limit_reached {
            outcomes.push(failed_outcome(
                &segment,
                error("output_limit_exceeded", false),
            ));
            continue;
        }

        match render_equation(renderer, &request, &segment, limits.max_png_bytes).await {
            Ok(rendered) => {
                let next_total = total_png_bytes.checked_add(rendered.png.len());
                if next_total.is_none_or(|bytes| bytes > limits.max_total_png_bytes) {
                    aggregate_limit_reached = true;
                    outcomes.push(failed_outcome(
                        &segment,
                        error("output_limit_exceeded", false),
                    ));
                    continue;
                }
                total_png_bytes = next_total.expect("aggregate PNG length was validated");
                outcomes.push(EquationOutcomeV1::rendered(
                    segment.span.start..segment.span.end,
                    matches!(segment.kind, SegmentKind::DisplayMath),
                    general_purpose::STANDARD.encode(rendered.png),
                    (rendered.width_px, rendered.height_px),
                    rendered.baseline_px,
                    rendered.accessibility_text,
                ));
            }
            Err(render_error) => outcomes.push(failed_outcome(
                &segment,
                daemon_error_from_render(render_error),
            )),
        }
    }
    Ok(outcomes)
}

fn segment_equations(request: &ValidatedRenderMessageV1) -> Vec<Segment> {
    let inline_dollars = match request.inline_dollars {
        InlineDollarsV1::Off => InlineDollarMode::Off,
        InlineDollarsV1::Smart => InlineDollarMode::Smart,
        InlineDollarsV1::Always => InlineDollarMode::Always,
    };
    let mut segmenter = Segmenter::with_config(SegmenterConfig { inline_dollars });
    let mut segments = segmenter.push(&request.source);
    segments.extend(segmenter.finish());
    segments
        .into_iter()
        .filter(|segment| {
            matches!(
                segment.kind,
                SegmentKind::InlineMath | SegmentKind::DisplayMath
            )
        })
        .collect()
}

async fn render_equation(
    renderer: &(impl MathRenderer + ?Sized),
    message: &ValidatedRenderMessageV1,
    segment: &Segment,
    max_png_bytes: usize,
) -> Result<RenderedEquationV1, RenderError> {
    let request = RenderRequest {
        source: segment.content.clone(),
        display_mode: matches!(segment.kind, SegmentKind::DisplayMath),
        foreground: message.foreground,
        background: message.background,
        scale: message.scale,
        max_width_px: message.max_width_px,
    };
    let rendered = renderer.render(request).await?;
    rendered.validate(&RenderLimits::default())?;
    let width_px = rendered.width_px;
    let height_px = rendered.height_px;
    let baseline_px = rendered.baseline_px;
    let accessibility_text = rendered.accessibility_text;
    let svg = rendered.svg;
    let raster = tokio::task::spawn_blocking(move || {
        let sanitized = sanitize_svg(&svg, SvgSanitizerLimits::default())?;
        rasterize_svg(
            &sanitized,
            RasterRequest {
                width_px,
                height_px,
            },
            RasterLimits {
                max_png_bytes,
                ..RasterLimits::default()
            },
        )
    })
    .await
    .map_err(|_| {
        RenderError::new(
            RenderErrorCode::RenderFailed,
            "Raster task ended without a result",
            false,
        )
    })??;

    Ok(RenderedEquationV1 {
        png: raster.bytes,
        width_px: raster.width_px,
        height_px: raster.height_px,
        baseline_px,
        accessibility_text,
    })
}

fn failed_outcome(segment: &Segment, error: DaemonErrorV1) -> EquationOutcomeV1 {
    EquationOutcomeV1::failed(
        segment.span.start..segment.span.end,
        matches!(segment.kind, SegmentKind::DisplayMath),
        error,
    )
}

fn daemon_error_from_render(render_error: RenderError) -> DaemonErrorV1 {
    let code = match render_error.code {
        RenderErrorCode::InvalidRequest => "invalid_request",
        RenderErrorCode::InputLimitExceeded => "input_limit_exceeded",
        RenderErrorCode::OutputLimitExceeded => "output_limit_exceeded",
        RenderErrorCode::InvalidTex => "invalid_tex",
        RenderErrorCode::Protocol => "protocol",
        RenderErrorCode::WorkerUnavailable => "worker_unavailable",
        RenderErrorCode::Timeout => "timeout",
        RenderErrorCode::QueueFull => "queue_full",
        RenderErrorCode::Cancelled => "cancelled",
        RenderErrorCode::UnsafeOutput => "unsafe_output",
        RenderErrorCode::RenderFailed => "render_failed",
        _ => "unknown",
    };
    error(code, render_error.retryable)
}

struct RenderedEquationV1 {
    png: Vec<u8>,
    width_px: u32,
    height_px: u32,
    baseline_px: Option<f32>,
    accessibility_text: String,
}

#[cfg(test)]
#[path = "daemon_renderer_v1_tests.rs"]
mod tests;
