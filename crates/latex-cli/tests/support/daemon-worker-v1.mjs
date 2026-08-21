import { writeFileSync } from "node:fs";
import { createInterface } from "node:readline";

const exitMarker = process.env.LATEX_DAEMON_EXIT_MARKER;
process.on("exit", () => {
  if (exitMarker) {
    writeFileSync(exitMarker, "stopped\n", "utf8");
  }
});

const ready = {
  protocol: 1,
  type: "ready",
  renderer: { name: "mathjax", version: "0.1.0" },
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
  const svg =
    '<svg xmlns="http://www.w3.org/2000/svg" width="4px" height="2px" role="img" focusable="false" viewBox="0 0 4 2" style="color: #e6edf3;"><path fill="currentColor" d="M0 0L4 0L4 2L0 2Z"/></svg>';
  const response = {
    protocol: 1,
    id: request.id,
    ok: true,
    result: {
      svgUtf8: svg,
      widthPx: 64,
      heightPx: 32,
      baselinePx: 24,
      accessibilityText: "rendered math",
    },
  };
  process.stdout.write(`${JSON.stringify(response)}\n`);
}
