# Codex integration stage 1 acceptance

## Scope

Stage 1 extracts terminal capability detection from the pet-specific implementation
at Codex commit `b68acc4d4b56fdfa1d5b6a2c36102c66876e0c46`. The implementation commit in the
experimental Codex checkout is `78467f2a55`.

The new private `terminal_image_v2` layer owns protocol selection and unsupported
reasons. The existing pet adapter keeps its configuration override, user-facing
messages, transport functions, and rendering behavior.

## Verification

- The six generic capability tests pass, including malformed versions,
  multiplexer precedence, and every terminal-identification case retained from the
  pet suite.
- All nine pet protocol tests pass, covering automatic and explicit selection,
  Kitty direct and local-file commands, tmux escaping, Sixel encoding, and the
  existing iTerm2 message.
- A broad `codex-tui` run passes all 3,974 selected tests. The filter excludes two
  tests sharing the hidden-paste substring; an isolated run confirms the plan-mode
  case passes and the composer case has a pre-existing macOS shortcut-label
  snapshot mismatch.
- The unrelated composer snapshot expects `⌥ + ↑` while the current macOS build
  produces `shift + ←`. Its generated candidate was removed and the upstream
  snapshot was not changed.
- Scoped Rust formatting completed. Repository-wide formatting was not authorized
  because it could rewrite unrelated upstream files.
- `just fix -p codex-tui` completed without warnings or fixes.
- `git diff --check` is clean. Every new or extracted implementation and test file
  is below 500 lines.

## Production failure prediction

1. Missing, malformed, or spoofed terminal fields can select an unsupported image
   protocol. Detection rejects malformed iTerm2 versions, gives multiplexer safety
   precedence, and keeps unknown terminals unsupported.
2. A terminal or multiplexer can change after synchronous detection and before a
   later image write. The future controller must bind the selected backend to a
   generation and reject work after capability changes.
3. Future asynchronous probes can time out, panic, or leave an unobserved task.
   Stage 1 deliberately performs no asynchronous work; the supervisor stage must
   own every task, timeout, and join path before adding probes.

## Self-review

Dependency flow is one way from the pet adapter to `terminal_image_v2`, so the
extraction introduces no circular dependency. The pet-specific unsupported enum is
not redundant because it preserves product wording while the generic enum remains
feature neutral. No error or async path moved in this stage. Review initially found
that several terminal variants had been compressed out of the extracted tests; all
original variants were restored before the final test and lint gates.

