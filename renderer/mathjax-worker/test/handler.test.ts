import assert from "node:assert/strict";
import test from "node:test";

import { RequestHandler } from "../src/handler.js";
import { MAX_JSON_LINE_BYTES } from "../src/limits.js";

test("handler returns protocol errors without throwing", async () => {
  const handler = await RequestHandler.create();

  const invalidJson = await handler.handleLine("{broken");
  assert.equal(invalidJson.ok, false);
  if (!invalidJson.ok) {
    assert.equal(invalidJson.error.code, "INVALID_JSON");
  }

  const oversized = await handler.handleLine(
    "x".repeat(MAX_JSON_LINE_BYTES + 1),
  );
  assert.equal(oversized.ok, false);
  if (!oversized.ok) {
    assert.equal(oversized.error.code, "INPUT_LIMIT_EXCEEDED");
  }
});

test("handler correlates successful rendering", async () => {
  const handler = await RequestHandler.create();
  const response = await handler.handleLine(
    JSON.stringify({
      protocol: 1,
      id: "eq-42",
      method: "render",
      params: { source: "x^2", displayMode: false },
    }),
  );

  assert.equal(response.ok, true);
  assert.equal(response.id, "eq-42");
  if (response.ok) {
    assert.match(response.result.svgUtf8, /^<svg /);
  }
});
