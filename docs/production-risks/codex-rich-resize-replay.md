# Codex rich resize replay production risks

Feature commit: `4c96b23b9b`

1. Null rows, malformed placements, duplicate image identifiers, missing fallback lines, or a row cap that cuts through an image could create invalid terminal geometry. Layout composition shifts every placement with checked bounds, and clipping through an image converts the retained tail to readable source.
2. Resize, rollback, pagination, stream consolidation, and image publication can race and leave an old image over newly wrapped text. Replay deletes every published history image before clearing the screen, rebuilds one immutable layout from source cells, and publishes its text and placements as one batch.
3. Missing raster files, terminal write failures, closed pagination receivers, or an unhandled replay error could produce partial output. Asset preflight falls back atomically to source, image deletion and screen clearing propagate terminal errors, event sends remain nonblocking, and failed replay leaves the source transcript unchanged for retry.

Focused verification: four layout tests, six image batch tests, ten resize replay tests, two stream finalization tests, one status rebase test, and one unrestricted pagination integration test passed. The required Codex fixer and scoped formatter completed without code warnings.
