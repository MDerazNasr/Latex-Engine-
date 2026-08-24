# Codex renderer discovery production risks

Feature commit: `7100119ee6`

1. Empty, malformed, missing, or nonfile renderer overrides and empty worker or Node values could start an unintended process or create an unusable partial feature. Explicit invalid settings fail closed and never fall through to another candidate.
2. A packaged binary, PATH entry, or configured path could change between discovery and process spawn. Discovery only selects a candidate, while direct argument process creation and the strict daemon handshake remain the authoritative validation boundary.
3. Missing renderer dependencies or asynchronous daemon startup failure could interrupt Codex startup or an agent turn. Discovery returns unavailable source mode without spawning, and the production controller must remain optional and surface failures through source fallback diagnostics.

Focused verification: seven discovery tests passed for sibling and PATH order, explicit direct paths, disabled mode, missing and malformed values, bounded supervisor defaults, and the real environment entrypoint linkage. The required Codex fixer and scoped formatter also passed without code warnings.
