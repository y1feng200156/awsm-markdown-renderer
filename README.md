# AWSM Markdown Renderer

A high-performance, universal Markdown renderer written in Rust and compiled to WebAssembly (Wasm).

It is designed to run everywhere: **Browsers**, **Node.js**, **Next.js**, and **Cloudflare Workers**.

## Features

- **Universal Support**: Works seamlessly in Next.js (Edge/Node runtimes) and Cloudflare Workers without complex configuration.
- **Fast**: Built with Rust and `pulldown-cmark`.
- **Zero-Config Initialization**: Auto-initializes Wasm in standard environments.
- **Syntax Highlighting**: Uses `syntect` for compile-time generated syntax dumping (no huge JS runtime bundles).
- **Math Support**: Renders LaTeX to MathML using `latex2mathml` (Validation compatible).
- **GFM Support**: Tables, Strikethrough, Tasklists, Footnotes.

## Installation

```bash
npm install @y1feng200156/awsm-markdown-renderer

```

## Usage

### 1. Next.js / React / Browser (Standard)

In standard environments (like Next.js Server Components, Client Components, or plain HTML), the renderer automatically handles the Wasm initialization.

```typescript
import { render_markdown } from '@y1feng200156/awsm-markdown-renderer';

async function main() {
    const markdown = `
# Hello World
This is **bold** text.
    `;

    // Just call it! No need to manually init().
    const html = await render_markdown(markdown);
    
    console.log(html);
}

main();

```

**Note for Next.js:** Ensure `asyncWebAssembly` is enabled in your `next.config.mjs`.

### 2. Adding Styles for Code Blocks

The renderer generates syntax-highlighted code blocks with CSS classes, but you need to include the styles to see the colors. We provide a ready-to-use CSS file with CSS variables for easy customization.

#### Option A: Import the CSS file (Recommended)

```typescript
// In your app entry file or layout
import '@y1feng200156/awsm-markdown-renderer/code-highlight.css';
```

#### Option B: Copy CSS variables to your own stylesheet

If you prefer to customize the theme, copy the CSS variables from `code-highlight.css` and adjust the colors to match your design.

#### Available CSS Variables

```css
:root {
  /* Code block container (light theme default) */
  --awsm-code-bg: #f6f8fa;
  --awsm-code-fg: #24292e;
  --awsm-code-border: #e1e4e8;
  --awsm-code-border-radius: 8px;
  --awsm-code-padding: 1rem;

  /* Syntax colors */
  --awsm-syntax-text: #24292e;
  --awsm-syntax-comment: #6a737d;
  --awsm-syntax-keyword: #d73a49;
  --awsm-syntax-string: #032f62;
  --awsm-syntax-number: #005cc5;
  --awsm-syntax-function: #6f42c1;
  --awsm-syntax-variable: #e36209;
  --awsm-syntax-type: #005cc5;
  /* ... and more */
}
```

#### Dark Theme Support

Add `data-theme="dark"` to your HTML or a parent element to enable the dark theme:

```html
<html data-theme="dark">
```

### 3. Cloudflare Workers (Edge)

Cloudflare Workers require you to explicitly import the `.wasm` file and pass it to the renderer.

```typescript
// worker.ts
import { render_markdown } from '@y1feng200156/awsm-markdown-renderer';

// Import the WASM file explicitly (Wrangler handles the bundling)
import wasm from '@y1feng200156/awsm-markdown-renderer/awsm_markdown_renderer_bg.wasm';

export default {
  async fetch(request, env, ctx) {
    const markdown = "# Hello from Edge!";
    
    // Pass the wasm module as the second argument
    const html = await render_markdown(markdown, wasm);

    return new Response(html, {
      headers: { "content-type": "text/html" },
    });
  },
};

```

## Building Locally

Requirements:

* Rust (latest stable)
* `wasm-pack`

```bash
# 1. Install wasm-pack (if not installed)
curl [https://rustwasm.github.io/wasm-pack/installer/init.sh](https://rustwasm.github.io/wasm-pack/installer/init.sh) -sSf | sh

# 2. Build the Wasm module
wasm-pack build --target web --scope y1feng200156

# 3. Run Post-Processing
# This step injects the wrapper and updates package.json for universal support
cargo run --bin post_process

```

## License

MIT