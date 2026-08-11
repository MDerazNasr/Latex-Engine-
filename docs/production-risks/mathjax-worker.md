# MathJax worker production risks

This note records the required failure prediction after the Phase 0 MathJax worker
spike. These are known limits of the spike, not claims that the production controls
are complete.

## 1. Missing, malformed, or oversized input

The protocol rejects null values, malformed JSON, invalid fields, unsafe control
characters, source over 16 KiB, JSONL messages over 64 KiB, and SVG over 2 MiB.
However, Node's line reader can allocate an entire unterminated line before the
handler checks its size.

Production control: replace the line reader with a bounded byte splitter before
exposing the worker to untrusted or arbitrarily large streams. Preserve the current
source-free error responses and add boundary fuzz tests.

## 2. Concurrent rendering and response ordering

The spike processes requests sequentially, so its shared MathJax instance cannot
race and responses remain ordered. A later concurrent handler could mutate shared
renderer state, return responses in a different order, or exceed the two-render and
32-request limits.

Production control: keep one renderer lane until MathJax concurrency is proven,
bound the Rust-side queue, correlate every result by request ID, and test cancellation
and out-of-order completion explicitly.

## 3. Unhandled asynchronous failure or stalled rendering

Promise failures are converted into stable protocol errors and the server reports
fatal initialization failures only on stderr. A CPU-bound or stalled MathJax call
cannot be safely interrupted inside the current process, so a worker-local timer
alone would not guarantee recovery.

Production control: the Rust supervisor must enforce the one-second deadline, kill
and reap the child process on expiry, restart it under a bounded policy, and preserve
the original LaTeX in the UI. The production worker may move rendering to a worker
thread if measurement shows that a separate interruptible lane is needed.

## Additional hardening before MVP exit

- Replace the current defensive reject-list with an explicit SVG element and
  attribute allowlist as required by the architecture specification.
- Generate a reliable spoken accessibility description where possible; raw TeX is
  currently retained as the lossless fallback.
- Add token and nesting-depth enforcement plus protocol and sanitizer fuzz targets.
