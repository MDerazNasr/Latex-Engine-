# Codex LaTeX Renderer 0.1.0

Version 0.1.0 is the first experimental developer release of automatic LaTeX math
presentation for the interactive Codex CLI.

## Included

- Markdown-aware TeX segmentation with canonical source preservation.
- A supervised, versioned MathJax protocol over a local Node.js worker.
- Fail-closed SVG sanitization, deterministic PNG rasterization, bounded caching,
  and theme-aware output.
- Cell-aligned Kitty, iTerm2, and Sixel transcript presentation with resize replay.
- A narrow Codex TUI adapter with bounded asynchronous scheduling, history restore,
  source controls, and source-free `/math status` diagnostics.
- An independent `latex-render` CLI for render, check, doctor, and daemon workflows.
- A hashed `codex-latex` developer bundle with explicit installation and rollback.

## Compatibility

The Codex adapter is based on upstream commit
`b68acc4d4b56fdfa1d5b6a2c36102c66876e0c46` and ends at integration commit
`460c176304`. Bundles require Node.js 22 or newer at runtime. Packaging currently
supports macOS and Linux; Windows packaging is deferred.

Direct Kitty-family terminals and iTerm2 3.6 or newer receive rich images. Detected
Sixel terminals use the existing Codex-owned redraw path. Unsupported terminals,
multiplexers, redirected output, and any renderer failure preserve source text.

## Known limits

- The integration is an experimental Codex branch rather than an upstream feature.
- Node.js is external and is not embedded in the bundle.
- `tmux` and Zellij passthrough are disabled for MVP.
- Runtime settings use private environment variables pending an upstream config
  contract.
- Protocol v1 diagnostics do not yet report the worker protocol, renderer version,
  or cache telemetry through the Codex status view.

Installation, controls, troubleshooting, and rollback are documented in
[codex-integration.md](codex-integration.md).
