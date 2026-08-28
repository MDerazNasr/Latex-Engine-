# Codex renderer source diagnostics production risks

Feature commit: `460c176304`

1. A missing executable, empty override, malformed path, or null discovery result could be mislabeled as a valid renderer source. The existing fail closed discovery result remains authoritative, and only successful configured, sibling, or `PATH` resolution constructs a typed source value.
2. Environment, executable, or `PATH` state could change after startup and make a later status request appear to describe a new discovery decision. The immutable source value is captured with the resolved executable before controller startup and reused for the lifetime of that runtime generation.
3. Daemon startup, restart, timeout, task join, or shutdown errors could cause diagnostics to infer a source from asynchronous worker state or expose an executable path. Source classification is independent of supervisor health, the status path reports only its bounded source name, and unavailable runtimes remain explicitly unconfigured.

Focused verification: three origin and status tests passed, then all 78 affected math subsystem tests passed. The only retry was the existing fake daemon shutdown timing case, which passed automatically on its second attempt. Both modified production modules remained below 500 lines and the diff check passed.
