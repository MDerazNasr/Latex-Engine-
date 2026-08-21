# Renderer daemon loop production risks

## 1. Null, malformed, or unterminated input desynchronizes the stream

- Trigger: Codex sends null JSON, invalid UTF 8, a line above 1 MiB, or a final line
  without a newline.
- Impact: The daemon could allocate without bound, consume part of the next request,
  exit without a response, or expose source while reporting the failure.
- Mitigation: The asynchronous reader accumulates only through the fixed line limit,
  drains the remainder of an oversized line, accepts a bounded final line at EOF, and
  emits only stable source-free errors.
- Test coverage: Unit tests cover malformed recovery, oversized-line draining, the
  following valid request, final lines without newlines, and source-free I/O errors.

## 2. Ordering or writer races corrupt request correlation

- Trigger: Multiple requests arrive while rendering or a future change introduces
  concurrent response writers.
- Impact: A response could be attached to the wrong Codex message or two JSON lines
  could be interleaved into invalid output.
- Mitigation: One loop owns the writer and processes requests serially. Each complete
  bounded response is serialized before one asynchronous write and flush operation.
- Test coverage: Unit tests verify malformed and valid requests retain arrival order,
  IDs remain correlated, short writes complete one JSON line, and the runtime can
  progress while the input stream is idle.

## 3. Async worker or pipe failure leaves child processes behind

- Trigger: The renderer supervisor exits while input is idle, rendering fails, stdout
  closes, EOF arrives, or child shutdown exceeds its deadline.
- Impact: Codex could wait indefinitely, the CLI could leak a Node process, or a
  partial response could be mistaken for valid output.
- Mitigation: Standard input and output remain asynchronous so supervision continues,
  every render failure becomes a complete response, every serve exit attempts bounded
  worker shutdown, and output failure stops the loop without exposing source.
- Test coverage: Unit tests cover idle runtime progress, render error continuation,
  short output writes, and broken output pipes. The real-process suite must verify EOF
  shutdown, worker exit, timeout, and process reaping before Phase 3 acceptance.
