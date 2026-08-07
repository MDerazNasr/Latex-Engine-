import { appendFileSync, existsSync, writeFileSync } from "node:fs";
import { createInterface } from "node:readline";

const mode = process.argv[2] ?? "healthy";
const marker = process.argv[3];

const ready = {
  protocol: 1,
  type: "ready",
  renderer: {
    name: "mathjax",
    version: mode === "bad-handshake" ? "9.9.9" : "0.1.0",
  },
  capabilities: { formats: ["svg"], displayModes: ["inline", "display"] },
  limits: {
    maxSourceBytes: 16384,
    maxJsonLineBytes: 65536,
    maxSvgBytes: 2097152,
    maxWidthPx: 4096,
    maxHeightPx: 2048,
    minScale: 0.5,
    maxScale: 4,
  },
};

process.stdout.write(`${JSON.stringify(ready)}\n`);

const lines = createInterface({
  input: process.stdin,
  crlfDelay: Number.POSITIVE_INFINITY,
});

for await (const line of lines) {
  const request = JSON.parse(line);
  if (mode === "hang") {
    await new Promise(() => {});
  }
  if (mode === "slow") {
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  if (mode === "crash-once" && marker && !existsSync(marker)) {
    writeFileSync(marker, "crashed\n", "utf8");
    process.exit(7);
  }
  if (mode === "malformed") {
    process.stdout.write("not json\n");
    continue;
  }
  if (mode === "invalid-tex") {
    respond({
      protocol: 1,
      id: request.id,
      ok: false,
      error: {
        code: "INVALID_TEX",
        message: "Math input is invalid",
        retryable: false,
      },
    });
    continue;
  }
  if (mode === "healthy-count" && marker) {
    appendFileSync(marker, "render\n", "utf8");
  }
  respond({
    protocol: 1,
    id: mode === "wrong-id" ? "eq-wrong" : request.id,
    ok: true,
    result: {
      svgUtf8: '<svg xmlns="http://www.w3.org/2000/svg"></svg>',
      widthPx: 64,
      heightPx: 32,
      baselinePx: 24,
      accessibilityText: request.params.source,
    },
  });
}

function respond(value) {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}
