# ADR 0006: Fail closed at the SVG boundary

- Status: Accepted
- Date: 2026-08-27

## Context

MathJax is a separate process and its SVG is untrusted at the Rust process boundary.
SVG can contain scripts, external references, embedded images, style URLs, XML
entities, and expensive geometry. Passing worker output directly to a terminal or
general purpose parser would make a compromised worker a local resource and data
access risk.

The rendering path must also be deterministic on the Rust 1.85 minimum version and
must not discover system fonts or load resources at runtime.

## Decision

Place `latex-render-svg` between every worker success response and every cache or
raster operation. Parse and canonically rewrite XML with `quick-xml` 0.41 through an
explicit allowlist of the MathJax elements, attributes, and values that the corpus
requires. Strip MathJax source and accessibility metadata before the result enters
the cache.

Represent accepted bytes with a private-field `SanitizedSvg` type. Only that type can
enter the `resvg` 0.48 rasterizer. Build `resvg` without text, system font, compressed
SVG, or raster image features, provide no resource directory, and bound SVG bytes,
structure, dimensions, raw allocation, and encoded PNG size. Include sanitizer and
rasterizer policy versions in cache identity.

The rasterizer remains synchronous because the underlying native path is
synchronous. Asynchronous callers must execute it in a bounded blocking task.

## Consequences

- Unknown SVG syntax fails closed instead of silently expanding the attack surface.
- Worker output cannot enter a cache until it has passed the current policy.
- Cached SVG contains no original TeX source or MathJax accessibility metadata.
- Policy expansion requires a corpus example and a focused security regression test.
- Rendering the same accepted SVG at the same dimensions produces identical PNG
  bytes with the pinned dependency set.

## Rejected alternatives

- A regular expression or denylist was rejected because XML namespaces, entities,
  CSS, and URL encodings create too many bypass forms.
- Trusting MathJax output was rejected because process isolation is not a validation
  boundary.
- Browser, WebView, and Quick Look rendering were rejected because they enlarge the
  runtime and resource loading surface.
- `quick-xml` 0.42 was rejected because it requires Rust 1.86 and would violate the
  project minimum version.
