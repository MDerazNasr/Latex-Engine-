# Terminal support matrix

Last validated: 2026-08-27

## Phase 2 reference results

| Terminal | Version | Transport | Automated | Real display | Cleanup | Status |
|---|---:|---|---:|---:|---:|---|
| Kitty | 0.47.0 | Kitty direct PNG | Pass | Pass | Pass | Reference |
| iTerm2 | 3.6.10 | Kitty local file | Pass | Pass | Pass | Reference |
| Ghostty | Installed | Kitty compatible | Detection only | Not run | Not run | Fallback until tested |
| WezTerm | Not installed | Kitty compatible | Detection only | Not run | Not run | Fallback until tested |
| tmux | Active test session | Disabled in Phase 2 | Pass | Not applicable | Not applicable | Source fallback |
| Redirected output | Not a TTY | Text | Pass | Not applicable | Not applicable | Source fallback |
| Zellij | Not run | Disabled in Phase 2 | Pass | Not applicable | Not applicable | Source fallback |
| Sixel terminal | Not available | Sixel | Deferred | Not run | Not run | Source fallback |

## Validation procedure

The pinned MathJax worker generated the equation through the production renderer
pipeline. The Rust `latex-terminal-smoke` binary sanitized and rasterized it, placed
it in a temporary alternate screen, replaced it at a second measured geometry,
deleted the stable image ID, and restored the cursor and screen on exit.

For each reference terminal, validation included:

1. A process test of the complete worker and escape sequence lifecycle.
2. A real application run on macOS.
3. Visual inspection while the equation was present and after cleanup.

Kitty received visual inspection before and after the generation checked replacement
to confirm that the prior placement disappeared. Both reference terminals received
visual inspection after completion to confirm that no image remained and the normal
screen and cursor returned.

Screenshots remain temporary because they contain machine-local terminal chrome
and are not needed as runtime fixtures.

## Known limitations

- Phase 2 disables tmux, GNU Screen, and Zellij rather than attempting passthrough.
- Active capability probing is not implemented yet.
- Sixel is deferred to synchronized Codex redraw in Phase 3.
- The smoke tool requires explicit cell and pixel geometry because independent
  terminal queries are deferred to the Codex event loop.
