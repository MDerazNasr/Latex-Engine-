# Terminal presenter production risks

This note records the required failure prediction for the generation checked bridge
between layout, rasterization, and image placement.

## 1. Missing or malformed geometry, PNG bytes, or source data reaches publication

Trigger: a caller fabricates a result with zero or incorrect dimensions, supplies an
empty or malformed PNG, selects a source for the wrong backend, or points at a
missing local file.

Impact: the terminal could reserve one rectangle while decoding another image, emit
an invalid protocol command, or leave an old placement active without explanation.

Mitigation: validated layout and fitted raster constructors reject invalid geometry.
The correlated result requires exact canvas dimensions, a PNG signature, and the
global encoded byte bound. Publication requires the expected source variant and
byte equality; local comparison reads no more than one byte past the bounded result.

Test coverage: integration tests cover canvas mapping, wrong dimensions, malformed
PNG bytes, source mismatch, raster limits, text fallback, and placement state.

## 2. Resize, cancellation, or backend selection races an older completion

Trigger: a native raster job finishes after another job begins, explicit fallback is
selected, or terminal capability changes.

Impact: an older equation could overwrite newer prose, use invalid cell dimensions,
or send a protocol to the wrong terminal.

Mitigation: each begin, fallback, and backend change advances a checked generation.
Every job also captures its backend. Publication compares both before reading source
data or invoking placement and returns a side effect free stale result on mismatch.

Test coverage: tests finish jobs out of order, complete work after fallback, and
complete work after a backend change. All stale paths emit no bytes and create no
active image.

## 3. Raster tasks, local file reads, placement encoding, or terminal writes fail

Trigger: the blocking raster task panics or is cancelled, native rendering returns an
error, a local file disappears, protocol encoding fails, or the terminal write is
partial.

Impact: the old image could remain while new source text is hidden, or a partially
written escape sequence could corrupt the visible transcript.

Mitigation: raster and placement failures are typed and leave presenter state
unchanged. The Codex adapter must handle both task join and inner results, retain the
canonical source until command bytes are written successfully, and invoke fallback
cleanup on any failure. Local file ownership and atomic terminal writes remain
explicit Phase 2 integration work.

Test coverage: raster limit and source validation failures prove state does not
change. Phase 3 event loop tests will inject task join, cancellation, and partial
writer failures at the integration boundary that owns them.

