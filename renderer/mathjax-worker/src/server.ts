import { once } from "node:events";
import { createInterface } from "node:readline";

import { RequestHandler } from "./handler.js";
import {
  MAX_HEIGHT_PX,
  MAX_JSON_LINE_BYTES,
  MAX_SCALE,
  MAX_SOURCE_BYTES,
  MAX_SVG_BYTES,
  MAX_WIDTH_PX,
  MIN_SCALE,
} from "./limits.js";
import { PROTOCOL_VERSION, RENDERER_VERSION } from "./version.js";

async function writeLine(value: unknown): Promise<void> {
  if (!process.stdout.write(`${JSON.stringify(value)}\n`)) {
    await once(process.stdout, "drain");
  }
}

async function main(): Promise<void> {
  const handler = await RequestHandler.create();
  await writeLine({
    protocol: PROTOCOL_VERSION,
    type: "ready",
    renderer: { name: "mathjax", version: RENDERER_VERSION },
    capabilities: { formats: ["svg"], displayModes: ["inline", "display"] },
    limits: {
      maxSourceBytes: MAX_SOURCE_BYTES,
      maxJsonLineBytes: MAX_JSON_LINE_BYTES,
      maxSvgBytes: MAX_SVG_BYTES,
      maxWidthPx: MAX_WIDTH_PX,
      maxHeightPx: MAX_HEIGHT_PX,
      minScale: MIN_SCALE,
      maxScale: MAX_SCALE,
    },
  });

  const lines = createInterface({
    input: process.stdin,
    crlfDelay: Number.POSITIVE_INFINITY,
  });
  for await (const line of lines) {
    await writeLine(await handler.handleLine(line));
  }
}

try {
  await main();
} catch (error: unknown) {
  const name = error instanceof Error ? error.name : "UnknownError";
  process.stderr.write(`mathjax worker terminated with ${name}\n`);
  process.exitCode = 1;
}
