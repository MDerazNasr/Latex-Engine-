# Codex raster preparation production risks

Feature commit: `d3360847c3`

1. Empty, malformed, oversized, or dimensionally deceptive PNG data could exhaust memory or create an invalid terminal canvas. Independent byte, dimension, decoded allocation, content rectangle, and canvas allocation limits fail to source before publication.
2. Resize or generation races could publish an asset fitted for obsolete cell geometry. Prepared files carry exact canvas metadata and remain unpublished until the later immutable identity check accepts the matching generation.
3. Decode, resize, encode, flush, persistence, or cleanup failures could leak files or leave an unhandled asynchronous preparation path. Temporary file ownership removes partial outputs, the store removes retained assets on drop, and every operation returns a typed error for source fallback.

Focused verification: six raster tests passed for exact transparent canvases, resize output, malformed and oversized PNG rejection, invalid geometry rejection, allocation limits, and owner cleanup. The required Codex fixer and scoped formatter also passed.
