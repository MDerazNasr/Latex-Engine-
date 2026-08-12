# ADR 0009: Reserve measured cells around an aspect preserving canvas

- Status: Accepted
- Date: 2026-08-27

## Context

The Kitty graphics protocol scales an image when both placement columns and rows are
specified. Passing an equation PNG directly into a rounded cell rectangle can
therefore stretch the glyphs. Supplying only one placement dimension preserves aspect
ratio, but terminal-side rounding can disagree with the rows already reserved by a
source-backed transcript.

Inline math also needs a predictable text baseline, while display math needs viewport
caps and stable centering across resize reflow.

## Decision

Require measured terminal cells and window pixels before graphical layout. Derive
whole cell pixels from those measurements and fail to source presentation when a cell
would have no pixels.

Reserve an exact cell rectangle and rasterize a transparent canvas with the same
pixel aspect. Uniformly scale equation content inside that canvas without upscaling.
Keep non-display math inline only when its baseline-aware, one-row layout fits the
remaining prose columns; otherwise promote it to a centered block. Always center and
cap display math by viewport width percentage and row limit.

Keep this layout function pure. It returns placement cells, canvas pixels, content
rectangle, scale, baseline, and horizontal column but emits no control sequences.

## Consequences

- Kitty direct and iTerm2 Kitty-compatible placement can specify both cell dimensions
  without distorting equation content.
- Transcript measurement and terminal placement agree on the reserved row count.
- A resize can recompute all layout data without touching stored source.
- Inline baseline alignment uses a fixed cell percentage until Codex exposes an exact
  font baseline measurement.
- Terminals without pixel geometry use source fallback.

## Rejected alternatives

- Sending an unpadded PNG with both cell dimensions was rejected because cell rounding
  can change its aspect ratio.
- Sending only columns or rows was rejected because the terminal may reserve a
  different rectangle than the transcript layout.
- Guessing common cell pixels was rejected because font size, display scale, and
  terminal zoom vary at runtime.

The placement behavior follows the official
[Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/).
