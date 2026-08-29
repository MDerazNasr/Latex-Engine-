# Experimental Codex integration guide

## What this build does

`codex-latex` is an experimental Codex TUI that recognizes supported LaTeX math in
assistant Markdown, renders it through the local `latex-render` daemon, and presents
the resulting terminal image in place of the source syntax. The canonical assistant
text remains the source of truth for history, copying, reflow, fallback, and session
restore.

This MVP does not replace the normal `codex` command. The installer creates only:

```text
<prefix>/bin/codex-latex
<prefix>/bin/latex-render
<prefix>/share/latex-render/mathjax-worker
<prefix>/libexec/codex-latex/<version>-<target>/...
```

All three visible paths are symbolic links owned by one verified, versioned bundle.

## Requirements

- macOS or Linux on the same target architecture as the bundle.
- Node.js 22 or newer available at runtime.
- Rust 1.85 or newer and Corepack when building from source.
- A clean renderer checkout.
- A clean Codex checkout containing pinned upstream commit
  `b68acc4d4b56fdfa1d5b6a2c36102c66876e0c46` and final integration commit
  `460c176304`.
- A supported direct terminal for images. Unsupported terminals still show source.

The source builder uses frozen pnpm dependencies and locked Cargo builds. It refuses
dirty checkouts, missing commit ancestry, invalid versions, and existing output
directories.

## Build a bundle

Run from the renderer repository:

```sh
cargo run -p codex-latex-package -- build \
  --engine-root /absolute/path/to/Latex-Engine- \
  --codex-checkout /absolute/path/to/codex-latex-integration \
  --output /absolute/path/to/codex-latex-0.1.0-aarch64-apple-darwin \
  --version 0.1.0
```

The output is immutable and contains the two release binaries, compiled worker,
MathJax, its locked runtime font, and `manifest-v1.json`. Every regular file is
listed with its byte count, executable bit, and SHA-256 digest. The manifest is
published only after staging succeeds.

The output path must not already exist. Choose a target suffix reported by
`rustc -vV`; the builder records the host value automatically.

## Install without replacing Codex

Choose an explicit writable prefix and run:

```sh
cargo run -p codex-latex-package -- install \
  --bundle /absolute/path/to/codex-latex-0.1.0-aarch64-apple-darwin \
  --prefix /absolute/install/prefix
```

Installation verifies the source manifest, copies into an isolated version root,
verifies the installed copy, and then activates the three links. It refuses to
replace any existing path. In particular, `<prefix>/bin/codex` is never created,
changed, or removed.

Validate and launch the installed build:

```sh
/absolute/install/prefix/bin/latex-render check
/absolute/install/prefix/bin/codex-latex
```

`latex-render check` must report `status=ok`, `protocol=1`, and
`worker_state=ready`. The first launch of Codex may ask whether the working directory
is trusted, exactly as the normal Codex CLI does.

## Natural math behavior

Ask an ordinary question that causes Codex to answer with delimited math. Supported
forms include `\(...\)`, `\[...\]`, `$$...$$`, and smart `$...$`. Smart dollar
parsing avoids common currency text. Math inside code spans, fenced code blocks,
links, and other protected Markdown remains source text.

Rendering is asynchronous and bounded. Source remains visible while an expression
is incomplete or pending. A completed image replaces only its own equation region.
If parsing, rendering, rasterization, terminal presentation, resizing, or process
supervision fails, Codex remains usable and the canonical source stays visible.

Redirected and other noninteractive output is unchanged. Rendering affects only the
interactive TUI presentation layer.

## Slash commands

The presentation controls are session local and do not edit the stored transcript:

| Command | Effect |
| --- | --- |
| `/math status` | Show feature, terminal, backend, daemon, task, error, and bound diagnostics without exposing equation source. |
| `/math source` | Toggle canonical LaTeX source for copying. |
| `/math off` | Reveal source for the current session. |
| `/math on` | Restore configured rendered presentation. |
| `/math auto` | Restore configured rendered presentation. |

Run `/math status` first when an equation remains as source. The most useful fields
are `feature`, `presentation`, `reason`, `selected_backend`, `fallback_reason`,
`worker_state`, `worker_restarts`, `last_error`, `renderer_source`, and
`worker_source`.

## Runtime settings

The private MVP adapter uses bounded environment settings so it does not expand the
upstream Codex configuration contract. Invalid values fail closed and keep source
visible.

