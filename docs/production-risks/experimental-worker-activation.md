# Experimental worker activation production risks

Feature commit: `bc88a28`

1. A missing, empty, malformed, or preexisting worker activation path could hide unrelated prefix content or leave the renderer without MathJax. Installation requires the exact activation path to be absent, creates its parent explicitly, and links only to the verified worker directory recorded inside the same bundle.
2. Concurrent installation, link replacement, or an activation link redirected during rollback could remove another process's path or separate the renderer from its installed worker. Create new link semantics reject collisions, installation cleanup removes only its exact relative target, and uninstallation resolves all three links to one version root before verifying each link again immediately before removal.
3. Directory creation, symbolic link creation, verification, or cleanup could fail after the version root has been copied. Every filesystem error is propagated, guards remove the exact links and incomplete version root they own, and rollback refuses to proceed when the worker link is missing, changed, or targets a different bundle.

Focused verification: eight package unit tests, four staging integration tests, and six installation integration tests passed. Coverage verifies worker target resolution, activation collision refusal, changed link rollback refusal, normal Codex preservation, and command process install and uninstall. Strict Clippy, Rust formatting, diff hygiene, ownership review, and the 500 line file limit passed.
