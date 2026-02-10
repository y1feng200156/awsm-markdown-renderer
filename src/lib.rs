use latex2mathml::{DisplayStyle, latex_to_mathml};
use once_cell::sync::Lazy;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd, html};

use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use wasm_bindgen::prelude::*;

// --- 1. 静态资源预加载 ---
// 这是一个昂贵的操作，我们只做一次。
// Lazy 确保它在 Wasm 模块加载时初始化，而不是每次 render 时。
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    let syntax_dump = include_bytes!("../assets/syntax.packdump");
    syntect::dumps::from_binary(syntax_dump)
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

            Event::Text(text) => {
                // 扫描 Text 寻找 $$ 或 $
                let mut last_idx = 0;
                let chars: Vec<char> = text.chars().collect();
                let mut i = 0;

                while i < chars.len() {
                    // Check $$
                    if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '$' {
                        // 发现 $$
                        // 1. Flush 之前的 text
                        if i > last_idx {
                            let t: String = chars[last_idx..i].iter().collect();
                            new_events.push(Event::Text(CowStr::from(t)));
                        }

                        // 2. 寻找闭合 $$ within THIS text node
                        let mut closed_in_line = None;
                        for j in (i + 2)..chars.len() {
                            if chars[j] == '$' && j + 1 < chars.len() && chars[j + 1] == '$' {
                                closed_in_line = Some(j);
                                break;
                            }
                        }

                        if let Some(end) = closed_in_line {
                            // 完整块 $$...$$
                            let math_content: String = chars[(i + 2)..end].iter().collect();
                            let math_html = render_math(&math_content, true);
                            new_events.push(Event::Html(CowStr::from(math_html)));

                            last_idx = end + 2;
                            i = last_idx;
                        } else {
                            // 跨行块 $$... (Opening)
                            in_display_math = true;
                            if i + 2 < chars.len() {
                                let remainder: String = chars[(i + 2)..].iter().collect();
                                math_buffer.push_str(&remainder);
                            }
                            last_idx = chars.len();
                            break;
                        }
                    }
                    // Check $ (Inline)
                    else if chars[i] == '$' {
                        if i + 1 < chars.len() && !chars[i + 1].is_whitespace() {
                            let mut end = None;
                            for j in (i + 1)..chars.len() {
                                if chars[j] == '$' {
                                    if j + 1 < chars.len() && chars[j + 1] == '$' {
                                        continue;
                                    }
                                    if chars[j - 1].is_whitespace() {
                                        continue;
                                    }
                                    end = Some(j);
                                    break;
                                }
                            }

                            if let Some(close_idx) = end {
                                if i > last_idx {
                                    let t: String = chars[last_idx..i].iter().collect();
                                    new_events.push(Event::Text(CowStr::from(t)));
                                }
                                let math_content: String =
                                    chars[(i + 1)..close_idx].iter().collect();
                                let math_html = render_math(&math_content, false);
                                new_events.push(Event::Html(CowStr::from(math_html)));

                                last_idx = close_idx + 1;
                                i = last_idx;
                                continue;
                            }
                        }
                    }
                    i += 1;
                }

                if last_idx < chars.len() {
                    let t: String = chars[last_idx..].iter().collect();
                    new_events.push(Event::Text(CowStr::from(t)));
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
