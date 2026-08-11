# ADR 0008: Review renderer changes through canonical and perceptual evidence

- Status: Accepted
- Date: 2026-08-27

## Context

Pinned renderer dependencies make output reproducible, but dependency, sanitizer,
geometry, and color changes can still alter terminal presentation. Exact PNG byte
comparison is sensitive to encoding details that do not affect pixels, while a hash
alone does not make an upgrade visually reviewable.

Performance claims also need a fixed sampling method that distinguishes process
startup, uncached worker work, cache hits, and segmenter cost.

## Decision

Store exact canonical SVG and decoded PNG reference images for all 25 corpus entries
in both light and dark foreground themes. Store dimensions, display mode, and baseline
values in a readable manifest. Compare SVG as bytes and PNG as RGBA pixels under the
limits defined by the quality specification.

Require `UPDATE_LATEX_SNAPSHOTS=1` before the test may write fixtures. Validate every
fixture stem before path construction and require the directory file set to match the
corpus exactly so stale artifacts cannot hide.

Use a framework-free release benchmark binary with nearest-rank p95 calculation and
fixed sample floors. Measure one persistent worker session, use distinct sources for
uncached samples, and make missed specification targets a failing exit status.

## Consequences

- Renderer upgrades produce reviewable vector, pixel, and baseline diffs.
- PNG encoder changes that preserve pixels can remain within the declared tolerance.
- Snapshot updates are deliberate but still require Git diff review before commit.
- Benchmark numbers remain machine-dependent and must be published with their
  hardware, operating system, toolchain, sample count, and revision.

## Rejected alternatives

- PNG hashes were rejected because harmless encoder changes would provide no visual
  comparison path.
- SVG hashes were rejected because sanitizer or geometry changes need readable diffs.
- Debug-profile timing was rejected because it does not represent the shipped Rust
  path.
- A benchmark framework dependency was rejected because the required fixed p95 gate
  is small and auditable with the standard library.
