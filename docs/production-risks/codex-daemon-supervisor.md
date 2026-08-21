# Codex renderer daemon supervisor production risks

## 1. Invalid configuration or queue pressure blocks the TUI

- Trigger: The executable path is empty, timeout values are zero, queue capacity is
  null or above 32, or more requests arrive while the daemon is busy.
- Impact: Rendering could consume unbounded memory, wait on the TUI thread, or turn an
  optional presentation failure into a failed Codex interaction.
- Mitigation: Configuration validation rejects unsafe limits, submission uses a
  bounded nonblocking channel, and queue saturation returns a source-safe error before
  allocating process work.
- Test coverage: Focused tests cover empty paths, zero timeouts, excessive capacity,
  an active request, one queued request, immediate saturation, and cancellation.

## 2. Restart and generation races create a crash loop or stale response

- Trigger: A daemon crashes, returns malformed JSON, or times out while later requests
  and shutdown are already queued.
- Impact: Codex could spawn processes repeatedly, attach an old result to new source,
  or delay shutdown behind a long renderer timeout.
- Mitigation: One task owns serial requests and correlation, permits only one restart
  per interval, blocks fresh starts during backoff, and lets shutdown preempt both
  active and queued work. The presentation controller must still reject stale
  immutable generations before publication.
- Test coverage: Process tests verify one crash recovery, two malformed launches for
  the first request, no third launch during backoff, queue order, active cancellation,
  and queued cancellation.

## 3. Async process failure leaks a daemon or leaves partial protocol state

- Trigger: Spawn, request write, response read, PNG decode task, timeout, pipe EOF,
  child wait, forced kill, or supervisor join fails.
- Impact: The TUI could wait indefinitely, retain a child process, parse a partial
  response, or surface source in an operational error.
- Mitigation: Request exchange and decoding share a deadline, response framing is
  bounded before parsing, errors are source free, graceful EOF shutdown is bounded,
  forced kill is reaped, and child kill on drop protects aborted tasks.
- Test coverage: Tests cover exact, oversized, and partial lines, healthy reaping,
  crash recovery, timeout and forced stop, malformed responses, preemptive shutdown,
  and a real Codex supervisor to built MathJax rendering pass.
