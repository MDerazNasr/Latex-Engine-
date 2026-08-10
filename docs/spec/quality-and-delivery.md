# Quality and Delivery Specification

This document is normative and is part of the specification indexed by
[`PROJECT_SPEC.md`](../../PROJECT_SPEC.md).

## 13. Security and privacy requirements

Assistant-produced LaTeX is untrusted input.

### 13.1 MVP threat model

The math renderer must prevent:

- arbitrary file reads or writes;
- shell or subprocess execution initiated by TeX input;
- network access and remote resource loading;
- external image, font, stylesheet, or script references;
- XML/SVG script execution;
- path traversal;
- unbounded macro expansion;
- exponential parser behavior;
- excessive input, output, memory, CPU, or recursion;
- terminal escape injection; and
- malicious accessibility text or error messages containing control characters.

### 13.2 Required controls

- Accept math fragments, not full TeX documents.
- Use a fixed allowlist of MathJax extensions and macros.
- Disable URL/resource-producing commands unless safely rewritten.
- Sanitize SVG using an explicit element/attribute allowlist.
- Strip scripts, event handlers, external references, foreign objects, and control
  characters.
- Cap source length, token count, nesting, SVG bytes, dimensions, and render time.
- Run the worker with no network and minimum filesystem access where supported.
- Apply timeouts in both client and worker.
- Kill and reap the worker process tree on timeout or shutdown.
- Use no `--shell-escape` or native TeX compiler in the inline path.
- Fuzz the segmenter, protocol decoder, sanitizer, and rasterizer boundary.

Suggested initial hard limits, subject to benchmark validation:

| Resource | MVP limit |
|---|---:|
| Source length | 16 KiB per expression |
| Nesting depth | 64 |
| Render timeout | 1 second |
| SVG response | 2 MiB |
| Raster dimensions | 4096 × 2048 px |
| Concurrent renders | 2 |
| Pending queue | 32 expressions |

If the queue is full, later expressions fall back to source rather than blocking
Codex.

### 13.3 Privacy

- Rendering is local by default.
- Equation source is not sent to a new network service.
- Logs must not contain equation source unless debug logging is explicitly enabled.
- Crash reports should contain error codes and versions, not transcript content.
- A future remote renderer must be opt-in and documented as a separate data path.

## 14. Reliability and performance requirements

### 14.1 Performance targets

Measured on a supported developer laptop after worker warm-up:

| Metric | Target |
|---|---:|
| Cached expression lookup and layout | p95 ≤ 10 ms |
| Uncached simple expression render | p95 ≤ 150 ms |
| Complex expression render | p95 ≤ 500 ms |
| Added time to prose streaming | p95 ≤ 5 ms per delta |
| Resident memory added after warm-up | ≤ 150 MiB |
| First worker readiness | ≤ 1.5 seconds |

Rendering occurs off the TUI thread. A missed target degrades to a pending/source
view, never to a frozen interface.

### 14.2 Reliability targets

- Rendering failures do not terminate Codex.
- The segmenter is lossless for 100% of test and fuzz inputs.
- Repeated redraw/resize leaves no stale terminal images.
- The worker protocol rejects malformed or mismatched-version messages.
- Shutdown leaves the terminal and child-process table clean.

## 15. Accessibility requirements

- Raw LaTeX remains available at all times.
- Rendering must not be the only representation in exported or copied content.
- Color is not used as the sole error indicator.
- The worker produces plain-text accessibility descriptions where reliable.
- A user can disable graphical math globally.
- Reduced-motion settings are respected; replacement should not animate.
- High-contrast foreground and transparent-background behavior must be tested.

## 16. Testing strategy

### 16.1 Unit and property tests

- Delimiter and environment recognition.
- Every possible streaming chunk boundary for golden expressions.
- Markdown code-span/fence precedence.
- Currency, shell, template-language, and escaped-dollar false positives.
- Lossless reconstruction property.
- Cache-key stability and invalidation.
- Protocol versioning and malformed messages.
- SVG sanitizer allowlist/denylist.
- Dimension and resource-limit enforcement.

### 16.2 Fuzzing

Fuzz targets:

- streaming segmenter with arbitrary UTF-8 and chunk boundaries;
- JSONL decoder and response correlation;
- SVG sanitizer;
- terminal control-sequence generation; and
- resize/reflow state transitions.

### 16.3 Rendering tests

- Canonical SVG snapshots for a curated formula corpus.
- PNG perceptual snapshots with a small tolerance.
- Light and dark themes.
- Inline and display baselines.
- Wide matrices, long derivations, nested fractions, accents, Unicode, and invalid
  input.
- Renderer upgrade review using explicit snapshot diffs.

The Phase 1 snapshot suite renders every entry in
`fixtures/rendering/math-corpus.json` once with the dark foreground `#e6edf3` and
once with the light foreground `#111827`, both on a transparent background. The
sanitized SVG must match byte for byte. PNG comparison decodes RGBA pixels, requires
equal dimensions, permits a maximum channel delta of 2, and permits at most 0.1
percent of pixels to differ. Snapshot replacement requires the explicit
`UPDATE_LATEX_SNAPSHOTS=1` environment variable and a reviewed fixture diff.

