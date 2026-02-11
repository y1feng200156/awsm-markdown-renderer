# AWSM Markdown Renderer

A high-performance Markdown renderer written in Rust and compiled to WebAssembly (Wasm). It supports GitHub Flavored Markdown (GFM), Syntax Highlighting, and LaTeX Math rendering.

## Features

- **Fast**: Built with Rust and `pulldown-cmark`.
- **Syntax Highlighting**: Uses `syntect` for compile-time generated syntax dumping (no huge JS runtime bundles).
- **Math Support**: Renders LaTeX to MathML using `latex2mathml` (Validation compatible).
- **GFM Support**: Tables, Strikethrough, Tasklists, Footnotes.

## Installation

```bash
npm install @y1feng200156/awsm-markdown-renderer
```


## Usage

```javascript
import init, { render_markdown } from '@y1feng200156/awsm-markdown-renderer';

async function main() {
    // Initialize the Wasm module
    await init();

    const markdown = `
# Hello World

Here is some code:

\`\`\`rust
fn main() {
    println!("Hello");
}
\`\`\`

And some math: $E = mc^2$
    `;

    const html = render_markdown(markdown);
    console.log(html);
}

main();
```

## Building Locally

Requirements:
- Rust
- `wasm-pack`

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build
wasm-pack build --target web
```
