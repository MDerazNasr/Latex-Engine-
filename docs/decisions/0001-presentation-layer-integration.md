# ADR 0001: Render Math in the Codex Presentation Layer

Status: Accepted  
Date: 2026-08-26

## Context

The required behavior is automatic: every supported LaTeX fragment in an
assistant response should render without a second user prompt or model-selected
tool call. Codex supports MCP tools and plugins, exposes an App Server for rich
clients, and publishes the CLI source.

## Decision

Implement detection and display in the Codex TUI presentation path. Keep the
parser, render protocol, and terminal backends in this independent repository so
the Codex-side adapter remains small. Preserve source LaTeX as the canonical
conversation representation.

## Consequences

- Automatic rendering does not depend on model behavior.
- The renderer can be tested independently from Codex.
- The integration requires an upstream Codex change or an experimental build
  until the capability is accepted upstream.
- Codex source changes can require adapter maintenance.

## Rejected alternatives

- MCP alone was rejected because tools require selection and do not provide a
  general transcript post-processing hook.
- A shell wrapper was rejected because an interactive alternate-screen TUI cannot
  be safely transformed as a plain stdout stream.
- A custom App Server client was deferred because recreating the complete Codex
  terminal experience is a larger and less focused first integration.

## Evidence

- <https://learn.chatgpt.com/docs/extend/mcp?surface=cli>
- <https://learn.chatgpt.com/docs/app-server>
- <https://learn.chatgpt.com/docs/open-source>
