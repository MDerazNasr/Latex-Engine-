# Local PNG store production risks

This note records the required failure prediction for session owned local image
files.

## 1. Null, malformed, oversized, or over capacity input reaches the filesystem

Trigger: a caller supplies empty bytes, bytes without a PNG signature, a payload over
the raster byte limit, zero store limits, too many distinct equations, or arithmetic
overflow while charging total bytes.

Impact: invalid files could be exposed to the terminal, or a long transcript could
consume unbounded memory and disk space.

Mitigation: constructors reject zero limits. Storage validates signature and global
PNG size, uses checked byte accounting, deduplicates by SHA 256, and rejects work
before file creation when either configured capacity would be exceeded.

Test coverage: integration tests cover malformed bytes, zero limits, total byte and
file count capacity, deduplication, and absence of partial files after rejection.

## 2. Concurrent sessions or external file changes corrupt correlation

Trigger: two processes choose the same directory or content name, a partial write is
observed, or another same user process changes a retained file between presenter
validation and terminal consumption.

Impact: the wrong equation could display, a decoder could see partial content, or a
session could overwrite another session's data.

Mitigation: directory identifiers combine process identity and an atomic sequence,
and both directories and content files use exclusive creation. Files are flushed and
closed before sources are returned. Unix permissions restrict access to the current
user, and the presenter compares local bytes with its correlated raster immediately
before encoding. SHA 256 names make accidental content collision impractical.

Test coverage: tests create simultaneous distinct stores, verify exact round trip
bytes and content reuse, exercise local presenter publication, and verify Unix
directory and file permissions.

## 3. Directory creation, partial writes, or cleanup fails asynchronously

Trigger: the temporary directory is missing or unwritable, disk space is exhausted,
a write or flush fails, the process crashes, or shutdown cannot remove a file.

Impact: rendering falls back to source, a partial file could remain, or a private
session directory could leak after abnormal termination.

Mitigation: creation and write failures are typed. Failed writes close and remove the
exact partial target before returning. The store owns one validated directory and
best effort drop cleanup never targets its parent. The integration must preserve
source on every error. Operating system temporary cleanup handles crash remnants.

Test coverage: capacity rejection proves no partial file is created and drop tests
prove only the owned directory is removed. Phase 4 fault injection will cover actual
write, flush, and removal failures because portable unit tests cannot force them
reliably without filesystem shims.

