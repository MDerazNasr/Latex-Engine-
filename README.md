# Codex LaTeX Renderer

Render LaTeX mathematics automatically inside the Codex CLI instead of showing
raw delimiters and commands.

The project is currently in the specification phase. See
[PROJECT_SPEC.md](PROJECT_SPEC.md) for the product requirements, architecture,
Codex integration strategy, security model, milestones, and acceptance criteria.

The key architectural decision is that transparent rendering belongs in the
Codex terminal presentation layer. An MCP tool can render an equation when the
model explicitly calls it, but cannot reliably post-process every assistant
message. This project therefore consists of:

1. An independent, reusable math parser and renderer.
2. A small integration with the open-source Codex CLI TUI.
3. Optional CLI/MCP adapters for testing and explicit artifact generation.
