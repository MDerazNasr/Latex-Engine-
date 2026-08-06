# Terminal support matrix

Last validated: 2026-08-26

## Phase 0 reference results

| Terminal | Version | Transport | Automated | Real display | Cleanup | Status |
|---|---:|---|---:|---:|---:|---|
| Kitty | 0.47.0 | Kitty direct PNG | Pass | Pass | Automated | Reference |
| iTerm2 | 3.6.10 | Kitty local file | Pass | Pass | Visual | Reference |
| Ghostty | Installed | Kitty compatible | Detection only | Not run | Not run | Fallback until tested |
| WezTerm | Not installed | Kitty compatible | Detection only | Not run | Not run | Fallback until tested |
| tmux | Active test session | Disabled in Phase 0 | Pass | Not applicable | Not applicable | Source fallback |
| Redirected output | Not a TTY | Text | Pass | Not applicable | Not applicable | Source fallback |
| Zellij | Not run | Disabled in Phase 0 | Pass | Not applicable | Not applicable | Source fallback |
| Sixel terminal | Not available | Sixel | Not implemented | Not run | Not run | Future |

## Validation procedure

The pinned MathJax worker generated the quadratic formula fixture. The Rust
`terminal-spike` binary displayed it in a temporary alternate screen, then
deleted the stable image ID and restored the cursor and screen on exit.

For each reference terminal, validation included:

1. A process test of the complete escape sequence lifecycle.
2. A real application run on macOS.
3. Visual inspection while the equation was present.

iTerm2 also received visual inspection after completion to confirm no stale
placement. Kitty used a window that deliberately preserved the last child screen,
so its cleanup result comes from the automated lifecycle test.

Screenshots remain temporary because they contain machine-local terminal chrome
and are not needed as runtime fixtures.

## Known limitations

- Phase 0 disables tmux and Zellij rather than attempting passthrough.
- Active capability probing is not implemented yet.
- Sixel encoding is not implemented in this repository yet.
- The checked PNG uses Quick Look only for feasibility; the production rasterizer
  will replace it.
