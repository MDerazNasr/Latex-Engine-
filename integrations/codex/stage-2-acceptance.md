# Codex integration stage 2 acceptance

## Scope

Stage 2 moves Kitty direct transfer, Kitty local-file transfer, targeted Kitty
deletion, tmux passthrough, Sixel frame generation, and the deterministic Sixel
encoder into the private Codex `terminal_image_v2` layer.

The transition is split into three independently reversible Codex commits:

1. `ec2ee859c0` adds and verifies `sixel_v2` and `transport_v2` while the working
   pet implementation remains unchanged.
2. `0b61eac381` routes the pet adapter through the verified shared transport while
   retaining the old Sixel encoder only for equivalence tests.
3. `458d76ca91` removes the redundant pet Sixel implementation after both paths
   pass.

## Verification

- All 18 generic terminal-image tests pass after cleanup. They cover capabilities,
  targeted deletion, direct and chunked Kitty payloads, local-file references,
  tmux escaping, missing files, deterministic Sixel bytes, transparent pixels,
  multiple bands, run-length encoding, and malformed RGBA buffers.
- All eight pet image-protocol tests pass through the new implementation.
- Before deletion, all five legacy Sixel tests also pass against the retained old
  module.
- The broad `codex-tui` gate passes all 3,980 selected tests with eight skipped by
  the profile. Nextest reports one unrelated startup-draft test that passed on
  retry and one nonfailing leaky-test marker.
- The two hidden-paste tests remain excluded by the known Stage 1 filter. Their
  isolated result remains one pass and one unrelated macOS shortcut snapshot
  mismatch, with no snapshot update accepted.
- Scoped Rust formatting, `git diff --check`, and `just fix -p codex-tui` pass.
- No dependency or lock file changed. Every new or extracted implementation and
  test file remains below 500 lines.

## Production failure prediction

1. A missing path, empty file, malformed PNG, or zero geometry can yield an error
   or an unusable terminal command. Existing file errors remain contextual; the
   future presentation boundary must validate PNG bytes and nonzero measured
   placement before calling transport.
2. Two writers can observe the same missing Sixel cache entry and publish it
   concurrently, exposing a partial file to a reader. The current pet flow is
   serialized; any shared asynchronous Sixel caller must use atomic publication or
   bypass this cache.
3. A later async render or terminal write can fail, time out, or complete after its
   image generation is stale. This transport is synchronous and returns every I/O
   error; the future controller must own timeout, join, cancellation, partial-write,
   deletion, and generation-check paths.

## Self-review

Dependency flow is one way from pets to `terminal_image_v2`, with transport depending
only on its sibling Sixel encoder. No circular dependency remains. The old encoder
and private tmux helper were removed after their replacement tests passed, so there
is no redundant production implementation. The migration initially exposed hidden
test imports for environment, filesystem, and Base64 helpers; those dependencies
are now explicit. Error messages and pet-facing output remain compatible, while
async lifecycle work remains intentionally outside this synchronous extraction.

