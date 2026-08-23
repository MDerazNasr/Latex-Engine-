# Codex math controller production risks

Feature commit: `2b8f55466b`

1. Empty raster jobs, duplicate equation indices, duplicate or zero image identifiers, oversized aggregate PNG data, or a stopped controller could enter an ambiguous state. Synchronous validation rejects each case before spawning work.
2. Renderer, raster, resize, replacement, and shutdown races could enqueue a completion after invalidation. Every event retains its immutable identity, and a synchronized completion gate prevents event sends after controller shutdown begins.
3. Daemon cancellation, blocking task failure, raster failure, partial asset cleanup, queue saturation, or child shutdown could remain unhandled. Bounded supervisor and preparation queues return typed failures, partial assets receive immediate best effort cleanup plus owner cleanup, tasks are aborted, and the daemon is reaped through the supervised shutdown path.

Focused verification: four controller integration tests passed with a compiled fake daemon, covering correlated render events, exact prepared assets, invalid and stopped requests, and late completion suppression during shutdown. All 45 math adapter tests also passed. The required Codex fixer and scoped formatter passed without code warnings.
