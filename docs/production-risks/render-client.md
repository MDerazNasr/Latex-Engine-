# Render client production risks

This note records the required failure prediction for the supervised MathJax worker
client.

## 1. Missing, malformed, mismatched, or oversized protocol data

Trigger: a worker omits handshake fields, reports an incompatible version, returns a
wrong request ID, emits inconsistent success fields, sends invalid dimensions, or
produces a line beyond the configured byte limit.

Impact: a response could be attributed to the wrong equation, memory use could grow,
or unsafe output could reach a later renderer boundary.

Mitigation: typed structures reject unknown or missing fields, identity and limits
are checked at startup, line framing is bounded before JSON decoding, correlations
must match exactly, and invalid success output recycles the worker.

Test coverage: protocol and line-reader tests cover malformed frames, CRLF, byte
limits, capability and version mismatches, wrong IDs, partial alpha, control-bearing
errors, and invalid success dimensions. The real MathJax worker passes the same
supervised boundary.

## 2. Queue, cache, and cancellation races expose stale work

Trigger: many completed expressions arrive together, two callers miss the same cache
entry, a caller disappears while rendering, or shutdown begins with queued work.

Impact: the TUI could block, a stale result could be cached or displayed, or shutdown
could wait for irrelevant work.

Mitigation: submission uses a capacity 32 nonblocking queue, cache access is
serialized, every request has an independent response channel, closed queued replies
are skipped, abandoned active replies are discarded, and shutdown preempts active
and pending work.

Test coverage: integration tests force a full queue, cancel an active caller, confirm
only the current result enters cache, and stop a worker while a render is hung. The
future Codex adapter must still apply thread, turn, equation, and generation tokens
before display.

## 3. Worker exits, timeouts, or cleanup errors escape asynchronous handling

Trigger: Node fails to start, crashes during a request, hangs in synchronous MathJax
work, returns a corrupt stream, or does not exit after stdin closes.

Impact: rendering could enter a crash loop, leak a child process, delay CLI exit, or
surface a task cancellation as an agent failure.

Mitigation: startup and render deadlines recycle the process, one restart is allowed
per interval, immediate later requests receive a backoff error, stderr cannot block,
and explicit shutdown closes, kills, waits, and reports a reap failure.

Test coverage: process tests cover missing runtimes, invalid configuration, crashes,
one successful restart, incompatible handshakes, timeouts, restart backoff, graceful
shutdown, active-work shutdown, and the built MathJax process end to end. Fake process
tests use the production startup budget under parallel CI load, and unique marker
files have scope-based cleanup even when an assertion panics.
