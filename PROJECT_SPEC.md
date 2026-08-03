# Codex LaTeX Renderer: Project Specification

Status: Proposed  
Target release: MVP 0.1  
Last updated: 2026-08-26

## 1. Executive summary

Codex frequently writes mathematical notation using LaTeX delimiters such as
`\(...\)`, `\[...\]`, and `$$...$$`. In a browser, those expressions can be
typeset automatically. In a terminal, the user normally sees the source syntax.

This project will add automatic, low-latency mathematical typesetting to the
Codex CLI. When an assistant response contains supported LaTeX, the terminal UI
will detect it, render it, and place the rendered equation in the transcript
without requiring the user to ask Codex to invoke a tool.

The product will preserve the original LaTeX for copying, accessibility,
debugging, and terminals that cannot display graphics. Rendering is a
presentation concern: it must not alter the conversation sent to the model or
the stored transcript.

The recommended implementation is a Rust core and Codex TUI integration, with a
long-lived TypeScript/MathJax worker for the first rendering backend. Rust is the
right host language because Codex CLI and its TUI are open-source Rust projects.
MathJax provides mature TeX-to-SVG behavior without implementing TeX layout from
scratch. The renderer boundary will remain language-independent so the worker
can later be replaced with a native Rust backend.

## 2. Product vision

Mathematics in Codex should feel like mathematics, not serialization syntax.

Given an assistant response containing:

```text
The roots are
\[
x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}.
\]
```

the transcript should show the prose followed immediately by a typeset quadratic
formula. The user should not have to say "render this," install a prompt rule for
each project, or wait for a second model/tool round trip.

## 3. Feasibility and integration boundary

### 3.1 What Codex supports

Codex can connect to local or remote MCP servers. MCP is appropriate for an
explicit `render_math` tool that creates SVG, PNG, or PDF artifacts. Plugins can
bundle skills and MCP servers. Those mechanisms extend what the agent can do,
but they do not provide a general hook that transparently rewrites every
assistant message in the stock CLI transcript.

Codex also exposes an app-server protocol for building custom rich clients, and
the Codex CLI itself is open source. Both can support automatic rendering at the
presentation layer.

### 3.2 Chosen approach

The primary integration will be a small, upstreamable change to the open-source
Codex TUI. This is the only approach that provides the requested experience in
the existing interactive CLI without an explicit tool call.

The rendering implementation will live in this repository and expose a stable
library/process boundary. The Codex change should be limited to:

- detecting completed math segments in streamed assistant Markdown;
- requesting a render asynchronously;
- allocating and painting terminal cells for the result;
- preserving the original source and fallback behavior; and
- exposing configuration and diagnostics.

Until the integration is accepted upstream, users will need either a Codex build
containing the patch or an experimental `codex-latex` distribution. Installing
only an MCP server cannot fulfill the transparent-rendering requirement.

### 3.3 Alternative approaches and why they are secondary

| Approach | Automatic for every response | Keeps full Codex TUI | Maintenance | Decision |
|---|---:|---:|---:|---|
| MCP tool/plugin | No; model must call it | Yes | Low | Optional artifact tool |
| Shell wrapper around stdout | Unreliable with an interactive TUI | No | Medium | Reject |
| Custom app-server client | Yes | Must recreate client UX | Medium/high | Future option |
| Codex TUI integration | Yes | Yes | Patch until upstreamed | Primary |

## 4. Goals

The MVP must:

1. Detect common LaTeX math in streamed assistant messages automatically.
2. Render completed expressions without another model request or MCP tool call.
3. Display high-quality output in supported graphical terminals.
4. Fall back cleanly in terminals without an image protocol.
5. Preserve raw LaTeX for copy, selection, logs, history, and accessibility.
6. Never delay or fail an otherwise valid Codex response because rendering failed.
7. Match light/dark themes and reflow after terminal resizing.
8. Be safe for untrusted model-generated math.
9. Be testable independently of Codex.
10. Keep the Codex-side change narrow enough to propose upstream.

## 5. Non-goals

The MVP will not:

- implement TeX or its line-breaking algorithms from scratch;
- compile arbitrary `.tex` documents;
- run `pdflatex`, shell escape, user commands, or arbitrary package code;
- render LaTeX found inside fenced code blocks or inline code spans;
- modify model messages or stored conversation data;
- guarantee graphical output in every terminal emulator;
- add mathematical editing, symbolic algebra, or equation solving;
- provide pixel-identical output to every TeX distribution; or
- initially support the Codex IDE extension or Codex cloud UI.

