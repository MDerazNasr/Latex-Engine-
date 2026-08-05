import assert from "node:assert/strict";
import test from "node:test";

import { MAX_SOURCE_BYTES } from "../src/limits.js";
import { extractRequestId, validateRequest } from "../src/protocol.js";

test("valid requests receive deterministic defaults", () => {
  const result = validateRequest({
    protocol: 1,
    id: "eq-42",
    method: "render",
    params: { source: "x^2" },
  });

  assert.equal(result.ok, true);
  if (result.ok) {
    assert.deepEqual(result.request.params, {
      source: "x^2",
      displayMode: false,
      foreground: "#e6edf3",
      background: "transparent",
      scale: 2,
      maxWidthPx: 1200,
    });
  }
});

test("null and malformed fields return stable validation errors", () => {
  for (const value of [
    null,
    [],
    {},
    { protocol: 2, id: "eq", method: "render", params: { source: "x" } },
    { protocol: 1, id: "bad id", method: "render", params: { source: "x" } },
    { protocol: 1, id: "eq", method: "unknown", params: { source: "x" } },
    { protocol: 1, id: "eq", method: "render", params: null },
    { protocol: 1, id: "eq", method: "render", params: { source: "" } },
  ]) {
    const result = validateRequest(value);
    assert.equal(result.ok, false);
    if (!result.ok) {
      assert.equal(result.error.code, "INVALID_REQUEST");
      assert.equal(result.error.retryable, false);
    }
  }
});

test("source byte limits and control characters are enforced", () => {
  const oversized = validateRequest({
    protocol: 1,
    id: "large",
    method: "render",
    params: { source: "π".repeat(MAX_SOURCE_BYTES) },
  });
  assert.equal(oversized.ok, false);
  if (!oversized.ok) {
    assert.equal(oversized.error.code, "INPUT_LIMIT_EXCEEDED");
  }

  const controlled = validateRequest({
    protocol: 1,
    id: "control",
    method: "render",
    params: { source: "x\u0000y" },
  });
  assert.equal(controlled.ok, false);
});

test("unsafe identifiers are never reflected as correlation IDs", () => {
  assert.equal(extractRequestId({ id: "safe:id_1.2" }), "safe:id_1.2");
  assert.equal(extractRequestId({ id: "line\nbreak" }), null);
  assert.equal(extractRequestId({ id: null }), null);
});
