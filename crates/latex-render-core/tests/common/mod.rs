#![allow(dead_code)]

use latex_render_core::{RenderRequest, RenderedMath, Rgba};

pub fn request(source: &str) -> RenderRequest {
    RenderRequest {
        source: source.to_owned(),
        display_mode: true,
        foreground: Rgba::opaque(230, 237, 243),
        background: None,
        scale: 2.0,
        max_width_px: 1200,
    }
}

pub fn result(key: &str) -> RenderedMath {
    RenderedMath {
        svg: b"<svg></svg>".to_vec(),
        width_px: 64,
        height_px: 32,
        baseline_px: Some(24.0),
        accessibility_text: "x squared".to_owned(),
        cache_key: key.to_owned(),
    }
}
