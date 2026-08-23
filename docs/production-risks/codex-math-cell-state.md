# Codex math cell state production risks

Feature commit: `5a5614473a`

1. Empty presentations, invalid UTF-8 spans, zero or overflowing image identifier ranges, missing PNG bytes, or mismatched canvas geometry could create incomplete image metadata. Plan construction and controller validation reject each condition before the cell can leave readable source.
2. Message replacement, equation reordering, width changes, and cross-generation asset races could attach pixels to the wrong equation. Every raster job and prepared asset carries the complete message identity plus equation ordinal and byte span, and readiness requires an exact match for every planned asset.
3. Renderer failure, raster failure, poisoned state locking, missing assets, or an asynchronous partial result could leave a cell stuck or unreadable. State transitions fail closed, ready layouts are all or nothing, and pending, failed, stale, and wrong-width states expose the original source layout.

Focused verification: five cell state tests and four controller integration tests passed for identity propagation, exact layout readiness, stale and mismatched completion rejection, wrong-width fallback, invalid identifier ranges, shutdown, and prepared raster delivery. The required Codex fixer and scoped formatter also passed without code warnings.