Full-document PDF compilation may be added later as a separate, explicitly
invoked and sandboxed tool. It must not share the threat model of the inline math
renderer.

## 6. Users and primary scenarios

### 6.1 Primary user

A developer, student, scientist, or engineer using the interactive Codex CLI in
a terminal that supports Kitty graphics, iTerm2 images, or Sixel.

### 6.2 Scenarios

- Ask Codex to explain an algorithm and see complexity formulas typeset.
- Ask a physics or statistics question and see displayed derivations rendered.
- Receive inline variables or short formulas embedded in prose.
- Resize the terminal and have display equations scale or reflow correctly.
- Copy the original LaTeX even though the transcript shows a rendered equation.
- Use Codex over SSH or inside `tmux` and receive either working passthrough or a
  predictable text fallback.
- Diagnose a malformed expression by toggling its source and viewing a compact
  rendering error.

## 7. User experience requirements

### 7.1 Default behavior

- Math rendering is enabled when a supported graphical terminal is detected.
- Completed display math is rendered as a centered block.
- Completed inline math is rendered within the text line when its dimensions fit
  the available row. If it cannot fit legibly, it is promoted to a small block.
- While a math expression is still streaming, the TUI shows its source in a dim
  style. It is replaced when the closing delimiter arrives and rendering
  completes.
- Rendering is asynchronous. Prose outside the candidate expression continues to
  stream normally.
- A failed render leaves the original source visible and adds a compact warning
  marker; it never removes content.

### 7.2 Source access

The canonical transcript retains source text. The UI must provide:

- a command to toggle rendered/source view globally;
- a temporary source view suitable for copying;
- source text in exported transcripts and non-interactive output; and
- descriptive text for screen readers where the terminal/client supports it.

Proposed commands:

```text
/math on
/math off
/math auto
/math source
/math status
```

Names must be reconciled with existing Codex commands before implementation.

### 7.3 Configuration

Proposed configuration shape for an upstream Codex integration:

```toml
[tui.math]
enabled = true
terminal_backend = "auto"     # auto | kitty | iterm2 | sixel | text
inline_dollars = "smart"      # off | smart | always
fallback = "source"           # source | unicode
max_width_percent = 90
max_height_rows = 18
theme = "auto"                # auto | light | dark
show_errors = "compact"       # compact | detailed | source-only
```

Unknown fields, default values, and config migration must follow Codex's existing
configuration conventions. If upstream does not accept these settings, the
experimental build will store equivalent settings in its own namespaced config.

## 8. Supported syntax

### 8.1 MVP delimiters

The segmenter must recognize:

- inline `\(...\)`;
- display `\[...\]`;
- display `$$...$$`;
- single-dollar `$...$` when smart-dollar detection is enabled; and
- selected display environments, including `equation`, `equation*`, `align`,
  `align*`, `gather`, `gather*`, and `multline`.

Common nested math environments such as `aligned`, `cases`, `matrix`, `pmatrix`,
and `bmatrix` are handled by the rendering backend once inside a recognized math
segment.

### 8.2 Exclusions

The segmenter must not render:

- fenced code blocks;
- inline code spans;
- escaped delimiters;
- obvious currency values such as `$5`, `$19.99`, or `$5–$10`;
- unmatched delimiters at the end of a turn; or
- content exceeding configured length or nesting limits.

### 8.3 Smart-dollar rules

Single-dollar delimiters are ambiguous in developer conversations. Smart mode
will use conservative rules:

- an opening `$` cannot be followed only by whitespace or an ordinary currency
  number;
- a closing `$` cannot be preceded by whitespace;
- the expression cannot cross a blank line;
- an escaped dollar is literal;
- code-span and code-fence state takes precedence; and
- uncertainty resolves to source text, not rendering.

The project will maintain a corpus of prose, shell commands, prices, template
languages, and actual mathematics to measure false positives.

## 9. System architecture

```text
Assistant message deltas
          │
          ▼
  Streaming Markdown/math segmenter
          │ segments: prose | code | math | incomplete
          ▼
  Codex transcript presentation model
          │                     ┌───────────────┐
          │ completed math ───▶ │ Render client │
          │                     └───────┬───────┘
          │                             │ JSONL over stdio
          │                     ┌───────▼────────┐
          │                     │ MathJax worker │
          │                     │ TeX → SVG      │
          │                     └───────┬────────┘
          │                             │ sanitized SVG + metrics
          │                     ┌───────▼────────┐
          │                     │ Raster/cache   │
          │                     │ SVG → PNG      │
          │                     └───────┬────────┘
          ▼                             ▼
  Text/source fallback       Terminal image widget
                                      │
                       Kitty | iTerm2 | Sixel | text
```

