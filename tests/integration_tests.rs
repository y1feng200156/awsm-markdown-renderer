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
