# Render core production risks

This note records the required failure prediction for the backend neutral render
contract and bounded memory cache.

## 1. Empty, malformed, or oversized values cross an adapter boundary

Trigger: an adapter constructs an empty request, invalid scale, zero dimensions,
non UTF 8 SVG, control-bearing text, or data beyond a configured resource limit.

Impact: malformed output could reach a terminal, resource use could exceed the
session budget, or diagnostic text could alter terminal presentation.

Mitigation: request and result validation fail closed with source-free errors,
colors have explicit alpha semantics, and shared defaults match the worker limits.

Test coverage: contract tests cover empty and multibyte source limits, unsafe
controls, nonfinite scale and baseline values, invalid dimensions, non UTF 8 SVG,
and sanitized public errors.

## 2. Concurrent cache access or stale version data returns the wrong image

Trigger: a future caller shares the cache without synchronization or omits a
renderer, policy, theme, scale, width, or rasterizer change from key context.

Impact: an equation can display another request's colors, dimensions, or stale
renderer output, and unsynchronized mutation would introduce a race.

Mitigation: the cache is deliberately owner mutable rather than internally shared,
the key hashes every specified rendering input, and adapters must serialize access
and supply version context.

Test coverage: key invalidation tests vary every input and implementation version;
cache tests verify replacement and least recently used ordering. Phase 1 service
tests will cover serialized concurrent requests.

## 3. Cancelled or abandoned asynchronous renders retain resources

Trigger: a caller drops a render future during thread changes, worker shutdown, or
terminal resize while the backend still owns process or queue resources.

Impact: pending work can leak, complete into stale state, or delay shutdown even
though the core future has been abandoned.

Mitigation: the core trait makes cancellation visible through future ownership and
defines a cancellation error, while process termination and stale-result tokens
remain mandatory responsibilities of the supervised client.

Test coverage: the trait object test proves futures can be dropped without polling.
Worker cancellation, timeout, exit, and cleanup tests are required in the next
Phase 1 feature block.
