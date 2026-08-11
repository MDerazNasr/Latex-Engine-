#![doc = "Cache key integration tests."]

mod common;

use latex_render_core::{CacheKeyContext, Rgba, derive_cache_key};

use common::request;

const CONTEXT: CacheKeyContext<'static> = CacheKeyContext {
    protocol_version: 1,
    renderer_version: "mathjax-0.1.0",
    macro_policy_version: "base-ams-1",
    sanitizer_version: "svg-allowlist-1",
    rasterizer_version: "none",
};

#[test]
fn cache_key_is_stable_for_identical_inputs() {
    let request = request("x = \\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}");

    let first = derive_cache_key(&request, CONTEXT);
    let second = derive_cache_key(&request, CONTEXT);

    assert_eq!(first, second);
    assert_eq!(
        first,
        "v1:ed8c59998985a501f1f91aba3bb13a124bc4217e0b2659dcf497f070bd90704d"
    );
}

#[test]
fn every_rendering_input_invalidates_the_key() {
    let original = request("x^2");
    let baseline = derive_cache_key(&original, CONTEXT);
    let mut variants = Vec::new();

    let mut value = original.clone();
    value.source.push(' ');
    variants.push(derive_cache_key(&value, CONTEXT));

    let mut value = original.clone();
    value.display_mode = false;
    variants.push(derive_cache_key(&value, CONTEXT));

    let mut value = original.clone();
    value.foreground = Rgba::opaque(1, 2, 3);
    variants.push(derive_cache_key(&value, CONTEXT));

    let mut value = original.clone();
    value.background = Some(Rgba::opaque(4, 5, 6));
    variants.push(derive_cache_key(&value, CONTEXT));

    let mut value = original.clone();
    value.scale = 1.5;
    variants.push(derive_cache_key(&value, CONTEXT));

    let mut value = original;
    value.max_width_px = 800;
    variants.push(derive_cache_key(&value, CONTEXT));

    variants.push(derive_cache_key(
        &request("x^2"),
        CacheKeyContext {
            protocol_version: 2,
            ..CONTEXT
        },
    ));
    variants.push(derive_cache_key(
        &request("x^2"),
        CacheKeyContext {
            renderer_version: "mathjax-0.2.0",
            ..CONTEXT
        },
    ));
    variants.push(derive_cache_key(
        &request("x^2"),
        CacheKeyContext {
            macro_policy_version: "base-ams-2",
            ..CONTEXT
        },
    ));
    variants.push(derive_cache_key(
        &request("x^2"),
        CacheKeyContext {
            sanitizer_version: "svg-allowlist-2",
            ..CONTEXT
        },
    ));
    variants.push(derive_cache_key(
        &request("x^2"),
        CacheKeyContext {
            rasterizer_version: "resvg-1",
            ..CONTEXT
        },
    ));

    assert!(variants.iter().all(|key| key != &baseline));
}

#[test]
fn source_is_not_trimmed_or_rewritten() {
    let plain = derive_cache_key(&request("x"), CONTEXT);
    let spaced = derive_cache_key(&request(" x "), CONTEXT);
    let line_ending = derive_cache_key(&request("x\r\ny"), CONTEXT);

    assert_ne!(plain, spaced);
    assert_ne!(plain, line_ending);
}
