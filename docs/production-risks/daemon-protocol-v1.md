# Daemon protocol version 1 production risks

## 1. Malformed or oversized input consumes memory or reflects attacker data

- Trigger: A client sends null JSON, an unknown field, an unsafe identifier, an
  empty or oversized message, invalid colors, invalid geometry, or a line above the
  protocol limit.
- Impact: Parsing could allocate excessively, logs or responses could reflect
  untrusted content, or invalid render parameters could reach the worker.
- Mitigation: The decoder rejects lines above 1 MiB before JSON parsing, uses strict
  unknown-field denial, reflects only a bounded control-free ID, validates source and
  colors, and delegates scale and width validation to render core.
- Test coverage: Focused tests cover malformed JSON, unknown fields, invalid IDs,
  empty, unsafe and oversized source, oversized lines, protocol mismatch, colors,
  scale, and width.

## 2. Correlation or ordering races attach results to the wrong message

- Trigger: A future daemon implementation processes requests concurrently or writes
  responses from more than one task without a single ordering owner.
- Impact: Codex could associate a valid PNG and span list with a different source
  generation.
- Mitigation: Protocol version 1 specifies serial request processing and one response
  line per request. Codex must additionally match the bounded correlation ID and its
  own immutable message generation before accepting an outcome.
- Test coverage: Serialization tests assert exact response IDs, ordered equation
  arrays, byte spans, and stable success and failure shapes. Daemon loop and process
  tests verify serial response order and correlation across malformed input.

## 3. Async worker failure leaves an incomplete or uncorrelated response

- Trigger: Worker startup, rendering, rasterization, timeout, task join, output write,
  or shutdown fails after a request is accepted.
- Impact: The client could wait indefinitely, parse a partial line, or lose readable
  source fallback.
- Mitigation: The protocol represents per-equation retryable failures and top-level
  errors without source. The runtime must serialize a complete bounded line before
  writing, flush it, map every async error, and shut down the supervised child on EOF.
- Test coverage: Protocol tests cover failure serialization and source omission.
  Runtime tests cover worker exit and timeout, partial output writes, EOF, process
  reaping, and bounded shutdown.
