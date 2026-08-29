# Experimental bundle build production risks

Feature commit: `41beaec`

1. Missing roots, malformed versions, dirty repositories, absent pinned ancestry, null tool output, or an existing destination could produce an unreproducible bundle. Preflight validation rejects every condition before compilation and requires the renderer and Codex checkouts to be clean.
2. Concurrent builds or a race that creates the destination during compilation could publish mixed binaries, worker files, or manifest data. Each build uses one nonexisting output path, stages into a private temporary directory, hashes the complete closure, and publishes the manifest only after all payload files are final.
3. Frozen dependency installation, TypeScript compilation, locked Cargo release builds, file copying, hashing, or child process termination could fail asynchronously and leave an artifact that looks installable. Every subprocess exit and filesystem operation is checked, incomplete staging lacks a valid manifest, and the installer independently verifies every byte before activation.

Focused verification: the corrected 0.1.0 bundle was rebuilt from both clean feature branches with frozen worker dependencies and locked Cargo releases. The release link completed in 23 minutes 20 seconds, the published bundle contained 1,254 hashed files, and subsequent health, render, install, TUI discovery, and rollback checks all passed.
