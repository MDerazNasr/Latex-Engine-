declare module "mathjax" {
  export interface MathJaxAdaptor {
    firstChild(node: unknown): unknown;
    getAttribute(node: unknown, name: string): unknown;
    getStyle(node: unknown, name: string): string;
    kind(node: unknown): string;
    outerHTML(node: unknown): string;
    serializeXML(node: unknown): string;
    setStyle(node: unknown, name: string, value: string): void;
  }

  export interface MathJaxApi {
    startup: { adaptor: MathJaxAdaptor };
    tex2svgPromise(
      source: string,
      options: { display: boolean },
    ): Promise<unknown>;
  }

  interface MathJaxLoaderApi {
    init(
      configuration: Readonly<Record<string, unknown>>,
    ): Promise<MathJaxApi | null>;
  }

  const MathJaxLoader: MathJaxLoaderApi;
  export default MathJaxLoader;
}
