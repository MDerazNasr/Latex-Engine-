# ADR 0013: Prefer safe passive terminal capability detection

Status: Accepted

Date: 2026-08-27

## Context

Terminal image protocols are not represented consistently by terminfo, and active
probe replies can arrive after a TUI has resumed normal input. Environment markers
are imperfect, but redirected output, multiplexers, remote sessions, and known
terminal programs provide a deterministic safe baseline. Local file transfer cannot
cross an SSH boundary, while direct image bytes remain on the terminal stream.

The current OpenAI Codex terminal image implementation uses the same iTerm2 3.6
minimum and selects local file transport for it. Kitty documentation separately
warns that file transfer modes do not work when the terminal is remote.

References:

- [OpenAI Codex terminal image detection](https://github.com/openai/codex/blob/main/codex-rs/tui/src/pets/image_protocol.rs)
- [Kitty image transfer mode guidance](https://sw.kovidgoyal.net/kitty/kittens/icat/#cmdoption-kitten-icat-transfer-mode)

## Decision

Use a total passive detector for Phase 2. Redirected output, tmux, Zellij, GNU Screen,
unsupported terminals, malformed or older iTerm2 versions, and iTerm2 local file
transport over SSH choose source text. Known Kitty, WezTerm, and Ghostty sessions use
direct PNG bytes even across SSH. Local iTerm2 3.6 or newer uses Kitty local files.

Treat empty environment values as absent. Return stable lowercase backend and
fallback names for configuration and doctor output. Do not send active probes until
the Phase 3 event loop can consume, correlate, and time bound terminal replies.

## Consequences

- Unsupported and redirected environments never receive image control sequences.
- Remote iTerm2 sessions lose images instead of receiving inaccessible file paths.
- Some capable but unidentified terminals remain on source fallback.
- Detection is deterministic and side effect free, which makes doctor output and
  integration tests repeatable.

## Rejected alternatives

- Assuming every true color terminal supports images was rejected because color and
  graphics capabilities are unrelated.
- Using iTerm2 local files across SSH was rejected because the emulator cannot read
  the remote process filesystem.
- Unbounded active probing was rejected because late replies can leak into user input
  and corrupt the transcript.

