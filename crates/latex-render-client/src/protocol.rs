//! Strict wire types for worker protocol version one.

use latex_render_core::{RenderError, RenderErrorCode, RenderLimits, RenderRequest, RenderedMath};
use serde::{Deserialize, Serialize};

use crate::{WORKER_PROTOCOL_VERSION, WorkerClientConfig};

pub(crate) fn response_line_limit(limits: &RenderLimits) -> usize {
    limits
        .max_svg_bytes
        .saturating_mul(2)
        .saturating_add(limits.max_json_line_bytes)
}

#[derive(Debug, Serialize)]
struct RequestEnvelope<'a> {
    protocol: u32,
    id: &'a str,
    method: &'static str,
    params: RequestParams<'a>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestParams<'a> {
    source: &'a str,
    display_mode: bool,
    foreground: String,
    background: String,
    scale: f32,
    max_width_px: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadyEnvelope {
    protocol: u32,
    #[serde(rename = "type")]
    kind: String,
    renderer: RendererDescriptor,
    capabilities: WorkerCapabilities,
    limits: WorkerLimits,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RendererDescriptor {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WorkerCapabilities {
    formats: Vec<String>,
    display_modes: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WorkerLimits {
    max_source_bytes: usize,
    max_json_line_bytes: usize,
    max_svg_bytes: usize,
    max_width_px: u32,
    max_height_px: u32,
    min_scale: f32,
    max_scale: f32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResponseEnvelope {
    protocol: u32,
    id: Option<String>,
    ok: bool,
    #[serde(default)]
    result: Option<ResponseResult>,
    #[serde(default)]
    error: Option<ResponseError>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ResponseResult {
    svg_utf8: String,
    width_px: u32,
    height_px: u32,
    baseline_px: f32,
    accessibility_text: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResponseError {
    code: String,
    message: String,
    retryable: bool,
    position: Option<usize>,
}

pub(crate) fn encode_request(
    id: &str,
    request: &RenderRequest,
    limits: &RenderLimits,
) -> Result<Vec<u8>, RenderError> {
    let foreground = request
        .foreground
        .to_rgb_hex()
        .ok_or_else(|| invalid("Foreground must be fully opaque for the MathJax worker"))?;
    let background = match request.background {
        None => "transparent".to_owned(),
        Some(color) if color.is_transparent() => "transparent".to_owned(),
        Some(color) => color.to_rgb_hex().ok_or_else(|| {
            invalid("Background must be fully opaque or transparent for the MathJax worker")
        })?,
    };
    let envelope = RequestEnvelope {
        protocol: WORKER_PROTOCOL_VERSION,
        id,
        method: "render",
        params: RequestParams {
            source: &request.source,
            display_mode: request.display_mode,
            foreground,
            background,
            scale: request.scale,
            max_width_px: request.max_width_px,
        },
    };
    let bytes = serde_json::to_vec(&envelope)
        .map_err(|_| protocol_error("Render request could not be encoded"))?;
    if bytes.len() > limits.max_json_line_bytes {
        return Err(RenderError::new(
            RenderErrorCode::InputLimitExceeded,
            format!(
                "Encoded request exceeds {} UTF 8 bytes",
                limits.max_json_line_bytes
            ),
            false,
        ));
    }
    Ok(bytes)
}

pub(crate) fn decode_ready(
    line: &[u8],
    config: &WorkerClientConfig,
) -> Result<String, RenderError> {
    let ready: ReadyEnvelope = serde_json::from_slice(line)
        .map_err(|_| protocol_error("Worker ready handshake is malformed"))?;
    if ready.protocol != WORKER_PROTOCOL_VERSION || ready.kind != "ready" {
        return Err(protocol_error(
            "Worker protocol version or message type is incompatible",
        ));
    }
    if ready.renderer.name != config.expected_renderer_name
        || ready.renderer.version != config.expected_renderer_version
    {
        return Err(protocol_error(
            "Worker renderer name or version is incompatible",
        ));
    }
    if !ready
        .capabilities
        .formats
        .iter()
        .any(|value| value == "svg")
        || !ready
            .capabilities
            .display_modes
            .iter()
            .any(|value| value == "inline")
        || !ready
            .capabilities
            .display_modes
            .iter()
            .any(|value| value == "display")
    {
        return Err(protocol_error(
            "Worker does not support required render modes",
        ));
    }
    validate_worker_limits(&ready.limits, &config.render_limits)?;
    Ok(ready.renderer.version)
}

pub(crate) fn decode_response(
    line: &[u8],
    expected_id: &str,
    cache_key: String,
    limits: &RenderLimits,
) -> Result<RenderedMath, RenderError> {
    let response: ResponseEnvelope =
        serde_json::from_slice(line).map_err(|_| protocol_error("Worker response is malformed"))?;
    if response.protocol != WORKER_PROTOCOL_VERSION || response.id.as_deref() != Some(expected_id) {
        return Err(protocol_error("Worker response correlation is invalid"));
    }
    match (response.ok, response.result, response.error) {
        (true, Some(result), None) => {
            let rendered = RenderedMath {
                svg: result.svg_utf8.into_bytes(),
                width_px: result.width_px,
                height_px: result.height_px,
                baseline_px: Some(result.baseline_px),
                accessibility_text: result.accessibility_text,
                cache_key,
            };
            rendered.validate(limits).map_err(|_| {
                RenderError::new(
                    RenderErrorCode::UnsafeOutput,
                    "Worker returned invalid render output",
                    false,
                )
            })?;
            Ok(rendered)
        }
        (false, None, Some(error)) => Err(map_worker_error(error)),
        _ => Err(protocol_error(
            "Worker response success shape is inconsistent",
        )),
    }
}

fn validate_worker_limits(
    worker: &WorkerLimits,
    configured: &RenderLimits,
) -> Result<(), RenderError> {
    if worker.max_source_bytes < configured.max_source_bytes
        || worker.max_json_line_bytes < configured.max_json_line_bytes
        || worker.max_svg_bytes < configured.max_svg_bytes
        || worker.max_width_px < configured.max_width_px
        || worker.max_height_px < configured.max_height_px
        || worker.min_scale > configured.min_scale
        || worker.max_scale < configured.max_scale
    {
        return Err(protocol_error("Worker resource limits are incompatible"));
    }
    Ok(())
}

fn map_worker_error(error: ResponseError) -> RenderError {
    let code = match error.code.as_str() {
        "INVALID_REQUEST" => RenderErrorCode::InvalidRequest,
        "INVALID_TEX" => RenderErrorCode::InvalidTex,
        "INPUT_LIMIT_EXCEEDED" => RenderErrorCode::InputLimitExceeded,
        "OUTPUT_LIMIT_EXCEEDED" => RenderErrorCode::OutputLimitExceeded,
        "RENDER_FAILED" => RenderErrorCode::RenderFailed,
        "INVALID_JSON" => RenderErrorCode::Protocol,
        _ => RenderErrorCode::Protocol,
    };
    let mut mapped = RenderError::new(code, error.message, error.retryable);
    mapped.position = error.position;
    mapped
}

fn invalid(message: impl Into<String>) -> RenderError {
    RenderError::new(RenderErrorCode::InvalidRequest, message, false)
}

fn protocol_error(message: impl Into<String>) -> RenderError {
    RenderError::new(RenderErrorCode::Protocol, message, true)
}
