# Codex live math event production risks

Feature commit: `2858c17ffd`

1. Null terminal pixels, fractional cell dimensions, unsupported image protocols, malformed renderer output, or a message with no valid equations could produce blank placeholders. Terminal sampling and daemon decoding fail closed, and every rejected request or completion leaves the original source visible.
2. Consolidation, message replacement, resize, event delivery, and shutdown can race with renderer or raster tasks. Each AppEvent retains the complete immutable identity, the runtime clock and source cell both validate it, shutdown closes the completion gate, and published images are deleted before terminal teardown.
3. Renderer discovery failure, queue saturation, closed AppEvent receivers, blocking raster failure, or an unhandled child process path could strand work. Discovery remains optional, bounded controller submissions fail to source, event sends are nonblocking and logged by variant only, and explicit shutdown aborts tasks and reaps the supervised daemon.

Focused verification: all 66 math subsystem tests passed, with both parallel fake daemon timing retries also passing alone. Three live AppEvent tests, two terminal measurement tests, four agent math tests, seven existing agent markdown tests, four layout tests, two consolidation replay tests, and the session log privacy test passed. The required Codex fixer and scoped formatter completed without code warnings.
