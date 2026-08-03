# Engineering Rules

These rules apply to every change in this repository.

## Architecture authority

Read [`PROJECT_SPEC.md`](PROJECT_SPEC.md) before implementation. Follow its
required-reading map for the area being changed. If implementation and the
specification disagree, stop and resolve the specification first.

There is currently no architecture PDF in this repository. The Markdown
specification index and its linked documents are authoritative until an approved
architecture decision changes that.

## Branch discipline

- Never implement a feature or fix directly on `main`.
- Use `feature/<phase-or-capability>` for features.
- Use `fix/<issue-or-capability>` for fixes.
- Use `docs/<topic>` for documentation-only work.
- Confirm the active branch before editing.
- Keep unrelated work out of the active branch.
- Preserve user changes and never reset or discard them to make a branch clean.

## Commit discipline

- Make multiple focused commits within each project phase.
- Keep each commit independently understandable and as testable as practical.
- Commit tests with the behavior they verify.
- Separate architecture, scaffolding, feature behavior, integration, and release
  documentation when they are independently reviewable.
- Do not combine unrelated cleanup with a feature commit.
- Use imperative conventional commit subjects such as `feat:`, `fix:`, `test:`,
  `docs:`, `build:`, and `refactor:`.

## File and module boundaries

- No repository file may exceed 500 lines.
- Split a file before adding code that would cross the limit.
- Each file owns one cohesive responsibility.
- Each function performs one job and has one reason to change.
- Avoid circular dependencies and modules that merely forward calls without
  adding a boundary.

## Safe replacement

Never overwrite a working implementation with an unverified replacement. Add a
versioned candidate such as `renderer_v2.rs`, test it in isolation and in the
integration path, then replace the old implementation in a later focused change.
Remove the old implementation only after the replacement is verified.

## Comments and decisions

- Code comments explain why a choice exists, not what the following syntax does.
- Write comments as complete sentences.
- Do not use dash punctuation in code comments.
- Record non-obvious cross-module decisions in `docs/decisions/`.
- Keep decision records short: context, decision, consequences, and rejected
  alternatives.

## Test workflow

For every feature or fix:

1. Add or update focused unit tests.
2. Add or update integration tests for changed boundaries.
3. Run the narrowest relevant test command.
4. Run the broader workspace validation before reporting completion.
5. Run formatters and linters.
6. Remove dead, duplicated, temporary, and debugging code.
7. Confirm every file remains below 500 lines.

Do not defer tests to the end of a phase.

## Production failure review

After each feature, record the three most likely production failures under
`docs/production-risks/`. The review must explicitly consider:

- null, empty, missing, or malformed inputs;
- races, cancellation, resource contention, and state transitions; and
- unhandled asynchronous errors, timeouts, worker exits, and cleanup.

Each risk entry states its trigger, impact, mitigation, and test coverage.

## Self-review

Before a feature is reported complete, inspect the diff for:

- circular dependencies;
- redundant or unreachable logic;
- missing error handling;
- integration gaps;
- unbounded input, memory, concurrency, or output;
- accidental protocol or public API changes;
- undocumented architectural decisions; and
- files over 500 lines.

Report any unresolved finding instead of hiding it.

## Reporting

- Report completed changes only after their tests, formatting, cleanup, and
  self-review have run.
- State what changed, what is newly possible, current plan position, validation,
  the three leading production risks, and the next action.
- At the end of every session, write one short status paragraph describing the
  current codebase, the work completed, and the next step.
- When a durable task note exists, update it before ending the session so the
  status paragraph becomes the next session's context header.

## Data-flow review

When asked to explain data flow, trace the complete path from assistant message
delta through segmentation, rendering, caching, terminal layout, and display.
Verify names and dependencies at every boundary rather than describing an
intended flow that the code does not implement.
