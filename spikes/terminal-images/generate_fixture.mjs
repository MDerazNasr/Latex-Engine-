import { mkdir, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

import { MathJaxRenderer } from "../../renderer/mathjax-worker/dist/src/renderer.js";

const fixtureDirectory = fileURLToPath(
  new URL("../../fixtures/terminal/", import.meta.url),
);
const svgPath = fileURLToPath(
  new URL("../../fixtures/terminal/quadratic-formula.svg", import.meta.url),
);

await mkdir(fixtureDirectory, { recursive: true });
const renderer = await MathJaxRenderer.create();
const result = await renderer.render({
  source: String.raw`x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}`,
  displayMode: true,
  foreground: "#111827",
  background: "#ffffff",
  scale: 4,
  maxWidthPx: 1024,
});
await writeFile(svgPath, `${result.svgUtf8}\n`, "utf8");
