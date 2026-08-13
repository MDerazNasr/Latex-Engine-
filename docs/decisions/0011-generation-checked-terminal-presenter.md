# ADR 0011: Correlate terminal frames with immutable generations

Status: Accepted

Date: 2026-08-27

## Context

Layout, SVG rasterization, and terminal writes run at different speeds. A terminal
resize, theme change, fallback, or backend change can occur while native
rasterization is still running. Correct PNG output is unsafe to display when it was
created for an older cell canvas. The presentation seam must also preserve the last
valid image until a complete replacement is ready and must never emit image control
sequences in text mode.

## Decision

Let one presenter own the selected backend, a checked monotonic generation, and the
existing deterministic image placement state. Beginning work creates an immutable
job containing generation, backend, image identity, row, and complete layout. The job
derives its fitted raster request without terminal side effects and can therefore run
in a blocking pool.

Correlate PNG output back to the same job. Reject malformed or globally oversized PNG
bytes and dimensions that differ from the reserved canvas. Publication first checks
generation and backend, then validates that its backend source contains the same
bounded raster, and only then asks placement state to encode replacement bytes.
Stale work returns a distinct outcome with no terminal output. Fallback and backend
changes invalidate pending jobs and return targeted cleanup bytes.

## Consequences

- Older raster tasks cannot overwrite a newer transcript layout.
- Raster failure leaves placement state unchanged until the caller chooses fallback.
- Text mode issues no jobs and therefore cannot leak image control sequences.
- Direct PNG and local file sources pass through one correlation boundary.
- The Codex adapter remains responsible for awaiting blocking tasks and writing
  returned command bytes atomically with transcript updates.

## Rejected alternatives

- Comparing only image identifiers was rejected because a resize can reuse the same
  equation and identifier with different geometry.
- Clearing the active image when work begins was rejected because a failed render
  would create avoidable flicker and a blank reservation.
- Allowing publication to mutate generation was rejected because completion order
  would then determine correctness.

