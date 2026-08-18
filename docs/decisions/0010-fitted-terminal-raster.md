# ADR 0010: Fit equation content inside a transparent terminal canvas

Status: Accepted

Date: 2026-08-27

## Context

Terminal image protocols place an image into a rectangle of character cells. The
rounded cell rectangle does not necessarily share the equation SVG aspect ratio, so
rasterizing directly to the rectangle width and height can stretch glyphs. The
terminal layout contract already separates its exact canvas from the smaller content
rectangle, but the Phase 1 raster API intentionally fills both requested dimensions.

## Decision

Add a second raster API instead of changing the verified Phase 1 behavior. The fitted
request contains a bounded canvas and a nonempty content rectangle that must lie
fully inside it. Parse the sanitized SVG with the same resource disabled `resvg`
configuration, choose one scale for both axes, cap that scale at one, center the
result inside the content rectangle, and leave every other canvas pixel transparent.

Give this presentation behavior its own cache version label. Keep common parsing,
allocation, encoding, and byte limit enforcement behind private helpers so both APIs
share one security boundary.

## Consequences

- Existing CLI and snapshot output remains byte compatible.
- Terminal backends receive a PNG whose dimensions exactly match reserved cells.
- Content aspect ratio remains stable even when rounded cell dimensions differ.
- Small source images are not magnified and therefore avoid avoidable blur.
- A generation check is still required before an asynchronous result is displayed.

## Rejected alternatives

- Changing the Phase 1 raster API was rejected because it would silently alter
  existing CLI and snapshot behavior.
- Nonuniform scaling was rejected because it visibly distorts mathematical glyphs.
- Cropping to the content rectangle was rejected because the backend still needs an
  exact cell aligned canvas.

