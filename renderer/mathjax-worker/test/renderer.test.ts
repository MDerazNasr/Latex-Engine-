import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import { MAX_HEIGHT_PX } from "../src/limits.js";
import type { RenderParams } from "../src/protocol.js";
import { MathJaxRenderer, RenderFailure } from "../src/renderer.js";

interface CorpusEntry {
  name: string;
  source: string;
  displayMode: boolean;
}

const CORPUS_URL = new URL(
  "../../../../fixtures/rendering/math-corpus.json",
  import.meta.url,
);

test("the 25-expression feasibility corpus renders to bounded SVG", async () => {
  const corpus = JSON.parse(
    await readFile(CORPUS_URL, "utf8"),
  ) as CorpusEntry[];
  assert.equal(corpus.length, 25);
  const renderer = await MathJaxRenderer.create();

  for (const entry of corpus) {
    const result = await renderer.render(params(entry));
    assert.match(result.svgUtf8, /^<svg /, entry.name);
    assert.match(result.svgUtf8, /<\/svg>$/, entry.name);
    assert.ok(result.widthPx > 0 && result.widthPx <= 1200, entry.name);
    assert.ok(
      result.heightPx > 0 && result.heightPx <= MAX_HEIGHT_PX,
      entry.name,
    );
    assert.ok(
      result.baselinePx >= 0 && result.baselinePx <= result.heightPx,
      entry.name,
    );
    assert.equal(result.accessibilityText, entry.source);
  }
});

test("very tall output is scaled within the raster height limit", async () => {
  const renderer = await MathJaxRenderer.create();
  let source = "1";
  for (let index = 0; index < 48; index += 1) {
    source = `\\frac{1}{${source}}`;
  }

  const result = await renderer.render(
    params({ name: "tall", source, displayMode: true }),
  );

  assert.ok(result.heightPx > 0 && result.heightPx <= MAX_HEIGHT_PX);
});

test("invalid TeX returns a source-free public error", async () => {
  const renderer = await MathJaxRenderer.create();

  await assert.rejects(
    renderer.render(
      params({ name: "invalid", source: "\\frac{1}{", displayMode: true }),
    ),
    (error: unknown) =>
      error instanceof RenderFailure &&
      error.publicError.code === "INVALID_TEX" &&
      !error.publicError.message.includes("frac"),
  );
});

function params(entry: CorpusEntry): RenderParams {
  return {
    source: entry.source,
    displayMode: entry.displayMode,
    foreground: "#e6edf3",
    background: "transparent",
    scale: 2,
    maxWidthPx: 1200,
  };
}
