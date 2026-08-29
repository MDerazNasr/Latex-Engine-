# Codex Phase 3 final acceptance

Acceptance date: 2026-08-27

## Verdict

Version 0.1.0 is accepted as an experimental local developer distribution. Ordinary
assistant Markdown remains canonical while recognized LaTeX is rendered
asynchronously into terminal images in supported interactive terminals. Every
unavailable, pending, malformed, stale, disabled, redirected, unsupported, or failed
path preserves readable source.

This acceptance does not represent an upstream Codex release. It covers the two
local feature branches and the explicit `codex-latex` bundle described below.

## Accepted revisions and artifact

- Renderer repository branch: `feature/phase-3-codex-integration`.
- Codex repository branch: `feature/latex-rendering-integration`.
- Pinned upstream Codex base: `b68acc4d4b56fdfa1d5b6a2c36102c66876e0c46`.
- Accepted Codex integration head: `229d0cd2ae`.
- Artifact: `dist/codex-latex-0.1.0-aarch64-apple-darwin`.
- Artifact identity: version 0.1.0, target `aarch64-apple-darwin`, 367 MB, and
  1,254 manifest-hashed payload files.

The bundle keeps `codex-latex`, `latex-render`, and the MathJax worker in one owned
version root. It neither creates nor changes a normal `codex` command.

## Accepted behavior

The final path is:

1. Codex stores and streams the original assistant Markdown without renderer edits.
2. A private bounded controller sends completed source to the local protocol-v1
   `latex-render daemon` process.
3. The daemon segments protected Markdown and supported delimiters, renders with one
   supervised MathJax worker, sanitizes SVG, and rasterizes bounded PNG assets.
4. Completion events carry immutable message, equation, renderer, geometry, theme,
   and backend generations back to the TUI.
5. The presentation layer replaces only current equation spans with cell-aligned
   Kitty, iTerm2, or Sixel placements. Source remains canonical for copy, raw output,
   export, app-server data, model context, resize replay, and session restore.

Supported delimiters are `\(...\)`, `\[...\]`, `$$...$$`, and smart `$...$`.
Protected code, links, escaped delimiters, currency-like dollars, malformed input,
and unsupported environments remain source.

`/math source`, `/math on`, `/math off`, and `/math auto` change only the current
presentation. `/math status` reports source-free terminal, discovery, worker, task,
error, protocol, configuration, and resource-bound diagnostics.

## Renderer verification

The following final engine gates passed:

- `cargo test --workspace --all-targets`: 200 passed and 3 intentionally ignored.
- All three real-worker ignored tests were then run explicitly and passed: daemon,
  supervised client, and reviewed rendering snapshots.
- Worker formatting, TypeScript checking, and all 14 Node tests passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo fmt --all -- --check` passed.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` passed.

The release benchmark passed every target:

| Metric | Measured | Target |
| --- | ---: | ---: |
| First render | 203.430 ms | at most 1,500 ms |
| Simple uncached p95 | 1.340 ms | at most 150 ms |
| Complex uncached p95 | 3.049 ms | at most 500 ms |
| Cached lookup p95 | 0.002 ms | at most 10 ms |
| Segmenter delta p95 | 0.001 ms | at most 5 ms |

## Codex verification

The final permission-enabled broad run used the pinned checkout and excluded only
the proven upstream baseline key-hint snapshot name:

```text
4099 tests run: 4099 passed, 9 skipped
```

One existing fake-daemon shutdown timing test passed on its automatic retry. The
two project-owned command-popup expectations found by the preceding unfiltered run
were corrected in `d25e485807`, and their focused tests passed.

The exclusion expression matched both the regular and plan-mode variants of
`hidden_shell_paste_queued_during_turn_submits_literal_prompt`. The plan-mode variant
passed in isolation. The regular snapshot also fails on the unchanged pinned base:
the current output selects the existing `shift + left` hint while the snapshot
hardcodes the existing `option + up` hint. No integration code, keymap, test, or
snapshot differs from the pinned base on that path, so unrelated upstream behavior
was not rewritten for this release.

