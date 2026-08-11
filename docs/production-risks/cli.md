# Standalone CLI production risks

This note records the required failure prediction for `latex-render render` and
`latex-render check`.

## 1. Null, malformed, or oversized input reaches the worker

Trigger: a user omits source on an interactive terminal, redirects empty, non UTF 8,
or oversized data, supplies duplicate options, or configures a missing worker or
Node executable.

Impact: the command could block unexpectedly, allocate too much memory, start the
wrong process, or leak source through an error.

Mitigation: argument parsing is strict, stdin is read only when redirected and is
capped before conversion, render contracts reject empty or control-bearing source,
worker paths must identify files, and process arguments never pass through a shell.

Test coverage: unit tests cover absent, duplicate, unknown, non UTF 8, invalid color,
invalid scale, and invalid width values. Process tests cover redirected input, empty
source, missing workers, explicit paths, and environment discovery.

## 2. Output or worker discovery races with external changes

Trigger: another process creates an output path at the same time, a configured worker
is replaced after discovery, or a force write races with another writer.

Impact: an existing artifact could be lost, output could be partial, or a different
worker file could start than the one that was inspected.

Mitigation: output is completely written and synchronized to an exclusively created
temporary sibling before publication. Non-force publication uses an atomic hard link
that fails when the destination exists, while force publication uses an atomic rename
on the macOS and Linux MVP. The worker must still pass an exact protocol and version
handshake after process start.

Test coverage: integration tests precreate an output file, verify refusal and byte
preservation, then verify explicit replacement. Worker client tests reject
incompatible handshakes and malformed responses.

## 3. Worker, raster, shutdown, or output work fails asynchronously

Trigger: the worker exits or hangs, the raster blocking task panics, process cleanup
fails after a render, or stdout closes while binary output is being written.

Impact: the command could claim success without a complete artifact, leave a child
process, or contaminate a binary stream with diagnostics.

Mitigation: every render awaits supervised shutdown, raster joins are mapped to an
internal failure, success output is withheld until rendering and cleanup complete,
and output errors use stderr with exit status 5. Worker and render errors retain
their distinct source-free status codes.

Test coverage: CLI tests cover worker and output failures while client tests cover
crash, timeout, cancellation, restart, backoff, and reap behavior. Full workspace
tests verify that no success bytes are emitted on checked failure paths.
