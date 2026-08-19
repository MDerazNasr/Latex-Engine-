# ADR 0018: Versioned local renderer daemon

## Context

The experimental Codex checkout must consume the independent renderer without a
sibling Cargo dependency, a network service, or one worker process per equation. The
standalone CLI specification did not define a persistent integration boundary.

## Decision

Add an internal `latex-render daemon` command with versioned newline-delimited JSON
over standard input and standard output. Version 1 accepts canonical Markdown, owns
segmentation and rendering, and returns ordered equation outcomes identified by byte
spans. Rendered outcomes contain bounded PNG data and layout metadata. Failed
outcomes contain only a stable error code and retryable flag.

The daemon processes one message at a time through one supervised worker. This
backpressure bounds memory and simplifies lifecycle ownership, while the Codex
controller remains asynchronous and owns its separate bounded queue. No shell is
involved, no arbitrary output path is accepted, and no source is repeated in
responses or diagnostics.

## Consequences

- Codex and the renderer can be built, tested, installed, and upgraded independently.
- One long-lived process preserves MathJax warm state and cache entries.
- Protocol, message, equation, PNG, aggregate output, and line limits are explicit.
- Concurrent daemon work requires a future protocol revision or compatible ordering
  rule.
- EOF provides graceful shutdown; abrupt loss relies on child kill-on-drop and Codex
  source fallback.

## Rejected alternatives

- A sibling Cargo dependency would make Codex nonreproducible and not fit Bazel.
- One CLI invocation per equation would discard warm state and complicate cancellation.
- A local network service would add authentication and privacy surface without value
  for the single-client MVP.
- Returning temporary file paths would expand filesystem trust and cleanup coupling.

