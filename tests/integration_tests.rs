use awsm_markdown_renderer::render_markdown;

#[test]
fn test_basic_markdown() {
    let input = "# Hello\n\nThis is a paragraph.";
    let html = render_markdown(input);
    assert!(html.contains("<h1>Hello</h1>"));
    assert!(html.contains("<p>This is a paragraph.</p>"));
}

#[test]
fn test_gfm_features() {
    let input = "
| Header |
| ------ |
| Cell   |

- [x] Task
~~strike~~
";
    let html = render_markdown(input);
    assert!(html.contains("<table>"), "Should render tables");
    assert!(
        html.contains("type=\"checkbox\""),
        "Should render task lists"
    );
    assert!(html.contains("checked"), "Task should be checked");
    assert!(
        html.contains("<del>strike</del>"),
        "Should render strikethrough"
    );
}

#[test]
fn test_code_highlighting() {
    let input = "
```rust
fn main() {}
```
";
    let html = render_markdown(input);
    assert!(
        html.contains("class=\"language-rust\""),
        "Should have language class"
    );
    // Check for syntect span generation (implies highlighting happened)
    // ClassStyle::Spaced generates "source rust"
    assert!(
        html.contains("<span class=\"source rust\">"),
        "Should have syntect classes"
    );
}

#[test]
fn test_unknown_language() {
    let input = "
```unknown-lang-123
some code
```
";
    let html = render_markdown(input);
    assert!(html.contains("class=\"language-unknown-lang-123\""));
    assert!(html.contains("some code"));
}

#[test]
fn test_math_inline() {
    let input = "Energy is $E=mc^2$.";
    let html = render_markdown(input);
    // Should NOT contain the original delimiters
    assert!(!html.contains("$E=mc^2$"));
    // Should contain MathML
    assert!(html.contains("<math"), "Should render MathML tag");
}

#[test]
fn test_math_block() {
    let input = r#"
$$
\int_0^\infty x^2 dx
$$
"#;
    let html = render_markdown(input);
    assert!(html.contains("<math"), "Should render MathML tag");
    assert!(
        html.contains("display=\"block\""),
        "Should be display block"
    );
}

#[test]
fn test_financial_amounts_edge_case() {
    // BUG REPRODUCTION: The current regex matches `$5 and $10` as math.
    // It shouldn't.
    let input = "The costs are $5 and $10 respectively.";
    let html = render_markdown(input);

    // If bug is present, this might look like: "The costs are <math>...</math> respectively."
    // We expect the original text or lightly formatted text, definitely NO math tags.
    assert!(
        !html.contains("<math"),
        "Financial text '$5 and $10' should NOT be rendered as math"
    );
    assert!(html.contains("$5"), "Should preserve $5");
    assert!(html.contains("$10"), "Should preserve $10");
}

#[test]
fn test_complex_mixed_content() {
    let input = r#"
Here is some implementation:

```rust
let cost = "$100"; // string literal with dollar
```

And here is the formula: $x + y = z$.
"#;
    let html = render_markdown(input);

    // Code block should be strict
    // Note: Syntect will insert spans between "let" (keyword) and "cost" (variable),
    // so we cannot check for "let cost" directly.
    assert!(html.contains("let"), "Code content 'let' missing");
    assert!(html.contains("cost"), "Code content 'cost' missing");

    // The $ inside code block should NOT be touched by the Text Regex
    // Note: pulldown-cmark emits Code events for the block content, so our Text regex shouldn't run on it.
    // BUT our code block handler clears the buffer.

    // Formula should be rendered
    assert!(html.contains("<math"), "Formula should render");
}

#[test]
fn test_multiple_inline_math_in_one_line() {
    let input = "If $a=1$ and $b=2$, then $c=3$.";
    let html = render_markdown(input);

    // 应该包含 3 个 math 标签
    let matches: Vec<_> = html.match_indices("<math").collect();
    assert_eq!(matches.len(), 3, "Should render all 3 inline formulas");

    // 确保中间的文本没有丢失
    assert!(
        html.contains(" and "),
        "Text between formulas should be preserved"
    );
    assert!(
        html.contains(", then "),
        "Text between formulas should be preserved"
    );
}

#[test]
fn test_math_inside_formatting() {
    let input = "This is **$E=mc^2$** and [$x$](http://example.com).";
    let html = render_markdown(input);

    // 验证粗体
    assert!(html.contains("<strong>"), "Should render bold");
    // 验证链接
    assert!(
        html.contains("<a href=\"http://example.com\">"),
        "Should render link"
    );
    // 验证数学公式依然存在
    assert!(
        html.contains("<math"),
        "Math inside formatting should render"
    );
}

#[test]
fn test_invalid_latex_handling() {
    // 缺少右括号，latex2mathml 应该会报错
    let input = "This matches wrong: $\\frac{1$";
    let html = render_markdown(input);

    assert!(
        html.contains("math-error"),
        "Should render error span for invalid latex"
    );
    assert!(html.contains("color:red"), "Error should be styled (red)");
    // 绝对不能包含 <math>，因为解析失败了
    assert!(
        !html.contains("<math"),
        "Invalid latex should not produce math tags"
    );
}

