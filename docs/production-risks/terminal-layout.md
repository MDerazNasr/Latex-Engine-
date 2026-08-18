# Terminal layout production risks

This note records the required failure prediction for terminal math cell layout.

## 1. Null or incoherent terminal and equation geometry

Trigger: the terminal reports zero columns, rows, width, or height; window pixels are
smaller than the cell grid; or a render reports zero dimensions or an invalid
baseline.

Impact: division by zero, an empty placement, or invalid cursor coordinates could
corrupt transcript layout.

Mitigation: validated constructors reject missing cell pixels, zero equation sizes,
nonfinite baselines, and baselines outside the image. Placement uses nonzero integer
types after validation.

Test coverage: layout tests cover every zero terminal dimension, insufficient pixel
measurement, zero equation dimension, negative, nonfinite, and oversized baseline,
and invalid policy value.

## 2. Resize or policy races display stale placement geometry

Trigger: columns, rows, window pixels, current prose column, theme, or limits change
while an older render or raster task is still running.

Impact: an equation could overlap prose, remain off center, or overwrite a newer
placement.

Mitigation: layout is a pure function of complete measured inputs and carries exact
cell and pixel outputs. The Phase 3 adapter must include terminal geometry, theme,
policy, thread, turn, and equation identity in its generation check before display.

Test coverage: resize tests prove narrower measurements reduce scale and placement
width. Phase 3 event tests will reject every stale generation dimension.

## 3. Raster or terminal work fails after cells are reserved

Trigger: the blocking raster task fails, an image backend rejects its source, or the
terminal changes size between transcript layout and synchronized placement.

Impact: reserved blank rows or a stale image could remain while source is hidden.

Mitigation: this layer has no side effects and never suppresses canonical source.
Callers must publish an image only after raster success, delete the prior placement
before replacement, and fall back to source on any generation or backend mismatch.

Test coverage: existing placement tests cover source mismatch, resize replacement,
clear, and identical redraw. The next fitted-raster feature verifies the exact canvas
contract before the combined presenter is built.
