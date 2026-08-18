# End to end terminal smoke production risks

This note records the required failure prediction for the Phase 2 acceptance tool.

## 1. Missing or malformed source and terminal geometry

Trigger: source is empty, cells or pixels are zero, dimensions overflow their public
types, resize geometry lacks an initial measurement, or duplicate options disagree.

Impact: layout could divide by zero, reserve the wrong cells, or place control
sequences outside the visible screen.

Mitigation: strict parsing rejects empty, duplicate, malformed, oversized, and
incoherent values. Image modes require explicit whole terminal cell and pixel
measurements, which pass through the validated layout constructor.

Test coverage: argument tests cover defaults, all valid options, null source, zero,
malformed and oversized geometry, duplicates, missing pairs, and source delimiters.

## 2. Resize or backend state races an earlier frame

Trigger: the optional second geometry changes cell dimensions while the first image
is active, or a terminal backend changes around a generated frame.

Impact: old pixels could remain, the image could overwrite prose, or replacement
could use the wrong protocol.

Mitigation: every smoke frame begins a new presenter generation and carries exact
layout through raster completion. Replacement encodes targeted prior image deletion
before the new image, while automatic backend selection happens before work starts.

Test coverage: process tests publish two Kitty geometries and assert two transfers
plus repeated targeted cleanup. Presenter tests independently cover stale and backend
change completion.

## 3. Worker, raster, local file, wait, or terminal output fails asynchronously

Trigger: worker startup, render, shutdown, rasterization, file creation, protocol
encoding, timer, write, or flush fails; or the process unwinds after screen entry.

Impact: source may disappear, a local file may leak, or cursor and alternate screen
state may remain altered.

Mitigation: rendering and first frame preparation complete before screen entry. Any
failure prints canonical source. The screen guard owns cursor visibility, targeted
deletion, alternate screen restoration, and best effort flush. The local store drops
after the screen guard and removes only its session directory.

Test coverage: process tests inject worker failure before screen entry, verify source
and absence of escapes, run real fake-worker render and raster for both transports,
and assert cleanup and restoration. Phase 4 writer fault injection will cover partial
write and flush failures.

