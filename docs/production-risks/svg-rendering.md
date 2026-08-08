# SVG rendering production risks

This note records the required failure prediction for SVG validation and native PNG
rasterization.

## 1. Missing, malformed, or active SVG reaches the renderer

Trigger: the worker returns null, empty, malformed, oversized, entity-bearing, or
external-resource SVG, or adds a previously unseen MathJax attribute or element.

Impact: unsafe content could access local resources, parser behavior could diverge,
or legitimate equations could unexpectedly fall back to source text.

Mitigation: the client accepts only UTF 8 XML within fixed byte and structural
limits, rejects every unlisted event, element, attribute, and value, removes source
metadata, and caches only rewritten `SanitizedSvg` bytes.

Test coverage: integration tests reject scripts, external links, entities, CDATA,
processing instructions, controls, malformed geometry, unknown metadata, missing
attributes, and all configured limits. All 25 real MathJax corpus expressions pass
the same client boundary.

## 2. Policy, cache, or presentation generations race

Trigger: sanitizer or rasterizer behavior changes while an older cache key remains
valid, or a delayed raster result is displayed after terminal dimensions, theme, or
conversation state changes.

Impact: stale colors or dimensions could be shown for the wrong presentation state,
or output accepted under an older policy could be reused.

Mitigation: cache identity includes independent sanitizer and rasterizer versions,
`SanitizedSvg` is immutable outside its crate, and the future Codex adapter must
check thread, turn, equation, and generation tokens immediately before placement.

Test coverage: cache key golden tests cover policy versions, deterministic raster
tests compare repeated bytes, and client cancellation tests prevent abandoned work
from entering the cache. Phase 3 integration tests will force stale presentation
generations.

## 3. Raster work blocks, exhausts memory, or fails asynchronously

Trigger: valid but complex SVG consumes excessive CPU, requested dimensions overflow
an allocation, PNG encoding grows unexpectedly, or a blocking raster task panics or
is abandoned during shutdown.

Impact: the Codex event loop could freeze, memory could spike, or a render future
could disappear without restoring source fallback.

Mitigation: validation bounds bytes, elements, depth, attributes, path segments, and
transforms; rasterization bounds width, height, raw RGBA bytes, and encoded PNG bytes.
Asynchronous callers must use a bounded blocking task and map join, panic, timeout,
and cancellation outcomes to source-visible fallback.

Test coverage: tests exercise every current structure and allocation limit, decode
the resulting PNG, and verify deterministic output. Phase 4 adds fuzzing, timeout,
and blocking-pool saturation coverage because parser and raster CPU cost still needs
adversarial measurement.
