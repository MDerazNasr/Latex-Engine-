# Codex app safe completion production risks

Feature commit: `f5bd48cfc8`

1. A malformed or null renderer result could leak low level process details into application events or leave the source cell pending forever. Controller tasks now log the internal error and emit an identity with no payload so the cell deterministically returns to source text.
2. Raster completion can race with message replacement or shutdown and publish assets for an obsolete cell. Every completion retains its immutable render identity, and the synchronized completion gate suppresses all sends after shutdown begins.
3. A panicking blocking task, failed raster conversion, closed event receiver, or unhandled asynchronous daemon error could strand temporary files or tasks. Join failures become payload free completions, the asset owner removes stored files on drop, event sends are nonblocking, and controller shutdown aborts and joins every task.

Focused verification: five controller integration tests and five cell state tests passed, covering correlated success, payload free raster failure, invalid submissions, exact asset publication, source fallback, and late completion suppression. The required Codex fixer and scoped formatter completed without code warnings.
