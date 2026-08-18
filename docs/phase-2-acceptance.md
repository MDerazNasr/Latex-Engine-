# Phase 2 terminal presentation acceptance

Date: 2026-08-27

Branch: `feature/phase-2-terminal-presentation`

## Result

Phase 2 passes its exit criteria. Representative equations render through the real
worker and production terminal components in Kitty and iTerm2 without stale images.
Redirected output, multiplexers, unsupported terminals, malformed arguments, and
worker failure preserve clean source text without image control sequences.

## Automated evidence

- All 150 ordinary Rust workspace tests pass.
- The ignored real MathJax integration and reviewed 50 snapshot cases pass.
- All 14 TypeScript worker tests pass.
- Rust formatting, strict Clippy, warning-free documentation, TypeScript formatting,
  and TypeScript type checking pass.
- Process tests cover direct and local-file transport, two geometry generations,
  targeted deletion, alternate screen restoration, local asset cleanup, redirected
  fallback, malformed arguments, and worker failure.
- Real worker protocol captures complete successfully for forced Kitty and iTerm2.

## Manual evidence

Kitty 0.47.0 displayed the first equation, replaced it at a second measured geometry,
left only the current placement visible, deleted the placement on exit, restored the
cursor, and returned to the original screen.

iTerm2 3.6.10 displayed the equation through Kitty local-file transfer and returned
to a clean prompt after targeted deletion and alternate screen restoration.

Screenshots were inspected from temporary paths and were not committed because they
contain machine-local terminal chrome and notifications.

## Performance evidence

The release benchmark passed every existing Phase 1 budget:

| Path | Observed | Budget |
|---|---:|---:|
| First render | 226.412 ms | 1500 ms |
| Uncached simple p95 | 1.404 ms | 150 ms |
| Complex p95 | 3.538 ms | 500 ms |
| Cached p95 | 0.002 ms | 10 ms |
| Segmenter delta p95 | 0.001 ms | 5 ms |

## Deferred integration work

- Codex owns terminal queries, transcript cells, asynchronous raster tasks, and
  synchronized redraw in Phase 3.
- Sixel remains source fallback until the Codex-owned redraw path adapts its existing
  deterministic encoder and passes replacement and partial-write tests.
- Cross-platform, SSH, multiplexer, accessibility, packaging, and security campaigns
  remain Phase 4 work.
