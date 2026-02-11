import "./style.css";
import init, { render_markdown } from "../../pkg/awsm_markdown_renderer.js";

document.querySelector("#app").innerHTML = `
  <div class="card">
    <h1>AWSM Markdown Renderer</h1>
    <div class="input-group">
      <textarea id="markdown-input" placeholder="Type your markdown here..."># Hello World
This is **bold** and *italic*.
- Item 1
- Item 2
      </textarea>
    </div>
    <div class="output-group">
      <h3>Rendered Output:</h3>
      <div id="markdown-output" class="markdown-body"></div>
    </div>
  </div>
`;

async function run() {
  await init();

  const input = document.querySelector("#markdown-input");
  const output = document.querySelector("#markdown-output");

  const update = () => {
    const markdown = input.value;
    const html = render_markdown(markdown);
    output.innerHTML = html;
  };

  input.addEventListener("input", update);
  update();
}

run();
