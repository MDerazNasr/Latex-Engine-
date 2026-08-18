# Doctor command production risks

This note records the required failure prediction for combined renderer and terminal
diagnostics.

## 1. Missing, malformed, or duplicate command and environment values

Trigger: worker or Node paths are empty, an option is repeated or unknown, terminal
facts are absent, or an iTerm2 version is malformed.

Impact: doctor could execute an unintended program, misreport terminal support, or
produce output that scripts cannot interpret.

Mitigation: doctor reuses strict worker option parsing with direct process arguments
and no shell. Passive detection fails closed, and reports only normalized booleans
and stable enum names rather than raw values.

Test coverage: parser tests cover every accepted option, command help, duplicate and
positional rejection. Capability tests cover null and malformed terminal facts.

## 2. Terminal state changes while doctor or later rendering is active

Trigger: output is redirected, an SSH session reconnects, or a multiplexer starts or
stops after the environment snapshot is captured.

Impact: doctor can describe the prior terminal while a subsequent render sees a new
backend.

Mitigation: output explicitly records its TTY and connection snapshot. Doctor makes
no lasting backend mutation. The interactive integration must redetect lifecycle
changes and invalidate presenter generation before publication.

Test coverage: process tests prove captured redirected output selects text, while
capability and presenter tests cover SSH, multiplexer, and backend change behavior.

## 3. Worker, raster, shutdown, or reporting work fails asynchronously

Trigger: the worker fails startup or handshake, render or native raster fails, the
blocking task loses its result, shutdown fails, or report construction unexpectedly
fails.

Impact: a partial status report could claim the installation is healthy, or an
unhandled task failure could terminate without the stable exit category.

Mitigation: doctor awaits the existing supervised check including shutdown before it
constructs terminal output. Every error maps through existing CLI categories, and
the command output is returned only as one complete in memory buffer.

Test coverage: process tests run the healthy worker and native pipeline, then inject
a missing worker and assert the worker exit code with empty stdout and no terminal
fields. Existing CLI tests cover raster join and output error mappings.

