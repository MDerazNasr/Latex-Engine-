# Codex restored math scheduler production risks

Feature commit: `c10a27c79e`

1. Null terminal geometry, malformed restored Markdown, exhausted identities, or a transcript cell without canonical source could enter scheduling without a valid render request. Terminal observation and identity allocation fail closed, only source backed agent cells are eligible, and inactive, deferred, or failed cells continue displaying their untouched source.
2. History replay, live consolidation, resize, palette change, thread replacement, and daemon completion can race for limited capacity. Every stale cell becomes source first under an exact successor identity, the live message receives priority, the newest restored cells use remaining capacity, and old completions cannot mutate deferred generations.
3. Queue saturation, controller shutdown, failed submission, poisoned cell state, or an unhandled asynchronous completion could strand work or leak an image. The real nonblocking controller queue remains the only asynchronous queue, saturation leaves cells explicitly deferred, unavailable work fails to source, orphan completions cannot find a publishable state, and controller shutdown owns task and asset cleanup.

Focused verification: all 69 math subsystem tests, four scheduler tests, five Agent math tests, seven existing Agent Markdown tests, ten resize tests, three initial replay tests, and two thread switch replay tests passed. Parallel fake daemon timing retries passed on retry and individually. The required Codex fixer and scoped formatter completed without code findings.
