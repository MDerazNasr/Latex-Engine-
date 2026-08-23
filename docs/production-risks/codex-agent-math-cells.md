# Codex agent math cell production risks

Feature commit: `6ecf857fa4`

1. Empty source, unsupported Markdown constructs, malformed spans, or an invalid render plan could replace readable text with blank cells. Planning returns no asset request unless normal Markdown rendering preserves every marker and the source-backed cell retains its ordinary visible and raw fallbacks.
2. A stale identity or wrong-width ready layout could appear in a replacement cell or after resize. The adapter checks the cell identity before planning, the state machine checks it again on completion, and layout retrieval requires the exact planned width.
3. Renderer or preparation failure, test candidate activation before lifecycle wiring, or an unhandled completion could change working Codex behavior. The adapter remains test scoped until invalidation is connected, failures return ordinary source, and all existing AgentMarkdownCell cache and raw-source tests remain unchanged.

Focused verification: four agent math tests and seven existing AgentMarkdownCell tests passed for pending, failed, ready, stale, wrong-width, code-span, cache, visualization, and raw-source behavior. The required Codex fixer and scoped formatter also passed without code warnings.
