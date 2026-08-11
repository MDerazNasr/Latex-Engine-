import { Buffer } from "node:buffer";

import { MAX_SVG_BYTES } from "./limits.js";
import type { WorkerError } from "./protocol.js";

const FORBIDDEN_PATTERNS: ReadonlyArray<RegExp> = [
  /<\s*script\b/i,
  /<\s*foreignObject\b/i,
  /<\s*(?:iframe|object|embed)\b/i,
  /\son[a-z]+\s*=/i,
  /\b(?:href|xlink:href|src)\s*=\s*["'](?!#)/i,
  /\burl\s*\(/i,
  /javascript\s*:/i,
  /<!\s*(?:DOCTYPE|ENTITY)\b/i,
];

/** Indicates that generated output violated a renderer safety invariant. */
export class SvgValidationError extends Error {
  /** Stable worker protocol error for this failure. */
  readonly publicError: WorkerError;

  constructor(publicError: WorkerError) {
    super(publicError.message);
    this.name = "SvgValidationError";
    this.publicError = publicError;
  }
}

/** Rejects SVG that exceeds size limits or contains active external content. */
export function validateGeneratedSvg(svg: string): string {
  if (!svg.startsWith("<svg ") || !svg.endsWith("</svg>")) {
    throw new SvgValidationError({
      code: "RENDER_FAILED",
      message: "Renderer did not produce a standalone SVG element",
      retryable: false,
    });
  }
  if (Buffer.byteLength(svg, "utf8") > MAX_SVG_BYTES) {
    throw new SvgValidationError({
      code: "OUTPUT_LIMIT_EXCEEDED",
      message: `SVG exceeds ${MAX_SVG_BYTES} UTF-8 bytes`,
      retryable: false,
    });
  }
  if (FORBIDDEN_PATTERNS.some((pattern) => pattern.test(svg))) {
    throw new SvgValidationError({
      code: "RENDER_FAILED",
      message: "Generated SVG contains forbidden active content",
      retryable: false,
    });
  }
  if (hasUnsafeControlCharacter(svg)) {
    throw new SvgValidationError({
      code: "RENDER_FAILED",
      message: "Generated SVG contains an unsafe control character",
      retryable: false,
    });
  }
  return svg;
}

function hasUnsafeControlCharacter(value: string): boolean {
  for (const character of value) {
    const codePoint = character.codePointAt(0) ?? 0;
    if (
      codePoint < 0x20 &&
      character !== "\n" &&
      character !== "\r" &&
      character !== "\t"
    ) {
      return true;
    }
  }
  return false;
}
