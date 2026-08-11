# ADR 0007: Keep the standalone CLI strict and source free

- Status: Accepted
- Date: 2026-08-27

## Context

Phase 1 needs a reusable way to validate the independent renderer before terminal
placement and Codex transcript integration exist. The command must be safe in shell
pipelines, produce binary PNG without diagnostic contamination, and exercise the
same worker, sanitizer, cache, and rasterizer boundaries that Codex will use.

Command behavior also needs to remain small enough to audit and compatible with the
Rust 1.85 minimum version.

## Decision

Provide one `latex-render` Rust binary with strict `render` and `check` subcommands.
Use a focused parser instead of adding a general command framework. Reject duplicate
or unknown options, preserve source bytes without normalization, cap redirected
stdin before UTF 8 conversion, and pass worker arguments directly without a shell.

Reserve stdout for raw render output or stable check key-value lines. Send only
source-free diagnostics to stderr and use distinct exit statuses for usage, worker,
render, output, and internal failures. Write and synchronize a temporary sibling,
then publish it atomically without replacement unless the user explicitly supplies
`--force`.

Run the synchronous PNG rasterizer in Tokio's blocking pool. `check` must complete a
fixed render, sanitizer pass, raster pass, and supervised shutdown before it emits a
success report.

## Consequences

- Shell pipelines can consume SVG, PNG, or check output without log bytes mixed in.
- The CLI is a realistic integration harness rather than a second rendering stack.
- Existing files survive accidental output path reuse.
- Worker discovery remains explicit and deterministic across development and
  packaged layouts.
- Terminal detection and placement stay out of Phase 1 and belong to `doctor` in
  Phase 2.

## Rejected alternatives

- A browser preview was rejected because it would bypass terminal and native raster
  behavior.
- Shell command strings were rejected because paths and arguments must not gain
  another interpretation layer.
- Silent output replacement was rejected because a diagnostic utility should not
  destroy an existing artifact by default.
- Using `check` as a TeX syntax validator was rejected because `render` already
  exposes render errors while `check` must validate installation health.
