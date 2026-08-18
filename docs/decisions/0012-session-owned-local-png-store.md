# ADR 0012: Retain local PNGs for the terminal session

Status: Accepted

Date: 2026-08-27

## Context

iTerm2 3.6 and newer implements the Kitty graphics protocol but Codex selects its
local file medium for terminal images. The Kitty protocol distinguishes simple files
from temporary files: a simple file remains client owned, while terminal deletion of
a temporary file has naming and directory requirements. With suppressed protocol
responses, deleting a newly written file immediately after writing terminal bytes
would race the emulator reading it.

The upstream Codex image implementation also uses Kitty `t=f` for iTerm2 local file
transfer. This project needs equivalent behavior for generated equation PNGs, which
do not already live in a persistent asset cache.

References:

- [Kitty graphics transmission media](https://sw.kovidgoyal.net/kitty/graphics-protocol/#the-transmission-medium)
- [OpenAI Codex terminal image protocol](https://github.com/openai/codex/blob/main/codex-rs/tui/src/pets/image_protocol.rs)

## Decision

Create one private directory under the canonical operating system temporary
directory for each terminal presentation session. Use a process identifier plus an
atomic sequence with exclusive directory creation. On Unix, create the directory as
`0700` and each file as `0600`.

Validate PNG signature and global encoded size, name files by SHA 256 content digest,
and create each file exclusively. Deduplicate identical PNGs. Enforce nonzero file
count and total byte limits without evicting files because the terminal may not have
read an older command yet. Retain all accepted files until the store drops, then
remove only its uniquely owned directory. Continue using Kitty `t=f` so ownership is
unambiguous.

## Consequences

- Local presentation does not depend on terminal acknowledgements or file read
  timing during the session.
- Identical equation rasters use one file and one capacity charge.
- Capacity exhaustion fails predictably and selects canonical source fallback.
- A process crash can leave a private temporary directory for operating system
  cleanup; a later doctor enhancement may report stale directories.
- Direct Kitty presentation creates no filesystem artifacts.

## Rejected alternatives

- Immediate client deletion was rejected because terminal write completion does not
  prove emulator file read completion.
- Kitty `t=t` was rejected for this path because terminal support and deletion
  behavior must remain consistent with the verified Codex iTerm2 implementation.
- Unbounded session storage was rejected because a long transcript could exhaust
  disk space.
- Evicting old files was rejected because suppressed acknowledgements provide no
  safe point that proves a terminal has consumed them.

