# Codex integration stage 4 acceptance

## Scope

Stage 4 adds a private bounded Codex adapter for the renderer daemon protocol,
process lifecycle, queueing, restart control, cancellation, and shutdown.

The implementation is split into four focused Codex commits:

1. `49de68c040` adds the versioned request and response candidate with strict
   correlation, span, text, geometry, PNG, and allocation validation.
2. `1e2d0a7c4d` accepts the daemon's finite decimal baselines under Codex's
   arbitrary precision JSON feature without weakening numeric bounds.
3. `14102d5be9` adds bounded newline framing and an owned child process with
   graceful and forced reaping.
4. `8e67073cca` adds a bounded nonblocking queue, serial ownership, one restart
   per interval, timeouts, cancellation, shutdown preemption, and source-free
   operational errors.

The adapter remains test-only until the presentation controller consumes it.
This keeps unreferenced candidate APIs out of the production binary while the
next stage adds generation ownership and source-backed transcript state.

## Verification

- All 17 focused protocol, process, and supervisor tests pass.
- The opt-in cross-repository test passes through the Codex supervisor, the real
  `latex-render daemon`, and the built MathJax worker.
- The broad `codex-tui` gate passes all 4,007 selected tests with nine opt-in
  tests skipped. One pre-existing startup paste test passes on its automatic
  retry.
- Required `just fix -p codex-tui`, scoped Rust formatting, and
  `git diff --check` pass after the test gates. Clippy made no changes.
- No dependency, lock, core, protocol, or app-server file changed.
- Every new implementation and test file is below 500 lines. The largest files
  are the 422-line protocol and the 368-line protocol test module.

## Production failure prediction

1. Null, malformed, oversized, or corrupt daemon output could exhaust memory or
   attach an invalid asset. Strict schema decoding, source and response limits,
   exact correlation, span validation, full PNG decoding, and aggregate bounds
   reject the response before it becomes an outcome.
2. Concurrent submissions, crashes, and stale generations could create a restart
   loop or publish an equation for replaced source. One task serializes daemon
   ownership, the bounded channel rejects saturation, restart backoff crosses
   request boundaries, and the next controller stage must generation-check every
   result before publication.
3. Async write, read, decode, cancellation, or shutdown failure could hang the TUI
   or leak a child. A shared request deadline covers exchange and blocking decode,
   shutdown preempts active and queued work, graceful exit is bounded, forced
   termination is reaped, and child drop is kill-protected.

## Self-review

Dependency flow remains one way: the supervisor owns the process, the process
uses only protocol framing constants, and decoded outcomes do not know about TUI
presentation. There is no circular dependency, duplicate supervisor, new public
API, or missing error conversion. Review found and fixed a timeout that initially
excluded PNG decoding, a shutdown allowance that was too close to the child worst
case, leaked fake-daemon artifacts, and restart backoff that a later request could
bypass. The real daemon test also exposed Codex's arbitrary precision decimal
behavior and led to bounded number conversion. The remaining integration gap is
intentional: source-preserving message rewriting, immutable generations, resize
rerendering, and terminal publication belong to the next controller stage.
