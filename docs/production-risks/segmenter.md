# Production Risks: Streaming Segmenter

Feature branch: `feature/phase-0-segmenter-spike`  
Reviewed: 2026-08-26

## 1. Missing, malformed, or oversized input

Trigger: Empty chunks, unmatched delimiters, adversarial nesting, or a math
candidate that never closes. Rust `&str` prevents null and invalid UTF-8 at this
API boundary, but an FFI or protocol adapter could still omit the field before
calling the segmenter.

Impact: Source could disappear on finalization or an open candidate could consume
unbounded memory.

Mitigation: Empty input is a no-op and `finish` returns incomplete candidates as
lossless text. Phase 1 must add the specified 16 KiB candidate limit and adapter
validation before external inputs reach this API.

Coverage: Integration tests cover empty chunks and finalization, unmatched math,
unmatched code, Unicode byte spans, and lossless reconstruction. The size
limit remains intentionally open for Phase 1.

## 2. Out-of-order chunks across concurrent tasks

Trigger: An integration wraps one segmenter in asynchronous synchronization and
feeds deltas in task-completion order rather than message-sequence order.

Impact: Delimiters can pair with the wrong content and byte spans no longer match
the canonical transcript.

Mitigation: The mutable API prevents ordinary simultaneous mutation. The Codex
adapter must serialize one segmenter per message and validate monotonically
increasing event sequence numbers before calling `push`.

Coverage: Streaming tests exercise every single split point and one-character
chunks. Sequence-number rejection belongs to the future Codex adapter tests.

## 3. Cancellation or unhandled asynchronous termination

Trigger: A turn is cancelled, replaced, or dropped while a candidate is pending,
or the render task fails before the integration finalizes parsing.

Impact: The presentation layer may omit the buffered source even though the
canonical conversation still contains it.

Mitigation: The transcript remains canonical outside the segmenter. Every normal,
cancelled, and failed completion path must call `finish` or rebuild presentation
from canonical source. Rendering errors must not own transcript state.

Coverage: Tests prove pending candidates return as text through `finish`. Async
cancellation coverage belongs to the future renderer client and Codex adapter.
