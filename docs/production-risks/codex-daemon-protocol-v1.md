# Codex daemon protocol version 1 production risks

## 1. Null, malformed, or oversized data reaches transcript presentation

- Trigger: A missing or compromised daemon returns null JSON, unknown fields, invalid
  numbers, excessive equations, oversized PNG data, or unsafe accessibility controls.
- Impact: Codex could allocate without bound, accept corrupt terminal assets, or show
  control characters while processing an otherwise valid assistant response.
- Mitigation: The private decoder rejects response lines above 12 MiB before parsing,
  denies unknown fields, enforces every per-equation and aggregate limit, and fully
  decodes each bounded PNG under strict dimensions before exposing it.
- Test coverage: Focused tests cover null and malformed JSON, unknown fields, equation,
  response, accessibility, per-image, aggregate PNG, geometry, and corrupt PNG limits.

## 2. Correlation or span races attach an image to different source

- Trigger: A daemon returns the wrong ID, overlapping or unordered spans, a UTF 8
  interior offset, or a result from an older message generation.
- Impact: A valid equation image could replace unrelated text or appear after the
  source, width, theme, or terminal backend has changed.
- Mitigation: Protocol decoding requires the exact request ID, ordered nonoverlapping
  UTF 8 byte ranges inside the submitted source, and complete outcome validation.
  The later presentation controller must additionally match immutable generations.
- Test coverage: Protocol tests cover wrong IDs, invalid UTF 8 boundaries, bounds,
  overlap, order, and exact success and failure correlation shapes.

## 3. Async failure leaves an untrusted or partially decoded image alive

- Trigger: Base64 decoding, PNG parsing, pixel decoding, task cancellation, response
  timeout, or daemon shutdown fails after a request has entered the queue.
- Impact: Codex could retain unnecessary memory, wait indefinitely, publish a partial
  asset, or leak the owned renderer process.
- Mitigation: The decoder constructs no outcome until base64, geometry, allocation,
  and complete PNG decoding pass. The supervisor must add bounded queueing, timeouts,
  cancellation checks, restart throttling, and child reaping before production use.
- Test coverage: Decoder tests cover invalid base64, corrupt and mismatched PNG data,
  decoded pixel limits, and aggregate bounds. Supervisor tests now cover timeouts,
  cancellation, restart throttling, shutdown, and the real engine daemon. The adapter
  remains test-only until the source-backed presentation controller consumes it.
