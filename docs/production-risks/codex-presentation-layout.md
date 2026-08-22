# Codex presentation layout production risks

## 1. Null or adversarial geometry reserves invalid terminal cells

- Trigger: Terminal columns or cell pixels are zero, cell measurements are
  implausibly large, equation dimensions are empty, a baseline is nonfinite or
  outside the image, or policy values exceed their supported bounds.
- Impact: The transcript could reserve no space, overflow arithmetic, distort an
  image, or place terminal graphics outside its owned region.
- Mitigation: Constructors reject null and bounded geometry before layout. Policy
  limits width percentages, baseline targets, and reserved rows. Every pixel to cell
  conversion uses positive checked values.
- Test coverage: Focused tests cover zero and oversized measurements, invalid
  equation geometry, invalid policy, and exact bounded block dimensions.

## 2. Width races make an inline equation overlap trailing text

- Trigger: A completion is laid out with stale columns, or inline width is checked
  without accounting for text already rendered after its marker.
- Impact: Text and image cells could overlap, or resize could leave a placement at a
  column that no longer belongs to the transcript generation.
- Mitigation: Pure layout receives immutable measured geometry and includes both the
  current column and trailing rendered columns in its fit decision. The controller
  must generation-check terminal measurements again before publication.
- Test coverage: A focused test proves trailing text promotes an otherwise inline
  equation to a centered block. Controller race tests remain required before enablement.

## 3. Async completion uses a layout that no longer matches its source

- Trigger: Rendering finishes after message replacement, resize, theme change,
  backend change, source reveal, disable, or shutdown.
- Impact: A mathematically valid image could be published over unrelated or newly
  reflowed text.
- Mitigation: Layout is side effect free and carries no terminal writer. The future
  controller owns immutable source and terminal generations and must discard stale
  layouts before synchronized insertion.
- Test coverage: Pure tests prove deterministic results for fixed inputs. Async stale
  completion and publication tests belong to the controller stage.
