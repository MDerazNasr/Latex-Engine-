# ADR 0019: Install an explicit experimental Codex bundle

## Status

Accepted for MVP 0.1.

## Context

The Phase 3 Codex adapter discovers a sibling `latex-render` daemon before consulting
`PATH`, while the daemon discovers its worker under
`share/latex-render/mathjax-worker`. The project specification permits a developer
installation that requires Rust and Node.js, but forbids silently replacing the
user's normal `codex` executable. The experimental Codex checkout and renderer must
also remain independently buildable without sibling Cargo dependencies.

## Decision

Create a versioned `codex-latex` developer bundle with this layout:

```text
codex-latex-0.1.0-<target>/
  bin/codex-latex
  bin/latex-render
  share/latex-render/mathjax-worker/server.js
  share/latex-render/mathjax-worker/node_modules/mathjax/
  manifest-v1.json
```

The package copies the experimental Codex binary under the distinct name
`codex-latex`. It keeps `latex-render` beside that binary so Codex discovery is
independent from the caller's `PATH`. The MathJax worker remains a separate locked
Node.js runtime asset under the daemon's existing packaged lookup path.

A source build script creates the artifacts and delegates staging to a small Rust
packaging tool. The tool validates regular files, rejects escaping or cyclic module
links, publishes a sorted SHA-256 manifest, and replaces no existing bundle unless
the caller explicitly opts in.

Installation copies the complete bundle into an isolated version directory under an
explicit prefix and creates only `codex-latex` and `latex-render` entry points. It
never creates or replaces an entry point named `codex`. Uninstallation removes only
paths owned by the installed manifest and exact entry points that still target that
bundle.

## Consequences

- The experimental build has a distinct invocation and an explicit rollback path.
- Codex and the renderer remain separate builds connected only by protocol v1.
- MVP users must provide Node.js 22 or newer at runtime.
- The bundle can later be archived per target without changing its internal
  discovery contract.
- Windows packaging remains deferred as allowed by the MVP specification.

## Alternatives rejected

- Replacing the normal `codex` binary was rejected because rollback and user intent
  would be ambiguous.
- Adding a sibling Cargo dependency was rejected because the Codex checkout would no
  longer be independently reproducible or Bazel compatible.
- Embedding Node.js was rejected for MVP because it would expand the security,
  licensing, and cross-platform release surface before the developer workflow is
  accepted.
