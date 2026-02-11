use std::path::Path;
use syntect::dumps::dump_to_file;
use syntect::parsing::SyntaxSet;

fn main() {
    // 1. 加载默认语法集
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    
    // 2. 从文件夹加载额外的语法定义
    // 这会正确处理 extends 依赖关系
    let syntax_dir = Path::new("assets/syntaxes");
    
    if syntax_dir.exists() {
        println!("Loading syntaxes from: {:?}", syntax_dir);
        match builder.add_from_folder(syntax_dir, true) {
            Ok(()) => println!("  ✓ Successfully loaded syntaxes from folder"),
            Err(e) => eprintln!("  ✗ Failed to load from folder: {}", e),
        }
    }
    
    // 3. 构建并导出
    let syntax_set = builder.build();
    println!("\nTotal syntaxes: {}", syntax_set.syntaxes().len());
    
    // 验证 TypeScript 是否加载成功
    if let Some(ts) = syntax_set.find_syntax_by_token("typescript") {
        println!("✓ TypeScript syntax found: {}", ts.name);
    } else {
        println!("✗ TypeScript syntax NOT found");
    }
    
    if let Some(ts) = syntax_set.find_syntax_by_token("ts") {
        println!("✓ TS syntax found: {}", ts.name);
    } else {
        println!("✗ TS syntax NOT found");
    }
    
    // 验证 TSX
    if let Some(tsx) = syntax_set.find_syntax_by_token("tsx") {
        println!("✓ TSX syntax found: {}", tsx.name);
    } else {
        println!("✗ TSX syntax NOT found");
    }
    
    dump_to_file(&syntax_set, "assets/syntax.packdump").unwrap();
    println!("\nDumped syntax set to assets/syntax.packdump");
}
