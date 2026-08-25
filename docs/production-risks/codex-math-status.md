# Codex math status production risks

Feature commit: `9bf8b63da2`

1. Missing configuration, malformed environment values, unsafe daemon error text, or unavailable terminal metadata could expose source content or make status fields ambiguous. Discovery failures map to closed error codes, runtime absence uses explicit disabled or unavailable states, error values are validated both when recorded and when displayed, and protocol v1 omissions are labeled unreported instead of inferred.
2. Status requests can race with daemon startup, restart, completion, task reaping, source toggles, raw output changes, and shutdown. A shared mutex protected snapshot owns worker health and saturating restart counts, generation control remains independent of diagnostics, completed task sets are reaped before counts are reported, and poisoned state fails to a source free degraded snapshot.
3. Renderer timeouts, worker crashes, malformed responses, raster failures, task join errors, queue saturation, and shutdown failures could otherwise disappear through asynchronous paths. Supervisor and controller boundaries record bounded diagnostic codes for each path, late completion behavior remains generation gated, shutdown marks the worker stopped, and the status path never waits on renderer work.

Focused verification: all 74 math subsystem tests passed with one existing fake daemon timing retry, its isolated rerun passed on the first attempt, both math status tests passed, and both math slash command tests passed. The required Codex fixer completed without warnings, the scoped formatter completed, the diff check passed, and every new diagnostics module remained below 500 lines.
