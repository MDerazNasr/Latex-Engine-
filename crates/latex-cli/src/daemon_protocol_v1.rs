//! Version 1 wire contracts for the local Codex renderer daemon.

use latex_render_core::{RenderLimits, RenderRequest, Rgba};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

pub(crate) const DAEMON_PROTOCOL_VERSION: u32 = 1;
pub(crate) const MAX_DAEMON_REQUEST_LINE_BYTES: usize = 1024 * 1024;
pub(crate) const MAX_DAEMON_MESSAGE_BYTES: usize = 256 * 1024;
pub(crate) const MAX_DAEMON_EQUATIONS: usize = 32;
pub(crate) const MAX_DAEMON_PNG_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const MAX_DAEMON_TOTAL_PNG_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const MAX_DAEMON_RESPONSE_LINE_BYTES: usize = 12 * 1024 * 1024;

const MAX_DAEMON_ID_BYTES: usize = 128;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DaemonRequestV1 {
    protocol: u32,
    id: String,
    method: DaemonMethodV1,
    params: RenderMessageParamsV1,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum DaemonMethodV1 {
    RenderMessage,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RenderMessageParamsV1 {
    source: String,
    inline_dollars: InlineDollarsV1,
    foreground: String,
    background: String,
    scale: f32,
    max_width_px: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InlineDollarsV1 {
    Off,
    Smart,
    Always,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ValidatedRenderMessageV1 {
    pub(crate) id: String,
    pub(crate) source: String,
    pub(crate) inline_dollars: InlineDollarsV1,
    pub(crate) foreground: Rgba,
    pub(crate) background: Option<Rgba>,
    pub(crate) scale: f32,
    pub(crate) max_width_px: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DecodeErrorV1 {
    pub(crate) id: Option<String>,
    pub(crate) error: DaemonErrorV1,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DaemonErrorV1 {
    pub(crate) code: String,
    pub(crate) retryable: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum DaemonResponseV1 {
    Success(DaemonSuccessResponseV1),
    Error(DaemonErrorResponseV1),
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DaemonSuccessResponseV1 {
    protocol: u32,
    id: String,
    ok: bool,
    result: RenderMessageResultV1,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DaemonErrorResponseV1 {
    protocol: u32,
    id: Option<String>,
    ok: bool,
    error: DaemonErrorV1,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RenderMessageResultV1 {
    pub(crate) equations: Vec<EquationOutcomeV1>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(crate) enum EquationOutcomeV1 {
    Rendered {
        #[serde(rename = "startByte")]
        start_byte: usize,
        #[serde(rename = "endByte")]
        end_byte: usize,
        #[serde(rename = "displayMode")]
        display_mode: bool,
        #[serde(rename = "pngBase64")]
        png_base64: String,
        #[serde(rename = "widthPx")]
        width_px: u32,
        #[serde(rename = "heightPx")]
        height_px: u32,
        #[serde(rename = "baselinePx")]
        baseline_px: Option<f32>,
        #[serde(rename = "accessibilityText")]
        accessibility_text: String,
    },
    Failed {
        #[serde(rename = "startByte")]
        start_byte: usize,
        #[serde(rename = "endByte")]
        end_byte: usize,
        #[serde(rename = "displayMode")]
        display_mode: bool,
        error: DaemonErrorV1,
    },
}

impl DaemonResponseV1 {
    pub(crate) fn success(id: String, equations: Vec<EquationOutcomeV1>) -> Self {
        Self::Success(DaemonSuccessResponseV1 {
            protocol: DAEMON_PROTOCOL_VERSION,
            id,
            ok: true,
            result: RenderMessageResultV1 { equations },
        })
    }

    pub(crate) fn error(id: Option<String>, error: DaemonErrorV1) -> Self {
        Self::Error(DaemonErrorResponseV1 {
            protocol: DAEMON_PROTOCOL_VERSION,
            id,
            ok: false,
            error,
        })
    }
}

impl EquationOutcomeV1 {
    pub(crate) fn rendered(
        span: std::ops::Range<usize>,
        display_mode: bool,
        png_base64: String,
        dimensions: (u32, u32),
        baseline_px: Option<f32>,
        accessibility_text: String,
    ) -> Self {
        Self::Rendered {
            start_byte: span.start,
            end_byte: span.end,
            display_mode,
            png_base64,
            width_px: dimensions.0,
            height_px: dimensions.1,
            baseline_px,
            accessibility_text,
        }
    }

    pub(crate) fn failed(
        span: std::ops::Range<usize>,
        display_mode: bool,
        error: DaemonErrorV1,
    ) -> Self {
        Self::Failed {
            start_byte: span.start,
            end_byte: span.end,
            display_mode,
            error,
        }
    }
}

pub(crate) fn decode_request_v1(json: &[u8]) -> Result<ValidatedRenderMessageV1, DecodeErrorV1> {
    if json.len() > MAX_DAEMON_REQUEST_LINE_BYTES {
        return Err(DecodeErrorV1 {
            id: None,
            error: error("input_limit_exceeded", false),
        });
    }
    let value = serde_json::from_slice::<Value>(json).map_err(|_| DecodeErrorV1 {
        id: None,
        error: error("invalid_request", false),
    })?;
    let id = safe_id_from_value(&value);
    let request = serde_json::from_value::<DaemonRequestV1>(value).map_err(|_| DecodeErrorV1 {
        id: id.clone(),
        error: error("invalid_request", false),
    })?;
    validate_request(request).map_err(|error| DecodeErrorV1 { id, error })
}

fn validate_request(request: DaemonRequestV1) -> Result<ValidatedRenderMessageV1, DaemonErrorV1> {
    if request.protocol != DAEMON_PROTOCOL_VERSION {
        return Err(error("protocol", false));
    }
    if request.id.is_empty()
        || request.id.len() > MAX_DAEMON_ID_BYTES
        || request.id.chars().any(char::is_control)
    {
        return Err(error("invalid_request", false));
    }
    if request.params.source.is_empty() {
        return Err(error("invalid_request", false));
    }
    if request.params.source.len() > MAX_DAEMON_MESSAGE_BYTES {
        return Err(error("input_limit_exceeded", false));
    }
    if request.params.source.chars().any(is_unsafe_control) {
        return Err(error("invalid_request", false));
    }

    let foreground =
        parse_color(&request.params.foreground).ok_or_else(|| error("invalid_request", false))?;
    let background = if request.params.background == "transparent" {
        None
    } else {
        Some(
            parse_color(&request.params.background)
                .ok_or_else(|| error("invalid_request", false))?,
        )
    };
    let validation_request = RenderRequest {
        source: "x".to_owned(),
        display_mode: false,
        foreground,
        background,
        scale: request.params.scale,
        max_width_px: request.params.max_width_px,
    };
    validation_request
        .validate(&RenderLimits::default())
        .map_err(|_| error("invalid_request", false))?;

    Ok(ValidatedRenderMessageV1 {
        id: request.id,
        source: request.params.source,
        inline_dollars: request.params.inline_dollars,
        foreground,
        background,
        scale: request.params.scale,
        max_width_px: request.params.max_width_px,
    })
}

fn safe_id_from_value(value: &Value) -> Option<String> {
    let id = value.get("id")?.as_str()?;
    (!id.is_empty() && id.len() <= MAX_DAEMON_ID_BYTES && !id.chars().any(char::is_control))
        .then(|| id.to_owned())
}

fn parse_color(value: &str) -> Option<Rgba> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some(Rgba::opaque(
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ))
}

fn is_unsafe_control(character: char) -> bool {
    character.is_control() && !matches!(character, '\n' | '\r' | '\t')
}

pub(crate) fn error(code: &str, retryable: bool) -> DaemonErrorV1 {
    DaemonErrorV1 {
        code: code.to_owned(),
        retryable,
    }
}

#[cfg(test)]
#[path = "daemon_protocol_v1_tests.rs"]
mod tests;
