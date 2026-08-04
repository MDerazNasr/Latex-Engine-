import assert from "node:assert/strict";
import test from "node:test";

import { PROTOCOL_VERSION, RENDERER_VERSION } from "../src/version.js";

test("protocol and renderer versions are explicit", () => {
  assert.equal(PROTOCOL_VERSION, 1);
  assert.match(RENDERER_VERSION, /^\d+\.\d+\.\d+$/);
});
