# Codex math runtime clock production risks

Feature commit: `1d5c8b7e11`

1. Zero terminal geometry, exhausted counters, overflowing message identifiers, or overflowing image identifier blocks could create invalid or colliding requests. Construction and checked advancement reject new work, and math images use a dedicated high identifier range separated from existing pet images.
2. Width, cell pixel size, palette, backend, thread lifecycle, or renderer generation changes could leave an old completion apparently current. The clock captures each dimension independently and produces a new immutable identity before rerendering the same message.
3. A late async completion could arrive after the clock is disabled or a generation can no longer advance. Disabled state exposes no terminal environment and registers no new messages, while the later app event boundary must compare the complete identity before publishing.

Focused verification: six runtime clock tests passed for unique message and image identifiers, width, pixel, theme, backend, lifecycle, renderer, null geometry, disabled state, and generation exhaustion behavior. The required Codex fixer and scoped formatter also passed without code warnings.
