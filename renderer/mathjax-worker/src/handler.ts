import { Buffer } from "node:buffer";

import { MAX_JSON_LINE_BYTES } from "./limits.js";
import {
  errorResponse,
  extractRequestId,
  successResponse,
  validateRequest,
  type WorkerResponse,
} from "./protocol.js";
import { MathJaxRenderer, RenderFailure } from "./renderer.js";

/** Processes validated render requests for a server or integration test. */
export class RequestHandler {
  private constructor(private readonly renderer: MathJaxRenderer) {}

  /** Initializes the handler after the local renderer is ready. */
  static async create(): Promise<RequestHandler> {
    return new RequestHandler(await MathJaxRenderer.create());
  }

  /** Handles one JSONL input line without exposing the source in errors. */
  async handleLine(line: string): Promise<WorkerResponse> {
    if (Buffer.byteLength(line, "utf8") > MAX_JSON_LINE_BYTES) {
      return errorResponse(null, {
        code: "INPUT_LIMIT_EXCEEDED",
        message: `Protocol line exceeds ${MAX_JSON_LINE_BYTES} UTF-8 bytes`,
        retryable: false,
      });
    }

    let value: unknown;
    try {
      value = JSON.parse(line) as unknown;
    } catch {
      return errorResponse(null, {
        code: "INVALID_JSON",
        message: "Protocol line is not valid JSON",
        retryable: false,
      });
    }

    const validation = validateRequest(value);
    if (!validation.ok) {
      return errorResponse(validation.id, validation.error);
    }

    try {
      return successResponse(
        validation.request.id,
        await this.renderer.render(validation.request.params),
      );
    } catch (error: unknown) {
      if (error instanceof RenderFailure) {
        return errorResponse(validation.request.id, error.publicError);
      }
      return errorResponse(extractRequestId(value), {
        code: "RENDER_FAILED",
        message: "Unexpected renderer failure",
        retryable: false,
      });
    }
  }
}
