# Terminal math theme production risks

This note records the required failure prediction for terminal math theme resolution.

## 1. A missing or incorrect appearance hint chooses unreadable colors

Trigger: auto mode receives no hint, the host reports the opposite appearance, or a
custom palette differs substantially from the reference light and dark backgrounds.

Impact: equation glyphs may have low contrast even though rasterization succeeds.

Mitigation: a missing hint has a documented dark default, explicit light and dark
modes override it, backgrounds remain transparent, and both reference palettes
exceed the high contrast threshold. Phase 3 exposes configuration so users can
override a bad host hint.

Test coverage: tests cover null, true, and false hints, both explicit overrides,
exact colors, transparency, stable names, and reference contrast ratios.

## 2. Theme change races a raster task from the prior appearance

Trigger: the terminal or Codex theme changes while MathJax or fitted rasterization is
running.

Impact: a light glyph image can appear on a light background or overwrite a newer
theme generation.

Mitigation: resolved colors already participate in render cache identity. The host
must begin a new presenter generation whenever the resolved value changes, and the
presenter discards older completions before terminal output.

Test coverage: render cache tests cover color invalidation, theme tests cover stable
resolved values, and presenter tests cover out of order generation completion.

## 3. Render, raster, or theme refresh work fails asynchronously

Trigger: the worker rejects a colored render, the raster task fails, or the host
drops an appearance change future during shutdown.

Impact: source could be hidden behind a stale image or an unhandled task error could
leave the previous theme active.

Mitigation: theme resolution itself is synchronous and total. Worker and raster
errors retain canonical source, while cancellation or refresh must advance presenter
generation and use its cleanup bytes before suppressing any source display.

Test coverage: existing worker, raster limit, fallback, cancellation, and stale
completion tests exercise these boundaries. Phase 3 event tests will inject a theme
change during an active render.

