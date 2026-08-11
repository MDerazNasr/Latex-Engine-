# ADR 0002: Use the supported MathJax component package

- Status: Accepted
- Date: 2026-08-26

## Context

The feasibility worker needs a locally installed, version-locked MathJax backend
that can initialize only the TeX and SVG components required by the protocol.
`mathjax-full` 3.2.2 is deprecated and its audited dependency graph contains a
critical advisory. The source-oriented `@mathjax/src` 4.1.3 package exposes
declaration files that do not type-check cleanly as an application dependency.

## Decision

Use the supported `mathjax` 4.1.3 Node component package with exact dependency
versions and a pnpm lockfile. Initialize only `tex-base`, the AMS TeX extension,
and SVG output. Keep a minimal local declaration for the small runtime API surface
the worker consumes because the component package does not publish matching TypeScript
declarations for its default loader entry point.

The worker will not dynamically load MathJax packages or external resources. The
local declaration is compile-time only and must be covered by a real-process
integration test so runtime export changes fail visibly.

## Consequences

- The initial renderer uses a maintained MathJax 4 distribution without the
  deprecated package's dependency exposure.
- The installed production dependency graph contains MathJax and its bundled font
  package only.
- Upgrading MathJax requires revalidating the local declaration, the 25-expression
  corpus, SVG safety checks, and the stdio handshake.
- The Rust side remains independent of Node and MathJax because it will consume
  only the versioned JSONL protocol.