The release-mode `latex-bench` harness measures first render, warmed uncached simple
render, warmed complex render, cached lookup, and segmenter delta p95 values. It uses
one supervised worker session, distinct sources for uncached samples, at least 50
simple render samples, at least 30 complex samples, and at least 500 cached samples.
The harness exits unsuccessfully when the targets in section 14.1 are missed.

### 16.4 Integration tests

- Replay recorded Codex message-delta sequences.
- Verify prose continues streaming while math is pending.
- Verify cancellation, worker crash, timeout, and restart behavior.
- Verify terminal resize and transcript scroll.
- Verify no image escape sequences when stdout is redirected.
- Verify raw source remains in stored/exported transcript content.

### 16.5 Terminal matrix

MVP manual and automated smoke testing should cover:

- macOS: iTerm2, Kitty, WezTerm, Apple Terminal fallback;
- Linux: Kitty, WezTerm, and one Sixel-capable terminal;
- `tmux` with and without passthrough;
- local and SSH sessions;
- light and dark themes; and
- narrow, normal, and very wide windows.

Windows support is required before a stable 1.0 release but may follow the macOS
and Linux MVP after terminal-protocol feasibility is demonstrated.

## 17. Observability and diagnostics

`/math status` and `latex-render doctor` should report:

- feature enabled/disabled state;
- detected terminal and selected backend;
- `tmux`/SSH detection;
- renderer and protocol versions;
- worker health and restart count;
- cache size/hit rate for the current session;
- active safety limits; and
- the most recent error code without transcript content.

Debug logging is opt-in, goes to stderr or a user-selected file, redacts equation
source by default, and never contaminates MCP/worker stdout.

## 18. Build and distribution

### 18.1 Development stack

- Rust stable for the segmenter, render client, cache, terminal integration, and
  standalone CLI.
- TypeScript on a current supported Node.js LTS for the MathJax worker.
- Locked Cargo and npm dependency graphs.
- Formatting, linting, tests, fuzz smoke tests, and security audit checks in CI.

The exact Rust edition and minimum compiler version will match the target Codex
release when TUI integration begins.

### 18.2 Packaging options

MVP developer installation may require Rust, Node.js, and the experimental Codex
build. A beta release should provide reproducible binaries/packages for macOS and
Linux.

Preferred long-term packaging, in order:

1. Rendering support accepted into Codex with a separately installed worker.
2. A self-contained worker bundle embedded or installed alongside Codex.
3. An experimental `codex-latex` distribution while upstream work proceeds.

The project must not silently replace a user's normal `codex` executable.
Installation and rollback must be explicit.

## 19. Delivery plan

### Phase 0: feasibility spike

Deliverables:

- Render 25 representative expressions to SVG with the proposed worker.
- Display one static equation in Kitty/WezTerm and iTerm2 from a Rust TUI spike.
- Prove image deletion/redraw and terminal resize behavior.
- Replay a streamed response through the proposed segmenter.
- Identify the smallest viable Codex TUI integration seam.

Exit criteria:

- At least two terminal protocols work on macOS.
- Rendering does not corrupt terminal state or block the UI.
- A written go/no-go decision records dependency and upstream constraints.

Estimated effort: 3–5 engineering days.

### Phase 1: independent renderer MVP

Deliverables:

- Rust workspace and public core types.
- Streaming segmenter with golden/property tests.
- Versioned MathJax worker protocol.
- SVG sanitizer, resource limits, cache, and standalone CLI.
- Snapshot corpus and benchmarks.

Exit criteria:

- `latex-render render` and `latex-render check` meet correctness and safety tests.
- Parser is lossless and invariant to stream chunking.
- Warm simple renders meet the p95 latency target.

Estimated effort: 1–2 engineering weeks.

### Phase 2: terminal presentation

Deliverables:

- Kitty and iTerm2 backends.
- Rasterization, sizing, baseline, theme, resize, and source fallback.
- Terminal capability detection and `doctor` command.
- Sixel investigation; include if stable within the milestone.

Exit criteria:

- Representative equations render without stale images in supported terminals.
- Redirected output and unsupported terminals are clean source-only text.

Estimated effort: 1–2 engineering weeks.

### Phase 3: Codex integration

Deliverables:

- Experimental Codex branch/build.
- Async transcript integration and configuration.
- Source toggle, failure UI, cancellation, and worker supervision.
- Replayed and live end-to-end tests.
- Upstream feature proposal or pull request.

Exit criteria:

- Asking an ordinary Codex math question renders delimited math automatically.
- No prompt or tool call is necessary.
- Rendering failure has no effect on the agent turn.
- Non-interactive Codex behavior remains compatible.

Estimated effort: 1–3 engineering weeks, excluding upstream review time.

### Phase 4: hardening and beta

Deliverables:

- Fuzzing campaign and security review.
- `tmux`, SSH, Linux, and accessibility validation.
- Reproducible packages, install guide, rollback guide, and release notes.
- Optional MCP artifact tool.

