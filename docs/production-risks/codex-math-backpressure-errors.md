# Codex math backpressure error production risks

Feature commit: `a54b73cd9d`

1. A null controller, invalid configuration, malformed request, or unexpected renderer error could be mistaken for temporary queue pressure and retried indefinitely. Only the supervisor's explicit queue full and stopped variants retain typed controller errors, while every other renderer failure remains a source safe terminal error.
2. Queue capacity can change between an App scheduling decision and controller submission. The nonblocking supervisor send remains the authority, and its exact synchronous result determines whether work is deferred or failed without blocking the TUI.
3. Shutdown can close the daemon queue while asynchronous history activation is still selecting work. Stopped remains distinct from queue full, the controller rejects new work immediately, and lifecycle invalidation prevents any abandoned completion from publishing terminal bytes.

Focused verification: all five controller tests passed, including typed queue pressure, typed shutdown, invalid preparation, daemon completion, raster failure, and late completion suppression. The required Codex fixer and scoped formatter completed without code findings.
