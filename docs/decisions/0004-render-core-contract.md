# ADR 0004: Keep the render contract backend neutral

- Status: Accepted
- Date: 2026-08-27

## Context

Codex must not depend directly on MathJax, Node.js, SVG parsing, or a particular
terminal rasterizer. The contract also needs dynamic dispatch so an integration can
select a backend at runtime without coupling to its future implementation.

Cache invalidation must include every presentation input and implementation
version. TeX whitespace can be semantic, so normalizing source by trimming or
rewriting line endings would risk returning the wrong render.

## Decision

Define requests, results, limits, source-free errors, an object-safe asynchronous
renderer trait, stable SHA 256 cache keys, and a size-bounded memory cache in
`latex-render-core`. Cache source exactly as received and use length-prefixed fields
to prevent ambiguous key encodings.

Keep concurrency outside the cache. The owning render service serializes cache
access, which avoids hidden locks in the portable core contract.

## Consequences

- Codex and the standalone CLI can depend on one small contract.
- Worker and future native renderers can implement the same trait.
- Source whitespace and version changes always invalidate cached output.
- Cache accounting bounds content bytes and entry count, while allocator overhead
  remains a small bounded addition.
- Worker supervision must provide synchronization and cancellation above this
  crate.

## Rejected alternatives

- Native asynchronous trait syntax was rejected because the resulting trait would
  not support dynamic dispatch on the minimum Rust version.
- Source trimming was rejected because it can change TeX meaning.
- A global concurrent cache was rejected because it would impose a runtime and
  locking policy on all clients.
