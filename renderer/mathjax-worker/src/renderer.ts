import MathJaxLoader, { type MathJaxApi } from "mathjax";

import { MAX_HEIGHT_PX } from "./limits.js";
import type { RenderParams, RenderResult, WorkerError } from "./protocol.js";
import { SvgValidationError, validateGeneratedSvg } from "./sanitize.js";

const EM_PX = 16;
const EX_PX = 8;

/** A render failure that can safely cross the worker protocol. */
export class RenderFailure extends Error {
  /** Stable public form of this failure. */
  readonly publicError: WorkerError;

  constructor(publicError: WorkerError) {
    super(publicError.message);
    this.name = "RenderFailure";
    this.publicError = publicError;
  }
}

/** Local TeX to SVG renderer with a fixed MathJax package policy. */
export class MathJaxRenderer {
  private constructor(private readonly mathjax: MathJaxApi) {}

  /** Initializes the locally installed MathJax component bundle. */
  static async create(): Promise<MathJaxRenderer> {
    const api = await MathJaxLoader.init({
      loader: { load: ["input/tex-base", "[tex]/ams", "output/svg"] },
      tex: {
        packages: ["base", "ams"],
        maxBuffer: 16 * 1024,
        maxMacros: 1_000,
      },
      svg: { fontCache: "none" },
    });
    if (api === null) {
      throw publicFailure(
        "RENDER_FAILED",
        "MathJax initialization returned no API",
      );
    }
    return new MathJaxRenderer(api);
  }

  /** Converts one validated TeX fragment into a bounded standalone SVG. */
  async render(params: RenderParams): Promise<RenderResult> {
    try {
      const container = await this.mathjax.tex2svgPromise(params.source, {
        display: params.displayMode,
      });
      const adaptor = this.mathjax.startup.adaptor;
      const svgNode = adaptor.firstChild(container);
      if (adaptor.kind(svgNode) !== "svg") {
        throw publicFailure(
          "RENDER_FAILED",
          "MathJax did not return an SVG element",
        );
      }
      if (containsMathError(adaptor.outerHTML(container))) {
        throw publicFailure("INVALID_TEX", "MathJax rejected the TeX fragment");
      }

      adaptor.setStyle(svgNode, "color", params.foreground);
      if (params.background !== "transparent") {
        adaptor.setStyle(svgNode, "background-color", params.background);
      }

      const naturalWidth = parseCssLength(
        adaptor.getAttribute(svgNode, "width"),
      );
      const naturalHeight = parseCssLength(
        adaptor.getAttribute(svgNode, "height"),
      );
      if (naturalWidth <= 0 || naturalHeight <= 0) {
        throw publicFailure(
          "RENDER_FAILED",
          "MathJax returned invalid SVG dimensions",
        );
      }

      const effectiveScale = Math.min(
        params.scale,
        params.maxWidthPx / naturalWidth,
        MAX_HEIGHT_PX / naturalHeight,
      );
      const widthPx = Math.max(1, Math.ceil(naturalWidth * effectiveScale));
      const heightPx = Math.max(1, Math.ceil(naturalHeight * effectiveScale));
      const verticalAlign = parseCssLength(
        adaptor.getStyle(svgNode, "vertical-align"),
      );
      const baselinePx = clamp(
        Math.round((naturalHeight + verticalAlign) * effectiveScale),
        0,
        heightPx,
      );
      const svgUtf8 = validateGeneratedSvg(adaptor.serializeXML(svgNode));

      return {
        svgUtf8,
        widthPx,
        heightPx,
        baselinePx,
        accessibilityText: params.source,
      };
    } catch (error: unknown) {
      if (error instanceof RenderFailure) {
        throw error;
      }
      if (error instanceof SvgValidationError) {
        throw new RenderFailure(error.publicError);
      }
      throw publicFailure(
        "RENDER_FAILED",
        "MathJax failed to render the TeX fragment",
      );
    }
  }
}

function containsMathError(container: string): boolean {
  return (
    container.includes('data-mml-node="merror"') ||
    container.includes("data-mjx-error") ||
    container.includes("mjx-error")
  );
}

function parseCssLength(value: unknown): number {
  if (typeof value !== "string") {
    return 0;
  }
  const match = /^(-?(?:\d+\.?\d*|\.\d+))(ex|em|px)$/.exec(value.trim());
  if (match === null) {
    return 0;
  }
  const amount = Number(match[1]);
  switch (match[2]) {
    case "ex":
      return amount * EX_PX;
    case "em":
      return amount * EM_PX;
    case "px":
      return amount;
    default:
      return 0;
  }
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(Math.max(value, minimum), maximum);
}

function publicFailure(
  code: WorkerError["code"],
  message: string,
): RenderFailure {
  return new RenderFailure({ code, message, retryable: false });
}
