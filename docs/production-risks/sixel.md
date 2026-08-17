# Sixel integration production risks

This note records the required failure prediction for the deferred Sixel backend.

## 1. Null, malformed, or unbounded pixels enter the encoder

Trigger: dimensions are zero, RGBA length does not match width and height, arithmetic
overflows, palette planes expand output far beyond PNG size, or no output limit is
configured.

Impact: the encoder can consume excessive CPU and memory or emit a partial DCS
sequence that disrupts the terminal.

Required mitigation: validate exact bounded dimensions and pixel length before work,
use checked indexing, enforce an encoded byte limit during every append, and return
source fallback before writing any partial sequence.

Required test coverage: zero, overflow, mismatched length, transparent image, maximum
palette, maximum run, and output limit cases must pass before the backend is enabled.

## 2. Resize or redraw races leave stale Sixel pixels

Trigger: a smaller or transparent frame replaces an older image, terminal geometry
changes during encoding, or a stale completion arrives after transcript reflow.

Impact: old glyph pixels remain visible over prose or the equation moves independently
of its source cell.

Required mitigation: encode only for a captured presenter generation, publish inside
Codex's synchronized full cell redraw, repaint the complete prior rectangle before a
new DCS sequence, and discard stale output.

Required test coverage: Phase 3 replay tests must cover smaller replacement, source
toggle, resize, scroll, cancellation, and out of order completion.

## 3. Encoding or terminal output remains unhandled asynchronously

Trigger: CPU encoding monopolizes the UI task, cancellation is dropped, output hits
its cap midway, a terminal write is partial, or shutdown occurs during DCS output.

Impact: input freezes, half an escape sequence reaches the terminal, or source is
hidden without a valid image.

Required mitigation: run encoding off the UI task with cancellation checkpoints,
build one bounded buffer before publication, handle both task join and inner errors,
and retain source until the full write succeeds. Cleanup must occur through the next
owned TUI redraw.

Required test coverage: time budget, cancellation, join failure, output cap, partial
writer, shutdown, and source preservation tests are mandatory in Phase 3.

