import fs from "fs/promises";
import path from "path";
import { fileURLToPath } from "url";
import init, { render_markdown } from "../../pkg/awsm_markdown_renderer.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const wasmPath = path.resolve(
  __dirname,
  "../../pkg/awsm_markdown_renderer_bg.wasm",
);

console.log("Loading WASM from:", wasmPath);

try {
  const wasmBuffer = await fs.readFile(wasmPath);
  // Initialize the WASM module with the buffer directly
  // This bypasses the default fetch() behavior which doesn't work in Node.js for local files
  await init(wasmBuffer);

  const markdown =
    "# Hello Node.js\nThis is rendered by **awsm_markdown_renderer**!\n\n- It supports lists\n- And other markdown features $E=mc^2$ \n\n $$E=mc^2$$";
  const html = render_markdown(markdown);

  console.log("\n--- Input Markdown ---");
  console.log(markdown);
  console.log("\n--- Rendered HTML ---");
  console.log(html);
} catch (error) {
  console.error("Error running demo:", error);
  process.exit(1);
}
