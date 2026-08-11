# ADR 0005: Serialize MathJax work behind a supervised client

- Status: Accepted
- Date: 2026-08-27

## Context

MathJax owns process-global startup state and the Phase 0 worker handles protocol
lines sequentially. Allowing multiple Rust tasks to write directly to its streams
would interleave requests, complicate response correlation, and make process cleanup
ambiguous.

The Codex TUI must remain responsive under worker stalls, crashes, incompatible
versions, and bursts of completed equations. Dropping the client must never place
equation source in diagnostics or leave an ordinary shutdown path unbounded.

## Decision

Use one Tokio supervisor task as the sole owner of one lazily started worker. Callers
submit to a queue capped at 32 entries and receive results through one-use response
channels. The supervisor validates a strict ready handshake, processes one request at
a time, enforces startup and render deadlines, and performs at most one rate-limited
restart after a process or protocol failure.

Read stdout with an explicit byte bound before deserialization. Discard replies whose
caller has gone away, reject new work immediately when the queue is full, and require
an explicit asynchronous shutdown to close stdin, kill if needed, and reap the child.

## Consequences

- Concurrent TUI tasks cannot interleave JSONL frames or consume another request's
  response.
- One MathJax process remains the performance and memory boundary for a session.
- Burst load degrades to source through `QueueFull` instead of blocking the TUI.
- Abrupt Rust object drop requests shutdown and aborts the supervisor, while the
  explicit shutdown method provides the verified reap guarantee.
- Increasing renderer parallelism later requires a bounded pool of complete
  supervisors rather than shared access to one child.

## Rejected alternatives

- Direct process access from each render future was rejected because stream writes
  and child ownership would race.
- An unbounded queue was rejected because model output can contain arbitrarily many
  completed expressions.
- Restarting after every failure was rejected because an incompatible or malicious
  worker could create a crash loop.
