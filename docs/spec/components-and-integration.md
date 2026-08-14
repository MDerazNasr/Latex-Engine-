# Component and Integration Specification

This document is normative and is part of the specification indexed by
[`PROJECT_SPEC.md`](../../PROJECT_SPEC.md).

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

The shared `latex-render-svg` boundary validates and rewrites worker SVG through an
explicit element, attribute, and value allowlist before any cache or rasterizer sees
it. Rasterization uses a pinned native library with external images, system fonts,
compressed SVG, and runtime resource loading disabled. The synchronous rasterizer is
called from a bounded blocking task by asynchronous integrations.

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

Passive Phase 2 detection selects direct Kitty transfer for known Kitty, WezTerm,
and Ghostty sessions, including SSH because bytes remain on the TTY. It selects local
file transfer only for iTerm2 3.6 or newer on the same host. iTerm2 over SSH, older or
malformed iTerm2 versions, redirected output, `tmux`, Zellij, GNU Screen, and unknown
terminals select source text with a stable fallback reason. Empty environment values
do not count as detected facts. Backend and fallback reason expose stable lowercase
diagnostic names.

The terminal widget must:

- reserve cells before placing an image;
- delete or replace image placements during redraw;
- survive scroll, alternate-screen restoration, and transcript reflow;
- avoid leaking escape sequences into redirected output;
- recompute layout on terminal resize;
- cap image width and height; and
- restore terminal state after errors, panics, or interrupts.

Phase 2 layout uses measured terminal columns, rows, pixel width, and pixel height.
Zero or unavailable pixel measurements select source fallback instead of guessing a
cell aspect ratio. Inline math is uniformly scaled down to at most one cell row and
remains inline only when it fits the columns left on the current row. Other inline
math is promoted to a centered block. Display math is always a centered block capped
by `max_width_percent`, `max_height_rows`, and the visible viewport.

The raster canvas exactly matches the reserved cell rectangle in measured pixels.
Equation content is uniformly scaled without upscaling and placed on that transparent
canvas, so Kitty `c,r` placement cannot distort its aspect ratio. Inline content uses
the MathJax baseline to align with the text baseline when available; block content is
centered in both axes. A terminal column, row, cell-pixel, theme, or policy change
creates a new layout generation and invalidates older raster and placement results.

The layout contract returns presentation mode, reserved cells, pixel canvas, content
rectangle, baseline, and horizontal placement. It never writes terminal control
sequences; synchronized placement remains the caller's responsibility.

The presentation adapter owns a monotonically increasing generation and active image
state. Starting a render returns an immutable job that contains generation, backend,
placement identity, row, layout, and the fitted raster request. Raster completion
must carry the same job back to the adapter. Publication succeeds only when its
generation and backend are still current and the PNG dimensions equal the reserved
canvas. Stale completion emits no terminal bytes. A valid completion is converted to
the backend specific source and passed through the deterministic placement state so
replacement deletes the prior image before drawing the new one. Cancellation,
backend changes, and explicit fallback invalidate pending jobs and expose cleanup
bytes without suppressing the canonical source.

The iTerm2 Kitty implementation uses local file transmission. Generated PNG files
live in one private session directory beneath the operating system temporary
directory. The store uses content addressed names, exclusive creation, bounded file
count and total bytes, and retains every published file until terminal presentation
shuts down. It removes only its uniquely created directory on drop. Creation,
capacity, write, or validation failure selects source fallback. Direct Kitty
transmission never creates a local file.

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

### 10.8 Standalone CLI

The Phase 1 binary is named `latex-render` and exposes two commands:

```text
latex-render render [OPTIONS] [SOURCE]
latex-render check [OPTIONS]
```

`render` reads one positional source value or, when the value is absent and stdin is
redirected, reads bounded UTF 8 input from stdin. It supports `--display`,
`--format svg|png`, `--output PATH|-`, `--foreground #RRGGBB`,
`--background transparent|#RRGGBB`, `--scale`, and `--max-width`. Output defaults to
raw SVG on stdout. A file that already exists is preserved unless `--force` is
explicitly supplied.

`check` starts the worker, validates its handshake, renders a fixed source-free
smoke expression through the Rust sanitizer, rasterizes the result, and reports
protocol, renderer, sanitizer, rasterizer, health, and active limit values as stable
key-value lines. Terminal capability diagnostics remain the Phase 2 `doctor`
command.

Both commands accept `--worker PATH` and `--node PROGRAM`. Worker discovery checks an
explicit path first, then `LATEX_RENDER_WORKER`, then packaged paths relative to the
binary, then the development repository path. Arguments are passed directly to the
worker process without shell interpretation.

The CLI writes diagnostics only to stderr, never includes equation source in an
error, and reserves stdout for command output. Exit status 2 identifies usage or
input errors, 3 identifies worker configuration or lifecycle errors, 4 identifies
render, sanitizer, or raster errors, 5 identifies output errors, and 6 identifies an
unexpected internal task failure.

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
latex-render check
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
