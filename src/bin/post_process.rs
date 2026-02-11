use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;

fn main() -> Result<()> {
    println!("🦀 Starting Rust post-processing...");

    // 获取项目根目录 (假设我们在根目录运行 cargo run)
    let root_dir = std::env::current_dir()?;
    let pkg_dir = root_dir.join("pkg");

    // 1. 检查 pkg 目录是否存在
    if !pkg_dir.exists() {
        anyhow::bail!("❌ 'pkg' directory not found. Did you run 'wasm-pack build' first?");
    }

    // 2. 复制 wrapper 文件 (假设这两个文件在项目根目录)
    // 如果你的 wrapper 文件也在 scripts 里，请修改这里的路径
    let wrapper_js = root_dir.join("wrapper.js");
    let wrapper_dts = root_dir.join("wrapper.d.ts");

    // 复制到 pkg 目录
    fs::copy(&wrapper_js, pkg_dir.join("wrapper.js"))
        .context(format!("Failed to copy wrapper.js from {:?}", wrapper_js))?;
    fs::copy(&wrapper_dts, pkg_dir.join("wrapper.d.ts")).context(format!(
        "Failed to copy wrapper.d.ts from {:?}",
        wrapper_dts
    ))?;

    println!("✅ Wrapper files copied.");

    // 3. 修改 pkg/package.json
    let pkg_json_path = pkg_dir.join("package.json");
    let json_content = fs::read_to_string(&pkg_json_path).context("Failed to read package.json")?;

    let mut json: Value = serde_json::from_str(&json_content)?;

    // 修改关键字段指向 wrapper
    json["main"] = "wrapper.js".into();
    json["module"] = "wrapper.js".into();
    json["types"] = "wrapper.d.ts".into();
    json["sideEffects"] = false.into();

    // 4. 写回文件
    fs::write(&pkg_json_path, serde_json::to_string_pretty(&json)?)
        .context("Failed to write updated package.json")?;

    println!("✅ package.json updated successfully! Ready to publish.");
    Ok(())
}
