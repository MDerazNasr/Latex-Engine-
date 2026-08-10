# Codex LaTeX Renderer

Render LaTeX mathematics automatically inside the Codex CLI instead of showing
raw delimiters and commands.

The independent Phase 1 renderer is implemented and under validation. It includes a
streaming Markdown-aware segmenter, supervised MathJax worker, fail-closed SVG
boundary, deterministic PNG rasterizer, bounded cache, terminal protocol spikes, and
the standalone `latex-render` CLI. Automatic Codex transcript integration remains a
later phase.

See [PROJECT_SPEC.md](PROJECT_SPEC.md) for the normative architecture, Codex
integration strategy, security model, milestones, and acceptance criteria.

The key architectural decision is that transparent rendering belongs in the
Codex terminal presentation layer. An MCP tool can render an equation when the
model explicitly calls it, but cannot reliably post-process every assistant
message. This project therefore consists of:

1. An independent, reusable math parser and renderer.
2. A small integration with the open-source Codex CLI TUI.
3. Optional CLI/MCP adapters for testing and explicit artifact generation.

## Development setup

The current development build requires Rust 1.85 or newer, Node.js 22 or newer, and
Corepack. Build the locked MathJax worker first:

```sh
cd renderer/mathjax-worker
corepack pnpm install --frozen-lockfile
corepack pnpm build
cd ../..
```

Validate the complete local pipeline:

```sh
cargo run -p latex-cli -- check
```

Render an equation to a new SVG or PNG artifact:

```sh
cargo run -p latex-cli -- render --display --output equation.svg 'E = mc^2'
cargo run -p latex-cli -- render --display --format png --output equation.png '\int_0^1 x^2\,dx'
```

Existing output files are preserved unless `--force` is explicit. Run
`cargo run -p latex-cli -- render --help` for all bounded rendering options.

## Tests

From the repository root:

```sh
cargo test --workspace --all-targets
```

From `renderer/mathjax-worker`:

```sh
corepack pnpm format
corepack pnpm check
corepack pnpm test
```
