// wrapper.js
import init, * as wasmExports from "./awsm_markdown_renderer.js";

let initPromise = null;

/**
 * 通用的 Markdown 渲染函数
 * @param {string} markdown - Markdown 文本
 * @param {WebAssembly.Module | Request | Response | Promise} [wasmModule] - (可选) Cloudflare Workers 必须传入导入的 .wasm 模块
 */
export async function render_markdown(markdown, wasmModule) {
  // 单例模式：保证只初始化一次
  if (!initPromise) {
    // 情况 A: Cloudflare Workers (或者手动挡)
    // 用户显式传了 wasmModule 进来
    if (wasmModule) {
      initPromise = init(wasmModule);
    }
    // 情况 B: Next.js / 浏览器 (自动挡)
    // 不传参，init 会尝试 fetch 默认的 .wasm 文件
    else {
      initPromise = init();
    }
  }

  await initPromise;
  return wasmExports.render_markdown(markdown);
}