### 9.1 Repository layout

Proposed layout:

```text
.
├── Cargo.toml
├── crates/
│   ├── latex-segmenter/       # streaming, Markdown-aware detection
│   ├── latex-render-core/     # requests, results, limits, cache keys
│   ├── latex-render-client/   # async worker lifecycle and JSONL protocol
│   ├── latex-terminal/        # capabilities, layout, terminal backends
│   └── latex-cli/             # standalone preview/diagnostic binary
├── renderer/
│   └── mathjax-worker/        # TypeScript worker producing SVG
├── integrations/
│   ├── codex/                 # patch notes, compatibility, test harness
│   └── mcp/                   # optional explicit render_math tool
├── fixtures/
│   ├── parsing/
│   ├── rendering/
│   └── transcripts/
├── docs/
│   ├── architecture/
│   ├── security.md
│   └── terminal-support.md
└── PROJECT_SPEC.md
```

The repository should not vendor the entire Codex source tree. Codex integration
will be developed in an OpenAI Codex fork/branch and tested against released
versions. This repository owns the reusable renderer and integration contract.

## 10. Component specifications

### 10.1 Streaming segmenter

The segmenter is a pure Rust state machine. It accepts arbitrary UTF-8 chunks;
chunk boundaries must not affect the final segmentation.

Required states include:

- normal prose;
- inline code with variable backtick fence length;
- fenced code with backtick or tilde fences;
- inline math;
- display math;
- recognized math environment;
- escape sequence; and
- incomplete candidate.

Required properties:

- linear time in input length;
- bounded memory;
- deterministic output;
- no regular expression with catastrophic backtracking;
- lossless reconstruction of original input;
- incremental updates suitable for streaming; and
- explicit end-of-turn flushing for unmatched syntax.

The first version should not depend on a full Markdown parser unless the target
Codex TUI already exposes a compatible incremental syntax tree. A small lexer is
easier to test against fragmented stream input.

### 10.2 Render core

The render core defines backend-neutral data structures and limits.

Conceptual Rust API:

```rust
pub struct RenderRequest {
    pub source: String,
    pub display_mode: bool,
    pub foreground: Rgba,
    pub background: Option<Rgba>,
    pub scale: f32,
    pub max_width_px: u32,
}

pub struct RenderedMath {
    pub svg: Vec<u8>,
    pub width_px: u32,
    pub height_px: u32,
    pub baseline_px: Option<f32>,
    pub accessibility_text: String,
    pub cache_key: String,
}

#[async_trait]
pub trait MathRenderer {
    async fn render(
        &self,
        request: RenderRequest,
    ) -> Result<RenderedMath, RenderError>;
}
```

Exact types may change, but Codex must depend only on this contract, not directly
on MathJax or Node.

### 10.3 MathJax worker

The MVP worker will:

1. Start once per Codex session or lazily on the first expression.
2. Read newline-delimited JSON requests from stdin.
3. Render supported TeX math to SVG with a fixed extension and macro policy.
4. Sanitize and bound the SVG.
5. Return SVG, dimensions, baseline information, warnings, and structured errors.
6. Write logs only to stderr so stdout remains a valid protocol stream.
7. Support request IDs because results may complete out of order.
8. Exit or reject work after configured resource limits are exceeded.

The worker package and MathJax version must be locked. It must not load packages,
fonts, scripts, or resources from the network at runtime.

### 10.4 Worker protocol

Request:

```json
{
  "protocol": 1,
  "id": "eq-42",
  "method": "render",
  "params": {
    "source": "x = \\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}",
    "displayMode": true,
    "foreground": "#e6edf3",
    "background": "transparent",
    "scale": 2,
    "maxWidthPx": 1200
  }
}
```

Success response:

```json
{
  "protocol": 1,
  "id": "eq-42",
  "ok": true,
  "result": {
    "svgUtf8": "<svg>...</svg>",
    "widthPx": 412,
    "heightPx": 96,
    "baselinePx": 71,
    "accessibilityText": "x equals negative b plus or minus ..."
  }
}
```

Error response:

```json
{
  "protocol": 1,
  "id": "eq-42",
  "ok": false,
  "error": {
    "code": "INVALID_TEX",
    "message": "Missing closing brace",
    "position": 14,
    "retryable": false
  }
}
```

