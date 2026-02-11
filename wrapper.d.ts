// wrapper.d.ts
/**
 * Renders Markdown to HTML using the AWSM renderer.
 * @param markdown The markdown string to render.
 * @param wasmModule (Optional) For Cloudflare Workers, pass the imported .wasm module here.
 */
export function render_markdown(markdown: string, wasmModule?: any): Promise<string>;