# ADR 0015: Resolve math color while keeping the canvas transparent

Status: Accepted

Date: 2026-08-27

## Context

Equation glyphs must remain readable in light and dark terminal themes without
painting a rectangular background over the transcript. The host TUI may know its
appearance, but terminal environment variables do not provide a reliable universal
theme signal. Rendering also needs stable colors for snapshots and cache identity.

## Decision

Expose `auto`, `light`, and `dark` theme modes. Auto follows an optional dark
background hint supplied by the host TUI and defaults to dark when the hint is
missing. Dark uses foreground `#e6edf3`; light uses `#111827`. Both use a transparent
background.

Return a concrete resolved mode and colors as a small immutable value. The
integration includes this value in the existing render request and cache key and
begins a new presentation generation when it changes.

## Consequences

- Theme resolution has no terminal probes or rendering side effects.
- Default output remains compatible with the reviewed dark snapshots.
- Light and dark reference pairs exceed a 7 to 1 contrast ratio.
- Hosts with custom palettes can add an explicit color extension later without
  changing current mode semantics.

## Rejected alternatives

- Rendering an opaque background was rejected because it creates visible boxes and
  does not blend with transcript selection or custom themes.
- Guessing from `COLORFGBG` was rejected because it is inconsistent and often stale.
- Sampling terminal pixels was rejected because terminal applications do not expose
  that operation portably.