After the final renderer-origin correction, all 78 affected math subsystem tests
passed. The focused origin tests cover configured, bundled sibling, and `PATH`
discovery, plus status rendering. The repository-required final lint autofix and
scoped formatter passed and changed no tracked files. No tests were run after that
final formatting gate.

## Bundle acceptance

The final bundle was rebuilt from both clean branches with frozen pnpm dependencies
and locked Cargo releases. The Codex release link completed in 23 minutes 20 seconds.
The resulting artifact passed all of these checks:

- Direct `latex-render check` reported status ok, protocol 1, MathJax 0.1.0, a ready
  worker, zero restarts, and no error.
- A real `E = mc^2` render produced a 140 by 33, 8-bit RGBA PNG of 1,991 bytes.
- `codex-latex --version` reported `codex-cli 0.0.0` from the packaged binary.
- Installation verified every manifest entry before creating the three activation
  links in a fresh isolated prefix.
- With renderer and worker overrides removed, the installed `/math status` reported
  `renderer_source=sibling`, `worker_source=packaged`, protocol 1, zero restarts,
  and no renderer error.
- Owned uninstall reverified the installed manifest and activation links, removed
  only bundle-owned content, and left an empty prefix.

Two packaging defects were discovered and corrected during real acceptance. The
installer now exposes the packaged worker through an owned share link because macOS
resolves the renderer executable symlink to its version root. The bundle also stages
the locked `@mathjax/mathjax-newcm-font` dependency required by MathJax at runtime.
The superseded artifact and temporary acceptance installs were removed only after the
corrected canonical bundle passed post-promotion checks.

## Terminal coverage

The terminal transport and layout suites cover Kitty direct transfer, iTerm2 3.6
local-file transfer, Sixel encoding and clearing, deletion ordering, partial writes,
cursor restoration, resizing, and unsupported fallbacks. Earlier direct presentation
smokes established the Kitty and iTerm2 transport paths.

The final installed TUI acceptance ran inside the tool's tmux PTY. It correctly
selected text fallback while still proving bundled discovery, controls, diagnostics,
startup, and shutdown. A release operator should repeat the installed natural-answer
smoke in each target graphical terminal before distributing a binary to other users.
Sixel remains automated-only in this acceptance. Tmux and Zellij passthrough are
intentionally disabled for the MVP.

## Self-review

- Circular dependencies: none found. Codex communicates with the renderer only
  through the versioned local process protocol and has no sibling Cargo dependency.
- Redundant logic: the old pet-specific transport and resize bodies were retained
  only until their versioned replacements passed equivalence tests, then reduced to
  narrow callers. No parallel math renderer path remains.
- Missing error handling: protocol bounds, queue saturation, timeouts, worker crash,
  malformed data, raster failure, terminal write failure, stale generations, and
  shutdown all preserve source and have focused coverage.
- Integration gaps found and fixed: installed worker activation, MathJax font
  closure, stale command-popup expectations, and ambiguous renderer-origin status.
- File boundaries: every new or replacement production and test module is below 500
  lines. Larger remaining upstream files predate this work and were not expanded with
  the new implementation.

## Remaining experimental limits

- The changes remain on local feature branches and have not been merged, pushed, or
  proposed upstream.
- Node.js 22 or newer remains an external runtime dependency.
- Windows packaging, embedded Node, multiplexer image passthrough, and a public Codex
  configuration contract are deferred.
- Protocol v1 does not expose worker version or cache telemetry through the Codex
  status view, although standalone renderer diagnostics report them.
- The one pinned-upstream key-hint snapshot mismatch remains outside this integration.

The codebase is therefore ready for local experimental use and merge review. The
next distribution step is a direct-terminal operator smoke followed by an explicit
decision to merge, push, or prepare an upstream proposal.
