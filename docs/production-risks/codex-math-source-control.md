# Codex math source control production risks

Feature commit: `d31356759c`

1. A missing toggle value, repeated command, disabled renderer, text backend, unsupported terminal, or source request without active math state could hide content or report a false rendered state. Bare commands map to a typed toggle, explicit arguments map to source or configured rendering, unavailable graphical rendering keeps canonical LaTeX visible, and every no state path remains source backed.
2. Source reveal, raw mode, startup raw configuration, resize, palette change, thread replacement, and daemon completion can race with one another. User source intent and raw mode are independent reasons for source view, every effective transition invalidates the lifecycle generation, active placements fail to source before reflow, and old completions cannot publish after a toggle.
3. Queued rendering, controller shutdown, failed transcript reflow, or a late asset task could remain unhandled while source view is active. The bounded controller retains ownership of existing asynchronous work, runtime gates reject new requests and asset preparation, late events fail against invalid identities, controller cleanup owns temporary assets, and reflow errors remain visible without replacing source.

Focused verification: all 72 math subsystem tests passed with two existing fake daemon timing retries, both isolated reruns passed on their first attempt, two slash command tests passed, two runtime control tests passed, one source and resume integration test passed, four existing raw output tests passed, both existing raw slash tests passed, thirteen slash popup tests passed, and five slash command contract tests passed. The required Codex fixer and scoped formatter completed without code findings.
