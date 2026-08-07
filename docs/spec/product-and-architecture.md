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
│   ├── latex-render-svg/      # SVG allowlist and deterministic PNG rasterization
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