The protocol will include a startup handshake reporting renderer version,
supported features, and hard limits.

### 10.5 Rasterization and cache

MathJax returns vector SVG. The terminal layer will rasterize it to a transparent
PNG when required by the selected terminal protocol.

Cache keys must include:

- normalized source without changing semantics;
- display/inline mode;
- renderer and protocol versions;
- enabled macro/extension policy;
- foreground and background colors;
- scale and width constraint; and
- rasterizer version.

The MVP cache is an in-memory, size-bounded LRU. A disk cache is optional after
privacy and invalidation behavior are defined. Cached content must never include
unbounded model output.

### 10.6 Terminal capability layer

Backends, in preferred order when available:

1. Kitty graphics protocol, including compatible terminals such as WezTerm.
2. iTerm2 inline image protocol.
3. Sixel.
4. Text fallback.

Capability detection must consider direct terminal use, SSH, `tmux`/`screen`
passthrough, the alternate screen, and terminals that set misleading environment
variables. Active capability probing must be optional and time-bounded.

The terminal widget must:

- reserve cells before placing an image;
- delete or replace image placements during redraw;
- survive scroll, alternate-screen restoration, and transcript reflow;
- avoid leaking escape sequences into redirected output;
- recompute layout on terminal resize;
- cap image width and height; and
- restore terminal state after errors, panics, or interrupts.

### 10.7 Text fallback

Every expression always has a source fallback. The optional Unicode fallback may
improve common constructs such as superscripts, subscripts, Greek symbols, roots,
and simple fractions, but it must not pretend to fully typeset arbitrary TeX.

Fallback order:

```text
rendered terminal image → optional Unicode approximation → original source
```

Redirected/non-TTY output always uses the original source unless the user
explicitly requests another format.

## 11. Codex TUI integration specification

### 11.1 Data flow

1. Codex receives assistant message deltas.
2. The current message's presentation layer feeds raw text to the segmenter.
3. Prose and code continue through normal Markdown rendering.
4. An incomplete math candidate remains visible as dim source.
5. When its delimiter closes, the TUI creates a stable equation item ID.
6. The async render client checks the cache and submits missing work.
7. The TUI continues processing agent events while rendering occurs.
8. A successful result invalidates only the affected transcript/layout region.
9. A failure replaces the pending state with the untouched source plus a warning.
10. Stored conversation items remain unchanged.

### 11.2 Integration rules

- No model prompting is required for correctness.
- No tool call is inserted into the conversation.
- Rendering events are local UI events and are not sent back to the model.
- A worker crash must be isolated from the Codex process. Restart at most once per
  configured interval to prevent a crash loop.
- Render cancellation follows message replacement, thread changes, and CLI exit.
- The integration must respect Codex's sandbox and approval model; displaying
  math must never generate an approval prompt.
- Non-interactive commands and JSON output formats remain byte-for-byte compatible
  unless a new explicit rendering flag is selected.

### 11.3 Version compatibility

The integration test matrix must record:

- Codex release or commit;
- Rust toolchain;
- renderer protocol version;
- target operating system; and
- terminal backend.

An adapter layer should isolate Codex transcript/layout APIs so renderer crates do
not change whenever the TUI internals move. CI will test the current supported
Codex release and the latest upstream main branch where practical.

### 11.4 Upstream contribution plan

Before submitting a large feature:

1. Open a focused Codex feature request describing the UX and terminal matrix.
2. Submit the streaming segmenter and tests independently if maintainers prefer a
   staged review.
3. Keep the renderer behind a feature flag during early review.
4. Avoid requiring Node in the default Codex binary until packaging is agreed.
5. Offer a backend trait so upstream can choose a native or separately packaged
   renderer.

If upstream rejects a bundled renderer dependency, maintain an experimental
Codex fork while continuing to publish the core renderer and protocol separately.

## 12. Optional CLI and MCP interfaces

These interfaces support testing and explicit artifact generation; they are not
the mechanism for transparent transcript rendering.

Proposed standalone commands:

```text
latex-render render --display 'E = mc^2'
latex-render render --format svg --output equation.svg '\int_0^1 x^2 dx'
latex-render check '\frac{1}{'
latex-render doctor
```

Proposed MCP tool:

```text
render_math(
  source: string,
  display_mode?: boolean,
  format?: "svg" | "png",
  theme?: "light" | "dark"
)
```

The MCP server can return the artifact and structured diagnostics. It should
advertise server-wide constraints in MCP `instructions`, including maximum input
size, no network access, and math-only support.

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
