# Daemon message renderer production risks

## 1. Empty, malformed, or adversarial math exhausts rendering resources

- Trigger: A bounded message contains too many equations, one oversized expression,
  invalid TeX, unsafe SVG, extreme dimensions, or PNG output above its limits.
- Impact: Rendering could allocate excessive memory, suppress unrelated equations,
  or return unsafe terminal assets.
- Mitigation: Protocol validation bounds the message, the renderer rejects more than
  32 equations before work, render core validates every request and result, the SVG
  allowlist runs before native rasterization, and per-image plus aggregate PNG caps
  convert only affected equations to source fallback.
- Test coverage: Tests cover the equation limit, invalid TeX continuation, unsafe SVG
  continuation, decodable PNG output, and aggregate capacity stopping later work.

## 2. Source policy or state transitions select the wrong byte range

- Trigger: UTF 8 text, code spans, prices, delimiter policies, or a future concurrent
  request path changes segmentation or outcome order.
- Impact: Codex could replace prose or code with the wrong image, or attach a result
  to a stale message generation.
- Mitigation: The independent lossless segmenter owns complete delimiter byte spans,
  outcomes retain source order, and version 1 renders one message serially. Codex must
  still match request ID, message generation, and source length before presentation.
- Test coverage: Tests verify exact spans, display mode, code exclusion, price
  exclusion, smart and disabled dollar modes, and continued ordered rendering.

## 3. Async worker, blocking raster, or task join failure is not surfaced

- Trigger: The supervised worker exits, times out, fills its queue, or the blocking
  sanitizer and raster task panics or outlives cancellation.
- Impact: A request could stall, consume blocking capacity, or lose its readable
  fallback outcome.
- Mitigation: Every renderer error maps to a stable per-equation code, blocking task
  join failure maps to `render_failed`, and later equations continue. The daemon loop
  must own EOF shutdown and Codex must cancel or kill the daemon on generation loss.
- Test coverage: Unit tests cover renderer failure and unsafe raster input. Real
  worker exit, timeout, process cancellation, and shutdown remain required process
  tests for the daemon loop.

