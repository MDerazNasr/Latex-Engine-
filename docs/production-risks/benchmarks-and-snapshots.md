# Benchmark and snapshot production risks

This note records the required failure prediction for the Phase 1 rendering evidence.

## 1. Missing, malformed, or unsafe corpus metadata changes fixture scope

Trigger: the corpus is empty, contains a malformed field or unsafe path name, loses an
entry, or snapshot update mode is enabled unintentionally.

Impact: coverage could silently shrink, a generated path could escape the fixture
directory, or reviewed files could be replaced without a deliberate upgrade.

Mitigation: the suite requires exactly 25 typed entries, permits only lowercase
letters, digits, and hyphens in names, expects exactly two fixed themes, and enables
writes only when `UPDATE_LATEX_SNAPSHOTS` equals `1`. The complete directory file set
and manifest must match the generated plan.

Test coverage: update and read-only verification modes both passed for 50 render
variants and 101 files. The suite also checks that source metadata is absent and
invalid TeX remains a structured nonfatal error.

## 2. Host load or cache races make benchmark numbers misleading

Trigger: background load, thermal throttling, accidental cache hits, worker restart,
or debug compilation changes a latency distribution.

Impact: a regression could pass or a healthy build could fail without representing
normal supported use.

Mitigation: the harness refuses debug builds, records sample counts and renderer
version, uses unique source strings for uncached samples, runs one supervised worker,
checks that it remains ready, and applies nearest-rank p95 over fixed minimum sample
counts. Published results include machine and toolchain context.

Test coverage: percentile unit tests cover empty and reversed samples. The release
harness passed all five targets on the recorded Phase 1 machine; CI and additional
hardware runs remain required before a broad performance claim.

## 3. Raster tasks or snapshot writes fail after partial asynchronous work

Trigger: a worker render fails late in the corpus, a blocking raster task panics, or
the filesystem fails during an explicitly authorized fixture update.

Impact: snapshot output could be incomplete, a test could hang, or a partial update
could be mistaken for a reviewed corpus.

Mitigation: all render and raster results are collected before update writes begin,
every blocking task is joined, worker shutdown is awaited, and any write or file-set
failure aborts the test. Git remains the recovery boundary for an explicitly enabled
partial fixture update.

Test coverage: the full corpus completed through one supervised worker, both snapshot
modes passed, and ordinary workspace tests keep the external worker suite ignored.
Phase 4 fault injection will add write-failure and blocking-pool saturation cases.
