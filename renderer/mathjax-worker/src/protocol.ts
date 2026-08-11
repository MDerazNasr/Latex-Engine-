import { Buffer } from "node:buffer";

import {
  MAX_SCALE,
  MAX_SOURCE_BYTES,
  MAX_WIDTH_PX,
  MIN_SCALE,
} from "./limits.js";
import { PROTOCOL_VERSION } from "./version.js";

/** Normalized parameters accepted by the renderer implementation. */
export interface RenderParams {
  source: string;
  displayMode: boolean;
  foreground: string;
  background: string;
  scale: number;
  maxWidthPx: number;
}

/** A validated render request. */
export interface RenderRequest {
  protocol: typeof PROTOCOL_VERSION;
  id: string;
  method: "render";
  params: RenderParams;
}

/** A successful SVG render result. */
export interface RenderResult {
  svgUtf8: string;
  widthPx: number;
  heightPx: number;
  baselinePx: number;
  accessibilityText: string;
}

/** Stable public errors that can cross the worker protocol. */
export interface WorkerError {
  code:
    | "INVALID_JSON"
    | "INVALID_REQUEST"
    | "INVALID_TEX"
    | "INPUT_LIMIT_EXCEEDED"
    | "OUTPUT_LIMIT_EXCEEDED"
    | "RENDER_FAILED";
  message: string;
  retryable: boolean;
  position?: number;
}

/** A worker response correlated to a request when an ID was valid. */
export type WorkerResponse =
  | {
      protocol: typeof PROTOCOL_VERSION;
      id: string | null;
      ok: true;
      result: RenderResult;
    }
  | {
      protocol: typeof PROTOCOL_VERSION;
      id: string | null;
      ok: false;
      error: WorkerError;
    };

export type ValidationResult =
  | { ok: true; request: RenderRequest }
  | { ok: false; id: string | null; error: WorkerError };

const ID_PATTERN = /^[A-Za-z0-9._:-]{1,128}$/;
const COLOR_PATTERN = /^#[0-9a-fA-F]{6}$/;

/** Extracts a safe correlation ID without trusting the remaining request. */
export function extractRequestId(value: unknown): string | null {
  if (
    !isRecord(value) ||
    typeof value.id !== "string" ||
    !ID_PATTERN.test(value.id)
  ) {
    return null;
  }
  return value.id;
}

/** Validates and normalizes an unknown protocol value. */
export function validateRequest(value: unknown): ValidationResult {
  const id = extractRequestId(value);
  if (!isRecord(value)) {
    return invalid(id, "Request must be a JSON object");
  }
  if (value.protocol !== PROTOCOL_VERSION) {
    return invalid(id, `Protocol must equal ${PROTOCOL_VERSION}`);
  }
  if (id === null) {
    return invalid(null, "Request ID must contain 1 to 128 safe characters");
  }
  if (value.method !== "render") {
    return invalid(id, "Method must equal render");
  }
  if (!isRecord(value.params)) {
    return invalid(id, "Params must be a JSON object");
  }

  const params = value.params;
  if (typeof params.source !== "string" || params.source.length === 0) {
    return invalid(id, "Source must be a non-empty string");
  }
  if (Buffer.byteLength(params.source, "utf8") > MAX_SOURCE_BYTES) {
    return {
      ok: false,
      id,
      error: {
        code: "INPUT_LIMIT_EXCEEDED",
        message: `Source exceeds ${MAX_SOURCE_BYTES} UTF-8 bytes`,
        retryable: false,
      },
    };
  }
  if (hasUnsafeControlCharacter(params.source)) {
    return invalid(id, "Source contains an unsupported control character");
  }

  const displayMode = params.displayMode ?? false;
  if (typeof displayMode !== "boolean") {
    return invalid(id, "displayMode must be a boolean");
  }

  const foreground = params.foreground ?? "#e6edf3";
  if (typeof foreground !== "string" || !COLOR_PATTERN.test(foreground)) {
    return invalid(id, "foreground must be a six-digit hexadecimal color");
  }

  const background = params.background ?? "transparent";
  if (
    typeof background !== "string" ||
    (background !== "transparent" && !COLOR_PATTERN.test(background))
  ) {
    return invalid(id, "background must be transparent or a hexadecimal color");
  }

  const scale = params.scale ?? 2;
  if (
    typeof scale !== "number" ||
    !Number.isFinite(scale) ||
    scale < MIN_SCALE ||
    scale > MAX_SCALE
  ) {
    return invalid(id, `scale must be between ${MIN_SCALE} and ${MAX_SCALE}`);
  }

  const maxWidthPx = params.maxWidthPx ?? 1200;
  if (
    typeof maxWidthPx !== "number" ||
    !Number.isInteger(maxWidthPx) ||
    maxWidthPx < 1 ||
    maxWidthPx > MAX_WIDTH_PX
  ) {
    return invalid(
      id,
      `maxWidthPx must be an integer between 1 and ${MAX_WIDTH_PX}`,
    );
  }

  return {
    ok: true,
    request: {
      protocol: PROTOCOL_VERSION,
      id,
      method: "render",
      params: {
        source: params.source,
        displayMode,
        foreground: foreground.toLowerCase(),
        background: background.toLowerCase(),
        scale,
        maxWidthPx,
      },
    },
  };
}

/** Creates a stable error response without exposing input source. */
export function errorResponse(
  id: string | null,
  error: WorkerError,
): WorkerResponse {
  return { protocol: PROTOCOL_VERSION, id, ok: false, error };
}

/** Creates a correlated success response. */
export function successResponse(
  id: string,
  result: RenderResult,
): WorkerResponse {
  return { protocol: PROTOCOL_VERSION, id, ok: true, result };
}

function invalid(id: string | null, message: string): ValidationResult {
  return {
    ok: false,
    id,
    error: { code: "INVALID_REQUEST", message, retryable: false },
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
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
