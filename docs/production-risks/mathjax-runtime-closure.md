# MathJax runtime closure production risks

Feature commit: `e41b184`

1. A missing, empty, malformed, or relocated MathJax font dependency could publish a worker that emits startup errors before its ready frame. Staging resolves the locked font from the installed MathJax package's own dependency root, requires a nonempty package manifest, and records both modules in the bundle manifest.
2. A pnpm link changed concurrently, redirected outside the owned module root, or made cyclic could copy untrusted files or recurse indefinitely. Both MathJax trees use the same canonical owned root, reject escaping targets and special files, and retain active path cycle detection while materializing links.
3. Font copying, manifest hashing, or dependency verification could fail after bundle creation begins. Every failure propagates through the stager, the incomplete bundle guard removes the reserved output, and the manifest remains unpublished until the required font package is present and hashed.

Focused verification: eight package unit tests, four staging integration tests, and six installation integration tests passed. Coverage verifies the font file is materialized, linked packages remain inside their owned root, missing inputs publish nothing, and installed bundles retain their activation ownership. Strict Clippy, Rust formatting, diff hygiene, dependency direction review, and the 500 line file limit passed.
