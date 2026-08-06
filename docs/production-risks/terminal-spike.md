# Terminal spike production risks

This note records the required failure prediction after the Phase 0 terminal and
Codex integration spike.

## 1. Missing capability data or redirected output

Null-equivalent environment values, misleading terminal names, multiplexers, and
redirected standard output can select an unsafe backend and leak control sequences
into logs or pipes.

Current control: detection is deterministic, redirected output always selects
text, tmux and Zellij select text, and malformed iTerm2 versions fail closed.

Production control: keep forced backends explicit, add bounded active probes only
when requested, and test the supported terminal matrix through pseudo terminals
and real applications.

## 2. Resize and asynchronous result races

A render started for an old width, theme, thread, or message can finish after a
newer request and overwrite the current placement. Reusing an image ID too early
can also delete an unrelated placement.

Current control: `ImageRenderState` serializes transitions through mutable access,
does nothing for identical draws, and deletes the prior image before replacement.

Production control: assign session-scoped IDs, attach generation tokens to every
request, discard stale completions in the Codex app event handler, and keep the
pending queue bounded.

## 3. Unhandled shutdown or terminal write failure

An asynchronous task failure, broken output stream, panic, interrupt, or forced
termination can leave a placement, hidden cursor, or alternate screen behind.
Process termination that skips destructors cannot be repaired by Rust cleanup
alone.

Current control: the spike restores image, cursor, and alternate-screen state from
its drop guard on normal completion, returned errors, and unwinding panics.

Production control: centralize terminal ownership in Codex, clear placements on
thread changes and alternate-screen transitions, handle normal termination signals,
and preserve source when any write or render task fails. `SIGKILL` remains outside
the process recovery model and must rely on terminal reset behavior.

## Additional constraints

- A local-file image must remain present until the terminal consumes it.
- tmux and Zellij stay on source fallback until pane-local behavior is proven.
- The Quick Look rasterizer is a macOS-only spike tool and cannot enter production.
- A second real Kitty or WezTerm visual validation remains required before beta.
