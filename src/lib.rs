use latex2mathml::{DisplayStyle, latex_to_mathml};
use once_cell::sync::Lazy;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd, html};
use regex::Regex;

use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use wasm_bindgen::prelude::*;

// --- 1. 静态资源预加载 ---
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    let syntax_dump = include_bytes!("../assets/syntax.packdump");
    syntect::dumps::from_binary(syntax_dump)
});

// 预编译正则：GitHub 风格数学公式检测
// 规则：
// - $$...$$ : 块级公式
// - $...$   : 行内公式，要求：
//   1. 第一个 $ 前是字符串开头或空格
//   2. 第二个 $ 后是字符串结尾或空格
// 这样可区分 $5/month (货币) 和 $1+1=2$ (公式)
// 正则不包含边界，边界检查在代码中进行
static MATH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\$\$)([\s\S]+?)(\$\$)|\$([^$\s][^$]*?)\$").unwrap()
});

// --- 2. 辅助函数：数学渲染 ---
fn render_math(latex: &str, display_mode: bool) -> String {
    let style = if display_mode {
        DisplayStyle::Block
    } else {
        DisplayStyle::Inline
    };
    match latex_to_mathml(latex, style) {
        Ok(mathml) => mathml,
        Err(_) => {
            format!(
                r#"<span class="math-error" style="color:red">Error: {}</span>"#,
                latex
            )
        }
    }
}

// --- 3. 核心导出函数 ---
#[wasm_bindgen]
pub fn render_markdown(markdown_input: &str) -> String {
    // A. 开启 Markdown 选项 (GFM)
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    // B. 创建解析器
    let parser = Parser::new_ext(markdown_input, options);

    // C. 状态机变量
    let mut in_code_block = false;
    let mut current_lang = None;
    let mut code_buffer = String::new();

    let mut in_display_math = false;
    let mut math_buffer = String::new();

    // D. 事件流处理
    let mut new_events = Vec::new();

    for event in parser {
        // --- State 1: Inside Code Block ---
        // 必须优先处理，捕获所有内容
        if in_code_block {
            match event {
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                    let lang = current_lang.as_deref().unwrap_or("text");

                    if lang == "math" || lang == "latex" {
                        let math_html = render_math(&code_buffer, true);
                        new_events.push(Event::Html(CowStr::from(math_html)));
                    } else {
                        let ss = &SYNTAX_SET;
                        let syntax = ss
                            .find_syntax_by_token(lang)
                            .unwrap_or_else(|| ss.find_syntax_plain_text());

                        let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
                            syntax,
                            ss,
                            ClassStyle::Spaced,
                        );

                        for line in LinesWithEndings::from(&code_buffer) {
                            let _ = html_generator.parse_html_for_line_which_includes_newline(line);
                        }

                        let highlighted_code = html_generator.finalize();
                        let html = format!(
                            r#"<pre><code class="language-{}">{}</code></pre>"#,
                            lang, highlighted_code
                        );
                        new_events.push(Event::Html(CowStr::from(html)));
                    }
                    code_buffer.clear();
                }
                Event::Text(text) => code_buffer.push_str(&text),
                Event::SoftBreak | Event::HardBreak => code_buffer.push('\n'),
                _ => {} // 忽略其他标记，代码块内只保留文本
            }
            continue;
        }

        // --- State 2: Inside Display Math Block ($$) ---
        // 处理多行公式块
        if in_display_math {
            match event {
                Event::Text(text) => {
                    if let Some(idx) = text.find("$$") {
                        // 结束 Display Math
                        math_buffer.push_str(&text[..idx]);
                        let math_html = render_math(&math_buffer, true);
                        new_events.push(Event::Html(CowStr::from(math_html)));

                        math_buffer.clear();
                        in_display_math = false;

                        // 剩下的部分作为普通文本处理
                        if idx + 2 < text.len() {
                            new_events.push(Event::Text(CowStr::from(text[idx + 2..].to_string())));
                        }
                    } else {
                        math_buffer.push_str(&text);
                    }
                }
                Event::SoftBreak | Event::HardBreak => math_buffer.push('\n'),
                _ => {}
            }
            continue;
        }

        // --- State 3: Normal State ---
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                current_lang = match kind {
                    CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                    CodeBlockKind::Indented => None,
                };
            }

            // --- 核心修改: 使用正则处理 Text 中的数学公式 ---
            Event::Text(text) => {
                // 1. [修复] 检查是否是多行公式块的开始 ($$)
                // 如果当前文本行纯粹是 "$$"，则切换到 display_math 模式。
                // 这解决了 test_math_block 失败的问题。
                if text.trim() == "$$" {
                    in_display_math = true;
                    continue;
                }

                // 如果没有 $ 符号，直接跳过正则，提升性能
                if !text.contains('$') {
                    new_events.push(Event::Text(text));
                    continue;
                }

                let mut last_end = 0;

                // 遍历所有正则匹配项 (处理行内公式和单行块级公式)
                for cap in MATH_REGEX.captures_iter(&text) {
                    let match_start = cap.get(0).unwrap().start();
                    let match_end = cap.get(0).unwrap().end();

                    // 1. 将匹配项之前的普通文本 push 进去
                    if match_start > last_end {
                        new_events.push(Event::Text(CowStr::from(
                            text[last_end..match_start].to_string(),
                        )));
                    }

                    // 2. 判断是 $$ 还是 $
                    if let Some(content) = cap.get(2) {
                        // 匹配到了 $$ (Group 2 是内容)
                        let math_html = render_math(content.as_str(), true);
                        new_events.push(Event::Html(CowStr::from(math_html)));
                    } else if let Some(content) = cap.get(4) {
                        // 匹配到了 $ (Group 4 是内容)
                        // GitHub 风格边界检查（扩展支持中文）：
                        // - 前缀：字符串开头、空白字符、或非 ASCII 字符（如中文）
                        // - 后缀：字符串结尾、空白字符、标点符号、或非 ASCII 字符
                        let prev_char = text[..match_start].chars().last();
                        let is_valid_prefix = match_start == 0 
                            || prev_char.map(|c| c.is_whitespace() || !c.is_ascii_alphanumeric()).unwrap_or(false);
                        
                        let next_char = text[match_end..].chars().next();
                        let is_valid_suffix = match_end == text.len()
                            || next_char.map(|c| {
                                c.is_whitespace() 
                                    || !c.is_ascii_alphanumeric()
                                    || matches!(c, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '"' | '\'' | '/')
                            }).unwrap_or(false);
                        
                        if is_valid_prefix && is_valid_suffix {
                            // 是有效的公式，渲染它
                            let math_html = render_math(content.as_str(), false);
                            new_events.push(Event::Html(CowStr::from(math_html)));
                        } else {
                            // 不满足边界条件（如 $5/month），当作普通文本
                            new_events.push(Event::Text(CowStr::from(
                                cap.get(0).unwrap().as_str().to_string(),
                            )));
                        }
                    }

                    last_end = match_end;
                }

                // 3. 将剩余的普通文本 push 进去
                if last_end < text.len() {
                    new_events.push(Event::Text(CowStr::from(text[last_end..].to_string())));
                }
            }

            _ => new_events.push(event),
        }
    }

    // E. 最终渲染
    let mut html_output = String::new();
    html::push_html(&mut html_output, new_events.into_iter());
    html_output
}
