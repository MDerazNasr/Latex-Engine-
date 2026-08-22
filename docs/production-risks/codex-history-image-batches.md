# Codex history image batch production risks

## 1. Null, malformed, or invisible placement hides equation source

- Trigger: An image ID, path, dimension, row, or column is empty, duplicated,
  overflowing, outside the batch, partially scrolled off screen, or wider than the
  terminal.
- Impact: Codex could insert reserved blank rows without a corresponding visible image.
- Mitigation: Batch construction bounds image count and validates identifiers, paths,
  dimensions, uniqueness, and logical rows. A preview maps post-wrap physical rows to
  the projected viewport. Any invalid or invisible placement clears its prior image and
  inserts the preserved source rows instead of rich rows.
- Test coverage: Focused tests cover null dimensions, duplicate IDs, hidden and partial
  placements, physical URL wrapping, source row mapping, and source fallback.

## 2. Resize or insertion races publish at a stale terminal coordinate

- Trigger: Logical Markdown rows wrap differently during insertion, the viewport grows,
  or a replacement generation reuses an image ID at a new location.
- Impact: An equation could overlap unrelated history or leave its prior placement visible.
- Mitigation: The same wrapping function prepares text and placement geometry, preview
  applies the exact insertion mode and viewport growth, and publication occurs inside
  the synchronized terminal update. Reusing an ID deletes the old placement before the
  new payload is emitted.
- Test coverage: Geometry tests cover prewrap, terminal soft wrap, and viewport growth.
  Byte-order tests prove text precedes image output and deletion precedes replacement.

## 3. Async asset or terminal failure leaves a partial rich batch

- Trigger: A local image disappears, Sixel preparation fails, a writer accepts partial
  bytes, flush fails, or shutdown begins while a batch remains queued.
- Impact: Blank reservations, partial control sequences, stale images, or unflushed state
  could remain in the terminal.
- Mitigation: Every image asset is prepared before text insertion. Asset failures are
  source fallback and never emit image commands. Terminal presentation retains cleanup
  ownership until flush succeeds, queued batches stay owned until the synchronized flush,
  and TUI drop clears all committed history image states.
- Test coverage: Batch tests cover missing assets and generation replacement. Shared
  presenter tests cover short writes, writer and flush failures, cleanup, and state commit.
