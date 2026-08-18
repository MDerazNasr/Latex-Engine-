# ADR 0017: Keep terminal acceptance on the production component path

Status: Accepted

Date: 2026-08-27

## Context

Unit protocol tests can prove exact bytes but not that the worker, theme, sanitizer,
layout, fitted raster, local image store, presenter, cursor handling, and screen
cleanup compose correctly. The original Phase 0 spike displays a reviewed static PNG
and intentionally does not exercise the production renderer pipeline.

## Decision

Add the internal `latex-terminal-smoke` workspace tool instead of modifying the
working Phase 0 spike. It renders one expression with the supervised MathJax worker,
resolves theme colors, sanitizes SVG, computes layout from explicit measured cells
and pixels, creates a fitted PNG, prepares the selected transport, publishes through
generation state, and restores the alternate screen with targeted deletion.

Auto mode uses passive capability detection and exits with source before worker
startup when output is redirected or unsupported. Forced Kitty and iTerm2 modes
allow protocol capture in process tests and real terminal acceptance. An optional
second geometry reuses the rendered SVG and exercises generation checked replacement
without another worker render.

## Consequences

- Acceptance uses the same library functions intended for Codex integration.
- The tool never guesses cell pixel geometry.
- Renderer failure preserves source before terminal state is changed.
- Screen restoration remains guarded by drop even when resize publication fails.
- The tool is internal and does not add user facing behavior to `latex-render`.

## Rejected alternatives

- Replacing the Phase 0 static spike was rejected because its verified behavior must
  remain available for protocol isolation.
- Assuming a fixed cell aspect ratio was rejected because image placement would
  distort across fonts, scaling, and Retina displays.
- Rendering after entering the alternate screen was rejected because worker failure
  would unnecessarily disrupt visible terminal state.