Exit criteria:

- No unresolved high-severity security findings.
- Supported terminal matrix and limitations are published.
- Fresh-machine installation and removal are verified.

Estimated effort: 1–2 engineering weeks.

Overall MVP-to-beta estimate: approximately 5–9 engineering weeks for one
experienced engineer, with upstream review and cross-terminal debugging as the
largest schedule uncertainties.

## 20. MVP acceptance criteria

The MVP is accepted when all of the following are true:

1. In a supported terminal, a normal Codex response containing `\[E=mc^2\]`
   automatically shows a typeset equation without an explicit render request.
2. Inline `\(x^2\)` and display `$$x^2$$` are distinguished and laid out
   appropriately.
3. Code blocks, escaped delimiters, and the price `$19.99` remain source text.
4. Fragmented streaming delimiters produce the same result as a single complete
   message.
5. Raw LaTeX can be revealed and copied.
6. Invalid LaTeX, a worker crash, or a timeout leaves readable source and does not
   interrupt the Codex turn.
7. Unsupported terminals and redirected output contain no image-control garbage.
8. Theme changes and terminal resizing update the presentation correctly.
9. The renderer performs no network, shell, or arbitrary filesystem operations.
10. The feature can be disabled completely through configuration.

## 21. Release criteria beyond MVP

### Beta

- macOS and Linux packages.
- Kitty, iTerm2, and at least one additional backend.
- `tmux` documentation and fallback behavior.
- Security review and sustained fuzzing.
- Compatibility with at least one released Codex version.

### 1.0

- A stable Codex distribution path, preferably upstream.
- Windows terminal support or a documented high-quality fallback.
- Semantic versioning for renderer protocol and library APIs.
- Upgrade/rollback documentation.
- Published compatibility and performance test results.
- No known terminal-state corruption bugs.

## 22. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Codex has no plugin transcript hook | MCP-only design cannot meet the goal | Integrate in open-source TUI; keep renderer independent |
| Upstream does not accept Node/MathJax | Distribution remains awkward | Backend trait; prototype a native replacement; separate worker package |
| Terminal image protocols behave differently | Broken layout or stale images | Backend abstraction, strict matrix, source fallback |
| Inline images do not align well in cell grids | Poor readability | Baseline metrics; promote oversized inline math to blocks |
| `$` causes false positives | Prices or shell text render as math | Conservative smart mode; default to source on uncertainty |
| Rendering blocks streaming | Codex feels slower | Async worker, bounded queue, pending/source state |
| Malicious TeX/SVG exhausts resources | Crash or local compromise | Math-only backend, allowlists, timeouts, sanitizer, fuzzing |
| Codex TUI internals change | Patch maintenance | Narrow adapter and CI against supported/upstream versions |
| `tmux`/SSH blocks protocols | No graphical display | Detect passthrough and fall back predictably |
| Accessibility regresses | Rendered content becomes unusable | Preserve source and plain-text representations |

## 23. Open questions to resolve during Phase 0

1. Which Codex transcript/Markdown abstraction is the narrowest stable insertion
   point for a rendered-math item?
2. Can the existing TUI image infrastructure be reused, or is a dedicated widget
   required?
3. Which terminal protocol should be the first reference backend: Kitty or iTerm2?
4. Will upstream accept an external worker, an optional feature, or only a native
   Rust renderer?
5. How should inline images participate in selection and copy behavior?
6. Can terminal cell pixel dimensions be queried reliably without visible probes?
7. Should `$...$` be enabled by default or only `\(...\)`/display delimiters?
8. What exact controls are needed to pass SVG safely into each terminal backend?
9. Should the experimental distribution patch Codex directly or ship a separate
   binary name?
10. Which accessibility representation is reliable enough for the MVP?

## 24. Immediate next actions

1. Create the Rust workspace and `latex-segmenter` crate.
2. Build a 100-case parsing corpus before integrating a renderer.
3. Implement the versioned MathJax JSONL worker and validate its SVG output.
4. Run the Phase 0 terminal-image spike in Kitty/WezTerm and iTerm2.
5. Inspect the target Codex version's Markdown/transcript rendering path and write
   a short integration design note with the exact affected modules.
6. Open an upstream Codex feature discussion after the feasibility evidence exists.

## 25. Documentation references

The integration decisions in this specification are based on current official
OpenAI documentation:

- Codex MCP configuration and supported transports:
  <https://learn.chatgpt.com/docs/extend/mcp?surface=cli>
- Codex CLI customization surface:
  <https://learn.chatgpt.com/docs/cli-customization>
- Codex App Server for building rich clients:
  <https://learn.chatgpt.com/docs/app-server>
- Open-source Codex components:
  <https://learn.chatgpt.com/docs/open-source>
- Codex plugin packaging:
  <https://learn.chatgpt.com/docs/build-plugins>

The documentation establishes the available extension paths. The conclusion
that transparent per-message rendering requires a presentation-layer integration
is an architectural inference from those documented boundaries and should be
revalidated against the target Codex source revision during Phase 0.
