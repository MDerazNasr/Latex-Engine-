# ADR 0014: Publish doctor output only after the full check succeeds

Status: Accepted

Date: 2026-08-27

## Context

Users need one command that distinguishes a broken worker or native renderer from an
unsupported terminal. The existing `check` command already verifies the worker
handshake, MathJax render, sanitizer, native rasterizer, worker health, shutdown, and
active limits. Reimplementing that path for terminal diagnostics would create two
definitions of renderer health and could print a misleading partial success report.

## Decision

Add `doctor` as an additive command with the same worker selection options as
`check`. Run the complete existing check first and keep its report in memory. Only
after successful render, raster, health validation, and shutdown, append a passive
terminal snapshot containing TTY state, backend, fallback reason, SSH, tmux, Zellij,
and Screen facts. Publish the combined UTF 8 key value report in one command output.

Treat the text backend as a successful diagnostic result because source fallback is
intentional. Preserve existing worker, render, and internal exit categories. Emit no
stdout bytes when any pipeline step fails.

## Consequences

- `check` remains behaviorally unchanged and scripts can continue using it.
- `doctor` proves both the renderer pipeline and terminal decision in one invocation.
- Redirected doctor output accurately reports redirected source fallback.
- Terminal facts are a point in time snapshot and may change after the command exits.

## Rejected alternatives

- Printing terminal facts before worker startup was rejected because later failure
  would leave a partial report that appears actionable.
- Treating an unsupported terminal as an error was rejected because source text is a
  supported product mode.
- Reporting raw environment values was rejected because stable normalized facts are
  safer and easier for scripts to consume.

