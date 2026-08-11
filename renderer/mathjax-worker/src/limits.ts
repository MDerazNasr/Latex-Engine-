/** Maximum UTF 8 bytes accepted for one TeX fragment. */
export const MAX_SOURCE_BYTES = 16 * 1024;

/** Maximum UTF 8 bytes accepted for one JSONL protocol line. */
export const MAX_JSON_LINE_BYTES = 64 * 1024;

/** Maximum UTF 8 bytes returned for one SVG image. */
export const MAX_SVG_BYTES = 2 * 1024 * 1024;

/** Maximum natural or scaled raster width requested by a client. */
export const MAX_WIDTH_PX = 4096;

/** Maximum raster height emitted by the renderer. */
export const MAX_HEIGHT_PX = 2048;

/** Maximum client scale accepted by the renderer. */
export const MAX_SCALE = 4;

/** Minimum client scale accepted by the renderer. */
export const MIN_SCALE = 0.5;
