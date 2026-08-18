# Terminal capability detection production risks

This note records the required failure prediction for passive terminal backend
selection.

## 1. Missing, empty, or malformed environment facts select the wrong backend

Trigger: terminal variables are absent, empty, spoofed, contain malformed iTerm2
versions, or describe an outer terminal instead of the current stream.

Impact: a capable terminal may show source text, or a false positive could receive
unsupported control sequences.

Mitigation: empty values are absent, versions require a complete numeric form, and
all unknown or inconsistent combinations fail to text. Only narrow known markers
select an image backend.

Test coverage: tests cover missing facts, unknown terminals, old, absent, and
malformed versions, redirected output, and stable diagnostic names.

## 2. SSH or multiplexer state races detection and later publication

Trigger: the process is reattached, enters a multiplexer, changes output destination,
or reconnects while rendering is active.

Impact: a job could carry local file transport into a remote session or publish an
image after the environment no longer supports it.

Mitigation: detection records explicit SSH, tmux, Zellij, and Screen facts. The TUI
integration must refresh support on terminal lifecycle changes and pass backend
changes through the presenter, which advances generation and invalidates older work.

Test coverage: tests cover every supported multiplexer marker, remote iTerm2 fallback,
and direct Kitty behavior over SSH. Presenter tests cover backend change races.

## 3. A future active probe hangs or its reply is left unhandled

Trigger: a terminal ignores a query, a multiplexer consumes it, cancellation drops
the awaiting task, or a delayed reply enters the normal keyboard stream.

Impact: the UI could block, render reply bytes as input, or enable a backend after
the relevant terminal generation ended.

Mitigation: Phase 2 performs no asynchronous probe and has no unhandled async path.
Any later probe must have a strict deadline, correlation identifier, cancellation
handling, and generation check before it can override this passive result.

Test coverage: current tests prove passive detection has no terminal side effects.
Phase 3 pseudo terminal tests must inject timeout, cancellation, and late replies
before active probing can be enabled.