| Variable | Values and default |
| --- | --- |
| `CODEX_LATEX_RENDER` | `auto` by default; `on`, `off`, `true`, or `false` are accepted. |
| `CODEX_LATEX_RENDERER` | Renderer path; otherwise discover a bundled sibling, then `PATH`. |
| `CODEX_LATEX_RENDER_WORKER` | Optional worker script override; packaged discovery is the default. |
| `CODEX_LATEX_RENDER_NODE` | Optional Node executable override; `node` from `PATH` is the default. |
| `CODEX_LATEX_BACKEND` | `auto` by default; `kitty`, `iterm2`, `sixel`, or `text`. |
| `CODEX_LATEX_INLINE_DOLLARS` | `smart` by default; `off` or `always`. |
| `CODEX_LATEX_THEME` | `auto` by default; `light` or `dark`. |
| `CODEX_LATEX_SCALE` | `2` by default; from `0.5` through `4`. |
| `CODEX_LATEX_MAX_WIDTH_PERCENT` | `90` by default; from `1` through `100`. |
| `CODEX_LATEX_MAX_HEIGHT_ROWS` | `18` by default; from `1` through `64`. |
| `CODEX_LATEX_QUEUE_CAPACITY` | `4` by default; from `1` through `32`. |
| `CODEX_LATEX_REQUEST_TIMEOUT_MS` | `3000` by default; from `100` through `30000`. |
| `CODEX_LATEX_SHUTDOWN_TIMEOUT_MS` | `500` by default; from `50` through `5000`. |
| `CODEX_LATEX_RESTART_INTERVAL_MS` | `5000` by default; from `100` through `60000`. |

Protocol v1 also limits each message to 256 KiB, each message to 32 equations, each
PNG to 4 MiB, and the total PNG payload per response to 8 MiB.

## Terminal behavior

Automatic detection selects:

- Kitty graphics for Kitty, Ghostty, and WezTerm.
- Kitty local-file graphics for iTerm2 3.6 or newer.
- Sixel for detected Windows Terminal, `mlterm`, `foot`, or explicit Sixel terminals.
- Text fallback for unsupported terminals, old iTerm2, `tmux`, and Zellij.

SSH does not bypass terminal validation. Multiplexers intentionally fall back to
source because passthrough was not accepted for MVP. Force a backend only for
compatibility testing; a false-positive terminal can display escape noise.

## Troubleshooting

### Feature reports `missing_renderer`

Launch the installed `codex-latex`, keep `latex-render` beside it, or set
`CODEX_LATEX_RENDERER` to a nonempty regular file. Check that no stale override
points to a deleted development build.

### Health check cannot find the worker

Confirm `<prefix>/share/latex-render/mathjax-worker` is still the installer-owned
link and that Node.js 22 or newer is on `PATH`. Reinstall rather than editing the
version root or activation link.

### Worker is degraded or restarts increase

Run the installed `latex-render check`. Use `/math status` to distinguish discovery,
timeout, protocol, and terminal failures. The supervisor rate-limits restarts and
preserves source while unavailable.

### Equations remain as text

Check `presentation`, `selected_backend`, and `fallback_reason`. `/math on` restores
rendered presentation after source mode. Undelimited TeX, protected Markdown, and
unsupported terminal sessions intentionally remain text.

### Rendering looks too large or clipped

Adjust the bounded scale, width percentage, or height rows before launch. Resizing
the terminal invalidates stale work and schedules a generation-checked rerender.

## Roll back

Use the same renderer checkout and the exact installed prefix:

```sh
cargo run -p codex-latex-package -- uninstall \
  --prefix /absolute/install/prefix
```

Rollback revalidates the complete installed manifest and confirms all activation
links still identify one version root. It refuses removal if a file is missing,
changed, linked, or unowned, or if an activation link was redirected. After a clean
rollback it removes only the owned links, files, manifest, and empty installation
directories. The normal `codex` command and unrelated prefix contents remain.

## Standalone diagnostics

The renderer can be exercised independently of Codex:

```sh
/absolute/install/prefix/bin/latex-render doctor
/absolute/install/prefix/bin/latex-render render \
  --display --format png --output equation.png 'E = mc^2'
```

`doctor` includes terminal capability information. `render` never overwrites an
existing output unless `--force` is explicit.
