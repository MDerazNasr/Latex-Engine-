# Codex math context rerender production risks

Feature commit: `a703691ddf`

1. Null terminal measurements, zero pixel geometry, unsupported image protocols, malformed cell state, or exhausted generations could leave an outdated image visible. Terminal sampling fails closed, successor identities require valid geometry and an exact generation advance, and unsuccessful refreshes restore the original source presentation.
2. Resize, palette discovery, message consolidation, and renderer completion can race while the transcript is rebuilding. Each cell swaps to the exact successor identity before submission, old completions cannot find or mutate the replacement generation, and the transcript reflows atomically from source backed cells.
3. Renderer queue saturation, controller shutdown, asset preparation failure, or an unhandled asynchronous completion could strand a cell in pending state. Submission and preparation failures explicitly fail the matching generation to source, controller events remain nonblocking, and shutdown invalidates the lifecycle before stopping the supervised process.

Focused verification: four App runtime tests, ten resize reflow tests, and six lifecycle tests passed. The terminal change case verifies generation replacement, source fallback, scheduled reflow, and rejection of the old completion. The required Codex fixer and scoped formatter completed without code findings.
