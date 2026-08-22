# Codex presentation rewrite production risks

## 1. Null, malformed, or adversarial spans hide readable source

- Trigger: A response carries an empty, overlapping, unordered, non UTF 8, escaped,
  code-contained, syntactically inconsistent, missing, or duplicated equation span.
- Impact: Markdown text could disappear, a code sample could become an image, or an
  image could be attached to unrelated source.
- Mitigation: Rewriting revalidates every span and delimiter, excludes Markdown code
  ranges, allocates private-use markers absent from the source, and requires every
  marker to survive normal Markdown rendering exactly once. Any failure rerenders the
  complete untouched source with no placements.
- Test coverage: Focused tests cover invalid UTF 8 boundaries, ordering, escaped and
  code delimiters, lost markers, pending and failed outcomes, and exact source fallback.

## 2. Layout or hyperlink state drifts while markers are resolved

- Trigger: Unicode byte offsets are mistaken for columns, a marker expands beyond its
  line, or hyperlink annotations retain columns from before the reservation.
- Impact: Images could overlap prose, wide characters could shift placement, or a
  terminal hyperlink could target the wrong cells.
- Mitigation: Resolution uses display columns, includes trailing text in inline fit,
  promotes oversized inline math to a block, and remaps hyperlink intersections as
  styled fragments move around reservations.
- Test coverage: Reviewed snapshots cover inline, centered block, promotion, and UTF 8
  placement. A focused test compares all unrelated hyperlink metadata before and after.

## 3. Async source replacement publishes a valid rewrite for an old message

- Trigger: A renderer completion arrives after resize, message consolidation,
  replacement, thread switch, source reveal, disable, theme change, or shutdown.
- Impact: Correct lines and placements for an old source could overwrite the current
  transcript even though the pure rewrite itself passed validation.
- Mitigation: Presentation rewriting has no terminal or task side effects. The next
  controller stage must bind outcomes to immutable message and terminal generations
  and reject them again before synchronized publication.
- Test coverage: Pure tests prove fallback and deterministic layout for fixed inputs.
  Lifecycle race coverage remains mandatory before the production adapter is enabled.
