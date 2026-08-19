# Codex `b68acc4d` Phase 3 integration contract

## Revision and checkout

- Official repository: `https://github.com/openai/codex.git`
- Inspected commit: `b68acc4d4b56fdfa1d5b6a2c36102c66876e0c46`
- Commit date: 2026-08-27
- Experimental checkout: sibling repository `codex-latex-integration`
- Experimental branch: `feature/latex-rendering-integration`

The LaTeX engine remains the renderer source of truth. The Codex checkout contains
only the presentation adapter and behavior-preserving generalization of existing
terminal image machinery.

## Verified current flow

The current assistant transcript path remains compatible with the Phase 0 seam:

1. `chatwidget/protocol.rs` receives `AgentMessageDelta`.
2. `chatwidget/streaming.rs` forwards the unchanged delta to the existing stream
   controller.
3. Finalization sends `AppEvent::ConsolidateAgentMessage` with authoritative raw
   Markdown source.
4. `app/agent_message_consolidation.rs` replaces provisional cells with one
   source-backed `AgentMarkdownCell`.
5. `AgentMarkdownCell` re-renders from source at each width and preserves separate
   raw lines.
6. `app/resize_reflow.rs` rebuilds terminal scrollback from stored history cells.
7. `tui.rs` flushes pending history inside synchronized terminal updates.

No renderer change may modify model context, app-server notifications, stored thread
items, raw transcript mode, export, JSON output, or redirected output.

## Verified terminal image seam

Codex currently keeps Kitty direct PNG, iTerm2 3.6 Kitty local-file transfer, Sixel,
terminal detection, targeted deletion, cursor preservation, and synchronized writes
under `tui/src/pets/`. This logic is correct but pet-specific.

Phase 3 first extracts the behavior into a private `terminal_image_v2` module and
leaves pet wrappers and byte-for-byte tests intact. The new module owns transport,
image identifiers, placement, replacement, deletion, Sixel clearing, cursor safety,
and writer flushing. Pet and math callers own only their asset and layout state.

The versioned module name preserves the working pet implementation until the
extraction passes its focused test gate.

## Renderer process boundary

The experimental Codex build must not use sibling Cargo path dependencies because
they make the checkout non-reproducible and cannot pass Codex Bazel builds. It uses a
versioned local daemon protocol exposed by the installed `latex-render` executable.

The daemon owns:

- the persistent supervised MathJax worker;
- lossless streaming segmentation;
- request validation and resource limits;
- SVG sanitization and native PNG rasterization;
- canonical source-preserving public errors; and
- clean child shutdown.

Codex owns:

- renderer discovery and the complete feature disable switch;
- terminal measurement and capability selection;
- one bounded asynchronous request queue;
- message, equation, width, theme, and backend generations;
- source-backed transcript layout and source reveal;
- synchronized terminal publication and deletion; and
- failure UI that never interrupts an agent turn.

The first protocol version accepts newline-delimited JSON and returns correlated
newline-delimited JSON. Each response carries equation byte spans, display mode,
PNG dimensions, baseline, and a bounded base64 PNG. Neither process may evaluate a
shell command or accept arbitrary output paths from the other.

## Transcript presentation contract

`HistoryCell` gains a default-empty rich layout method that returns logical lines and
image placements as one immutable value for a single width generation. Existing
cells retain their behavior through the default implementation.

`AgentMarkdownCell` remains the owner of raw Markdown. When an equation is ready, a
private presentation rewrite replaces only that source span with a collision-safe
marker before normal Markdown rendering. Marker resolution then:

- reserves measured blank cells for block math;
- reserves an inline rectangle when it fits the current row;
- promotes oversized inline math to a centered block;
- attaches an image placement to the exact reserved row and column; and
- leaves pending, failed, disabled, and unsupported equations as original source.

The rewrite never changes `raw_lines`. Code fences, inline code, escaped delimiters,
prices, malformed delimiters, and unsupported environments remain normal Markdown.

History insertion queues lines and placements together. The synchronized flush
writes text first, publishes only placements still visible in the owned history
region, and deletes replaced image identifiers before publishing new generations.
Resize reflow deletes the previous generation, re-renders layout from source using
fresh terminal cell and pixel measurements, and ignores late completions.

## Async lifecycle

Renderer I/O never runs in `ChatWidget` or the draw closure. A private controller
owns the daemon and sends completion events through `AppEvent`. Each request carries:

- message identity;
- equation identity;
- render generation;
- terminal width and pixel generation;
- theme generation; and
- terminal backend identity.

The app publishes a completion only when all values still match. Thread switch,
message replacement, raw-source toggle, feature disable, resize, theme change,
renderer restart, and shutdown invalidate older generations before any terminal
bytes are produced.

## Configuration and source reveal

The experimental build starts with a TUI-private configuration adapter so the first
patch does not expand `codex-core` or app-server APIs. Packaging supplies the daemon
path explicitly. The adapter exposes enabled state, backend selection, single-dollar
policy, theme, timeouts, and resource limits.

The rich transcript has a source toggle that invalidates active placements and
immediately reflows the same stored Markdown. Raw transcript mode always shows source
and never publishes equation images.

## Reviewable implementation stages

Each stage stays under the upstream 800-line review limit and receives focused tests
before the next stage begins:

1. Extract and verify `terminal_image_v2` without changing pet behavior.
2. Add the renderer daemon contract, implementation, and process tests in this
   repository.
3. Add the private Codex renderer supervisor and protocol decoder.
4. Add source-preserving presentation rewriting with snapshot tests.
5. Add history layout metadata and synchronized image insertion tests.
6. Connect consolidation, resize, cancellation, theme, and failure events.
7. Add configuration, source toggle, diagnostic UI, and packaging.
8. Run replayed and live Codex acceptance in Kitty and iTerm2.

## Mandatory Phase 3 tests

- Fragmented deltas and completed source produce identical raw Markdown.
- Inline, display, code, escaped delimiter, price, malformed, and UTF-8 cases match
  the independent segmenter corpus.
- Pending, rendered, failed, disabled, unsupported, and raw-source cells have reviewed
  snapshots.
- A delayed render cannot block input, commit ticks, or app-server events.
- Late results after resize, theme change, thread switch, replacement, toggle, and
  shutdown emit no terminal bytes.
- Two generations delete the old Kitty placement before publishing the new one.
- Sixel clears its prior reserved area before repaint.
- Partial writer, daemon crash, timeout, malformed JSON, oversized PNG, and queue
  saturation preserve readable source and terminate or restart owned children.
- Redirected output, raw transcript, transcript export, app-server data, and model
  context remain byte compatible.

## Phase 3 failure prediction

1. Null, malformed, or adversarial source and geometry could create invalid markers,
   excessive allocations, or placement outside the transcript. Strict protocol,
   segment, geometry, PNG, and marker validation must fail to source.
2. Streaming, resize, theme, backend, thread, and replacement races could publish a
   stale image. Immutable generations and weak ownership must reject every late
   completion before terminal output.
3. Daemon startup, read, write, timeout, task join, terminal partial write, or shutdown
   could remain unhandled. Bounded queues, supervised restart, source-first state,
   synchronized terminal guards, and child reaping are required at every async edge.