#[test]
fn test_unicode_and_chinese() {
    let input = "公式$x+y$的计算结果。";
    let html = render_markdown(input);

    assert!(html.contains("公式"), "Should preserve Chinese text before");
    assert!(
        html.contains("<math"),
        "Should render math adjacent to Chinese"
    );
    assert!(
        html.contains("的计算结果"),
        "Should preserve Chinese text after"
    );
}

#[test]
fn test_number_start_restriction() {
    // GitHub 风格边界判断：
    // - $1+1=2$ 应该渲染为公式（前后都有空格，边界满足）
    // - $5/month 应该保留为货币（后 / 不是空格，边界不满足）
    let input = "Equation $1+1=2$ should be math, but $1/day $5/month is currency.";
    let html = render_markdown(input);

    // $1+1=2$ 应该渲染为公式（边界满足）
    assert!(
        html.contains("<math"),
        "Formulas with proper boundaries should render as math"
    );
    assert!(
        !html.contains("$1+1=2$"),
        "$1+1=2$ should be converted to math, not preserved as text"
    );

    // $5/month 应该保留为货币（边界不满足，/ 不是空格）
    assert!(
        html.contains("$5/month"),
        "Should preserve $5/month as currency"
    );
}

#[test]
fn test_tsx_code_block() {
    let input = r#"```tsx
function ffff() {
  const i = 1;
}
```"#;
    let html = render_markdown(input);
    
    // 检查是否有 language-tsx 类
    assert!(html.contains("language-tsx"), "Should have language-tsx class, got: {}", html);
    
    // 检查代码内容是否保留
    assert!(html.contains("function"), "Should contain 'function' keyword");
    assert!(html.contains("const"), "Should contain 'const' keyword");
    
    // 检查是否有语法高亮（不是纯文本）
    assert!(
        html.contains("source tsx"),
        "Should have TypeScript/React syntax highlighting classes, got: {}", html
    );
}

#[test]
fn test_typescript_code_block() {
    let input = r#"```typescript
function def() {
  const i = 1;
}
```"#;
    let html = render_markdown(input);
    
    // 检查是否有 language-typescript 类
    assert!(html.contains("language-typescript"), "Should have language-typescript class, got: {}", html);
    
    // 检查是否有语法高亮（不是纯文本）
    assert!(
        html.contains("source ts"),
        "Should have TypeScript syntax highlighting classes, got: {}", html
    );
}

#[test]
fn test_jsx_code_block() {
    let input = r#"```jsx
function App() {
  return <div>Hello</div>;
}
```"#;
    let html = render_markdown(input);
    println!("\n=== JSX Input ===");
    println!("{}", input);
    println!("\n=== JSX Output ===");
    println!("{}", html);
    
    // 检查是否有 language-jsx 类
    assert!(html.contains("language-jsx"), "Should have language-jsx class");
    
    // 检查是否有语法高亮（不是纯文本）
    let has_highlighting = html.contains("source") && !html.contains("text plain");
    println!("Has syntax highlighting: {}", has_highlighting);
}

#[test]
fn test_js_code_block() {
    let input = r#"```js
function hello() {
  return "world";
}
```"#;
    let html = render_markdown(input);
    println!("\n=== JS Input ===");
    println!("{}", input);
    println!("\n=== JS Output ===");
    println!("{}", html);
    
    // 检查是否有 language-js 类
    assert!(html.contains("language-js"), "Should have language-js class");
}

#[test]
fn test_tsx_with_jsx() {
    let input = r#"```tsx
function App() {
  return <div className="app">Hello</div>;
}
```"#;
    let html = render_markdown(input);
    println!("\n=== TSX with JSX Input ===");
    println!("{}", input);
    println!("\n=== TSX with JSX Output ===");
    println!("{}", html);
    
    // 检查是否有语法高亮
    assert!(html.contains("source tsx"), "Should have tsx source class");
    
    // 检查 JSX 标签是否被正确处理（不应该被转义为 &lt;）
    // 注意：HTML 中 < 会被转义为 &lt;，但语法高亮应该能识别 JSX 标签
}

#[test]
fn test_tsx_with_types() {
    let input = r#"```tsx
interface Props {
  name: string;
  age: number;
}

function greet(props: Props): string {
  return `Hello ${props.name}`;
}

const App: React.FC<Props> = (props) => {
  return <div>{greet(props)}</div>;
};
```"#;
    let html = render_markdown(input);
    println!("\n=== TSX with Types Input ===");
    println!("{}", input);
    println!("\n=== TSX with Types Output ===");
    println!("{}", html);
    
    // 检查是否有 TSX 语法高亮
    assert!(html.contains("source tsx"), "Should have tsx source class");
    
    // 检查类型声明关键字
    assert!(html.contains("interface"), "Should contain 'interface' keyword");
    assert!(html.contains("string"), "Should contain 'string' type");
    assert!(html.contains("number"), "Should contain 'number' type");
    
    // 检查泛型语法 <Props>
    assert!(html.contains("Props"), "Should contain 'Props' type name");
}
