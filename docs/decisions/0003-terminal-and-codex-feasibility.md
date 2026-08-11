# ADR 0003: Reuse Codex terminal images at the transcript layer

- Status: Accepted
- Date: 2026-08-26

## Context

The renderer needs terminal image transport plus a narrow integration with the
installed Codex CLI. Codex 0.149.1 already implements Kitty direct transfer,
iTerm2 3.6 Kitty local-file transfer, Sixel, terminal detection, deletion, cursor
preservation, and synchronized drawing for its pet UI. Assistant messages already
consolidate into source-backed history cells that reflow on resize.

The independent Phase 0 spike generated a quadratic equation with the pinned
MathJax worker and displayed it successfully in installed iTerm2 3.6.10. Visual
inspection confirmed rendering and normal cleanup. Automated process tests prove
direct transfer, local-file transfer, replacement on resize, targeted deletion,
cursor restoration, alternate-screen restoration, and redirected source fallback.

The same equation also rendered successfully through direct PNG transfer in Kitty
0.47.0. A post-run iTerm2 capture confirmed targeted deletion and alternate-screen
restoration. Kitty lifecycle cleanup remains covered by process-level command tests
because its validation window deliberately preserved the last child screen.

## Decision

Proceed with the project. Reuse and generalize the Codex terminal image machinery
instead of creating a parallel TUI implementation. Integrate math only in the
assistant transcript presentation layer, preserving raw Markdown everywhere else.

Use the independent `latex-terminal` crate as a conformance and development
boundary. During upstream integration, extract the equivalent Codex pet code into
a shared private module and adapt this repository's renderer through a narrow TUI
adapter.

Treat both verified transports as the Phase 0 reference backends. Other terminals
remain source fallback until they pass the compatibility matrix.

## Consequences

- The Codex change can reuse tested terminal behavior and avoid a new public API.
- Node and MathJax remain an optional external worker rather than default Codex
  dependencies until upstream packaging is agreed.
- Transcript placement metadata and stale-result correlation become the central
  TUI design work.
- Direct placement is proven for the spike; Kitty Unicode placeholders remain a
  Phase 2 experiment for scrollback-native positioning.
- The project records a Phase 0 go decision backed by two real macOS terminal
  validations.
