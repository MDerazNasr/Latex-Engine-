import assert from "node:assert/strict";
import test from "node:test";

import { MAX_SVG_BYTES } from "../src/limits.js";
import { SvgValidationError, validateGeneratedSvg } from "../src/sanitize.js";

test("passive standalone SVG is accepted", () => {
  const svg = '<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0"/></svg>';
  assert.equal(validateGeneratedSvg(svg), svg);
});

test("active or externally referenced SVG is rejected", () => {
  const unsafe = [
    '<svg xmlns="http://www.w3.org/2000/svg"><script/></svg>',
    '<svg xmlns="http://www.w3.org/2000/svg" onload="run()"></svg>',
    '<svg xmlns="http://www.w3.org/2000/svg"><use href="https://example.test/x"/></svg>',
    '<svg xmlns="http://www.w3.org/2000/svg"><foreignObject/></svg>',
    '<svg xmlns="http://www.w3.org/2000/svg"><style>path{fill:url(x)}</style></svg>',
  ];

  for (const svg of unsafe) {
    assert.throws(() => validateGeneratedSvg(svg), SvgValidationError);
  }
});

test("oversized SVG is rejected before transport", () => {
  const svg = `<svg ${"x".repeat(MAX_SVG_BYTES)}></svg>`;
  assert.throws(
    () => validateGeneratedSvg(svg),
    (error: unknown) =>
      error instanceof SvgValidationError &&
      error.publicError.code === "OUTPUT_LIMIT_EXCEEDED",
  );
});
