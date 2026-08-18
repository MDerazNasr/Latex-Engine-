# ADR 0016: Enable Sixel only inside an owned TUI redraw path

Status: Accepted

Date: 2026-08-27

## Context

Sixel encoding is technically feasible and the current OpenAI Codex TUI already has
a deterministic in-process RGB332 implementation for terminal pets. Sixel placement,
however, does not provide Kitty image identifiers and targeted deletion. Transparent
or smaller replacement output can leave old pixels unless the host repaints the
entire affected cell region. Cursor and scrolling behavior also depends on terminal
Sixel modes.

## Decision

Do not expose or automatically detect an independent Sixel backend in Phase 2. Keep
Sixel-only terminals on canonical source text. In Phase 3, reuse the existing Codex
encoder inside the Ratatui redraw path that owns every affected cell. Add this
project's input, output byte, cancellation, generation, and resize limits before
enabling automatic selection.

The completed investigation and primary evidence are recorded in
`docs/investigations/sixel-2026-08-27.md`.

## Consequences

- Kitty and iTerm2 keep the verified targeted lifecycle guarantees.
- Sixel users receive correct source instead of a partially managed image.
- Phase 3 avoids a second encoder and integrates with code already shipped by Codex.
- Sixel support remains required before stable cross-platform completion, but it no
  longer blocks the independent Phase 2 crate.

## Rejected alternatives

- Porting only the encoder was rejected because encoding alone does not solve stale
  pixel deletion, scroll, or reflow.
- Clearing with opaque background pixels was rejected because it breaks transparent
  theme blending.
- Advertising Sixel from environment markers before lifecycle tests was rejected
  because capability detection must imply a backend that actually meets its contract.

