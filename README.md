# Codex LaTeX Renderer

Render LaTeX mathematics automatically inside the Codex CLI instead of showing
raw delimiters and commands.

The independent renderer, terminal presentation core, and experimental Codex TUI
integration are implemented. They include a streaming Markdown-aware segmenter,
supervised MathJax worker, fail-closed SVG boundary, deterministic PNG rasterizer,
bounded cache, cell-aware layout, Kitty, iTerm2, and Sixel presentation, capability
detection, source-preserving transcript integration, and the standalone
`latex-render` CLI.

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

Validate the pipeline and report terminal image support:

```sh
cargo run -p latex-cli -- doctor
```

Run the complete terminal presentation smoke path with measured cell and pixel
geometry. Forced backends are intended for compatibility testing:

```sh
cargo run -p latex-terminal-smoke -- \
  --backend kitty \
  --geometry 120x40@1200x800 \
  --resize-geometry 80x30@800x600
```

Use `--backend iterm2` for iTerm2 3.6 or newer. Automatic mode preserves source text
when output is redirected or the detected terminal is unsupported.

Render an equation to a new SVG or PNG artifact:

```sh
cargo run -p latex-cli -- render --display --output equation.svg 'E = mc^2'
cargo run -p latex-cli -- render --display --format png --output equation.png '\int_0^1 x^2\,dx'
```

Existing output files are preserved unless `--force` is explicit. Run
`cargo run -p latex-cli -- render --help` for all bounded rendering options.

## Experimental Codex build

The versioned bundle keeps the experimental command separate from the normal
`codex` command. It requires this renderer checkout and a clean Codex checkout that
contains the final Phase 3 integration commit `460c176304`:

```sh
cargo run -p codex-latex-package -- build \
  --engine-root /absolute/path/to/Latex-Engine- \
  --codex-checkout /absolute/path/to/codex-latex-integration \
  --output /absolute/path/to/codex-latex-0.1.0-aarch64-apple-darwin \
  --version 0.1.0
```

Install under an explicit prefix, validate the installed pipeline, and launch the
experimental TUI:

```sh
cargo run -p codex-latex-package -- install \
  --bundle /absolute/path/to/codex-latex-0.1.0-aarch64-apple-darwin \
  --prefix /absolute/install/prefix

/absolute/install/prefix/bin/latex-render check
/absolute/install/prefix/bin/codex-latex
```

Inside Codex, `/math status` reports discovery and renderer health, while
`/math source` reveals canonical LaTeX for copying. See
[docs/codex-integration.md](docs/codex-integration.md) for the full installation,
configuration, terminal support, troubleshooting, and rollback guide. The exact
release evidence and remaining experimental limits are recorded in
[integrations/codex/phase-3-acceptance.md](integrations/codex/phase-3-acceptance.md).

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

Run the reviewed real-worker snapshots and release performance gate from the
repository root:

```sh
cargo test -p latex-render-client --test snapshots -- --ignored --exact rendering_corpus_matches_reviewed_snapshots
cargo run --release -p latex-bench
```

Snapshot replacement is intentionally separate and requires
`UPDATE_LATEX_SNAPSHOTS=1`; inspect every SVG, PNG, and manifest diff before commit.
