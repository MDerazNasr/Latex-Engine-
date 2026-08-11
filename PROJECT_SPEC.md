# Codex LaTeX Renderer: Project Specification

Status: Proposed  
Target release: MVP 0.1  
Last updated: 2026-08-26

This file is the authoritative index for the project specification. All linked
specification documents are normative and must be read before changing the
areas they govern.

## Specification map

1. [Product and architecture](docs/spec/product-and-architecture.md)
   defines the vision, feasibility boundary, goals, user experience, supported
   syntax, system architecture, and repository layout.
2. [Components and integration](docs/spec/components-and-integration.md)
   defines the Rust interfaces, MathJax worker, protocol, caching, terminal
   capabilities, Codex TUI integration, and optional CLI and MCP adapters.
3. [Quality and delivery](docs/spec/quality-and-delivery.md)
   defines security, privacy, reliability, performance, accessibility, testing,
   diagnostics, distribution, milestones, acceptance criteria, risks, and next
   actions.

## Governing decisions

- Transparent rendering is implemented in the Codex terminal presentation layer.
- MCP remains optional because an explicit tool call cannot transparently process
  every assistant response.
- Rust owns parsing, orchestration, cache, terminal integration, and Codex-facing
  APIs.
- A long-lived TypeScript and MathJax worker is the initial TeX-to-SVG backend.
- Original LaTeX remains canonical for copying, history, accessibility, errors,
  and unsupported terminals.
- Rendering is asynchronous, bounded, local by default, and never allowed to
  interrupt an agent turn.
- Inline rendering accepts math fragments only. Full-document compilation is a
  separate future capability with a separate sandbox and threat model.

## Required reading by change type

| Change | Required specification |
|---|---|
| Product behavior or supported syntax | Product and architecture |
| Parser or public API | Product and architecture; Components and integration |
| Renderer or worker protocol | Components and integration; Quality and delivery |
| Terminal backend or Codex TUI | All three documents |
| Security, packaging, CI, or release | Quality and delivery |

## Conflict resolution

If implementation and specification conflict, stop implementation and resolve the
specification first. If linked specification documents conflict with each other,
the more restrictive security or reliability requirement wins until an explicit
architecture decision records otherwise.

## Source provenance

The Codex extension decisions were verified against current official OpenAI
documentation for MCP, CLI customization, App Server, plugins, and the
open-source Codex components. The inference that automatic per-message rendering
requires presentation-layer integration must be revalidated against the exact
Codex source revision during Phase 0.
