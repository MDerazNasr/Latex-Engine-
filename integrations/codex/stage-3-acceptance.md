# Codex integration stage 3 acceptance

## Scope

Stage 3 extracts image presentation state, cursor preservation, deletion ordering,
Sixel clearing, partial-write handling, and writer flushing into the private Codex
`terminal_image_v2` layer.

The transition is split into three focused Codex commits:

1. `94386997f8` adds a versioned presenter and verifies it while the working pet
   presenter remains unchanged.
2. `69d8f647ec` migrates pet drawing to shared state and preserves the pet-specific
   public error boundary.
3. `9b4b579942` moves pet presenter tests to a sibling module after migration.

The non-obvious state rule is deliberate: new cleanup ownership commits only after
the terminal flush succeeds. Asset, write, and flush failures retain the previous
state so a later call can retry cleanup rather than forgetting a visible image.

## Verification

- All ten generic presentation tests pass. They cover null cleanup, Kitty direct and
  local-file output, Sixel reservation clearing, protocol transitions, cursor save
  and restore ordering, short writes, asset failures, writer failures, flush
  failures, and complete state equality.
- All eleven matching pet and adjacent lifecycle tests pass through the migrated
  presenter.
- The broad `codex-tui` gate passes all 3,990 selected tests with eight skipped by
  the profile and no flaky result.
- The two hidden-paste tests remain excluded by the documented Stage 1 macOS
  shortcut snapshot filter.
- Scoped Rust formatting, `git diff --check`, and `just fix -p codex-tui` pass.
- Clippy made no changes. No dependency, lock, core, protocol, or app-server file
  changed.
- `pets/mod.rs` is reduced to 185 lines and its sibling test file is 225 lines. The
  generic presenter and its tests are 203 and 296 lines.

## Production failure prediction

1. A null request correctly means cleanup, but an empty asset or zero geometry can
   still produce an unusable command. The renderer adapter must validate decoded
   PNG content and nonzero measured placement before constructing a draw request.
2. Concurrent callers could reuse an image identifier or interleave writes from
   separate states. The API requires exclusive mutable state and a synchronized
   writer; the future controller must allocate stable feature-specific identifiers
   and publish only inside the TUI terminal lock.
3. An async render can finish after resize, replacement, disable, or shutdown, while
   a terminal can accept bytes and then fail its flush. The synchronous presenter
   preserves old cleanup ownership on every reported error; the future async owner
   must generation-check before entry and retry or invalidate cleanup after failure.

## Self-review

Dependency flow remains one way: pets convert their draw request into the generic
request, and presentation depends only on sibling transport. There is no circular
dependency or redundant production presenter. Pet error wording and variants remain
stable through an explicit conversion. Review found and removed unused raw transport
reexports, then moved inline pet tests to the upstream-required sibling layout. The
remaining integration gap is intentional: renderer supervision and generation
validation do not belong in this synchronous terminal layer and begin in the daemon
and controller stages.

