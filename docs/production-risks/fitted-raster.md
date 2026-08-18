# Fitted raster production risks

This note records the required failure prediction for aspect preserving terminal
rasterization.

## 1. Missing, zero, overflowing, or out of bounds geometry enters the rasterizer

Trigger: terminal measurement produces an empty content rectangle, arithmetic wraps
near an integer limit, or a caller combines geometry from different layouts.

Impact: the renderer could allocate the wrong canvas, draw outside the intended
placement, or pass invalid floating point values into the native rasterizer.

Mitigation: validate the bounded canvas first, use checked addition for content
edges, reject zero dimensions, and require the entire rectangle to lie inside the
canvas before SVG parsing or allocation.

Test coverage: fitted raster integration tests cover zero width, an edge outside the
canvas, integer overflow, allocation bounds, and a valid transparent canvas.

## 2. A resize races an older fitted raster result

Trigger: terminal geometry or theme changes while rasterization for an earlier
layout generation is running in the blocking pool.

Impact: a correctly formed PNG is placed into the wrong cell rectangle and disrupts
the transcript layout.

Mitigation: this API remains side effect free and returns dimensions with the PNG.
The presenter must compare its captured generation and dimensions with the active
reservation immediately before publication. A stale result is discarded.

Test coverage: deterministic fitted output is covered here. Presenter generation and
resize race tests belong to the terminal orchestration feature that owns publication.

## 3. Parsing, allocation, rendering, encoding, or task joining fails asynchronously

Trigger: accepted SVG still fails native parsing, memory pressure rejects a pixmap,
PNG encoding exceeds its limit, or the blocking task panics or is cancelled.

Impact: the reserved image area could remain blank or an unhandled join error could
terminate transcript rendering.

Mitigation: native failures map to stable render errors, raw and encoded output are
bounded, and no partial image is published by this synchronous API. The asynchronous
caller must handle both the inner render result and the blocking task join result,
then fall back to source text while clearing the reservation.

Test coverage: existing raster tests cover allocation and PNG limits; snapshot tests
exercise the complete worker, sanitizer, and raster path. Presenter integration tests
will cover task join errors and fallback behavior.

