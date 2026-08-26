# Experimental bundle staging production risks

Feature commit: `ad0f999`

1. Missing or empty binaries, malformed version labels, absent worker output, null paths, or pnpm links that escape the owned module root could publish an invalid or attacker-selected runtime. Typed stage inputs validate every required regular file, labels are bounded to portable characters, required runtime paths are checked, and symbolic links are materialized only when their canonical targets stay within the selected `node_modules` root.
2. Two package processes or an external writer could race to claim the same versioned output, while a build artifact could change during copying. Output reservation uses create-new directory semantics, an existing path is never replaced, file hashes describe the bytes actually copied, and the bundle becomes valid only when its manifest is atomically published last.
3. File reads, hashing, recursive copies, link resolution, permissions, manifest encoding, or publication could fail after staging begins. Every error returns through one typed boundary, active directory cycles are rejected, an incomplete-bundle guard removes the exact newly reserved output, and no consumer accepts a directory without the final manifest.

Focused verification: three parser and validation unit tests plus four bundle staging integration tests passed for complete layout, sorted SHA256 ownership, existing output preservation, internal pnpm link materialization, and escaping link rejection. Strict Clippy, Rust formatting, diff hygiene, dependency review, and the 500-line file limit passed.
