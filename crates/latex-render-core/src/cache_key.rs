//! Stable cache keys for rendered math.

use sha2::{Digest, Sha256};
use std::fmt::Write;

use crate::{RENDER_CORE_VERSION, RenderRequest};

/// Version data that can invalidate renderer and rasterizer output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CacheKeyContext<'a> {
    /// Renderer protocol version.
    pub protocol_version: u32,
    /// Renderer implementation version.
    pub renderer_version: &'a str,
    /// Fixed extension and macro policy version.
    pub macro_policy_version: &'a str,
    /// Sanitized output policy version.
    pub sanitizer_version: &'a str,
    /// Rasterizer version or the literal value `none` for SVG only output.
    pub rasterizer_version: &'a str,
}

/// Derives a stable SHA 256 key from every rendering input.
pub fn derive_cache_key(request: &RenderRequest, context: CacheKeyContext<'_>) -> String {
    let mut digest = Sha256::new();
    add_field(&mut digest, b"codex-latex-render");
    add_field(&mut digest, &RENDER_CORE_VERSION.to_be_bytes());
    add_field(&mut digest, &context.protocol_version.to_be_bytes());
    add_field(&mut digest, context.renderer_version.as_bytes());
    add_field(&mut digest, context.macro_policy_version.as_bytes());
    add_field(&mut digest, context.sanitizer_version.as_bytes());
    add_field(&mut digest, context.rasterizer_version.as_bytes());
    add_field(&mut digest, request.source.as_bytes());
    add_field(&mut digest, &[u8::from(request.display_mode)]);
    add_field(&mut digest, &request.foreground.channels());
    match request.background {
        Some(background) => {
            add_field(&mut digest, &[1]);
            add_field(&mut digest, &background.channels());
        }
        None => add_field(&mut digest, &[0]),
    }
    add_field(&mut digest, &request.scale.to_bits().to_be_bytes());
    add_field(&mut digest, &request.max_width_px.to_be_bytes());
    let mut key = format!("v{RENDER_CORE_VERSION}:");
    for byte in digest.finalize() {
        write!(key, "{byte:02x}").expect("writing to a string cannot fail");
    }
    key
}

fn add_field(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}
