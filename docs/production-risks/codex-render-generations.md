# Codex render generation production risks

Feature commit: `c52311a581`

1. A zero message identifier, zero terminal width, invalid UTF-8 equation span, or exhausted generation could make two requests indistinguishable. Constructors reject invalid values and generation advancement fails closed.
2. A message replacement, resize, pixel metric change, theme change, renderer restart, backend change, thread transition, or feature lifecycle change could accept stale pixels. Publication requires exact equality across the complete immutable render identity.
3. A cancelled, timed out, or delayed asynchronous result could arrive after its owner is gone. The completion must still pass the same identity comparison at the app event boundary before terminal publication.

Focused verification: five generation tests passed, including every environment mismatch, message replacement, invalid UTF-8 spans, zero values, and counter exhaustion. The required Codex fixer and scoped formatter also passed.
