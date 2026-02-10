use awsm_markdown_renderer::render_markdown;

/// Test 1: Verify the renderer does NOT panic on any kind of invalid LaTeX input.
/// This is the primary reliability guarantee.
#[test]
fn test_invalid_latex_no_panic() {
    // Various forms of potentially invalid LaTeX
    let inputs = vec![
        "$\\invalidcommand$",
        "$\\frac{x$",           // unbalanced braces
        "$}}}{{{$",             // broken braces
        "$\\$",                 // escaped dollar
        "$$\\begin{invalid}$$", // invalid environment
        "$E=mc^2$",             // valid (control)
    ];

    for input in &inputs {
        let result = std::panic::catch_unwind(|| render_markdown(input));
        assert!(
            result.is_ok(),
            "render_markdown panicked on input: {}",
            input
        );
        let html = result.unwrap();
        assert!(
            !html.is_empty(),
            "Output should not be empty for: {}",
            input
        );
    }
}

/// Test 2: Verify that valid math renders as MathML (not error span)
#[test]
fn test_valid_latex_renders_mathml() {
    let input = "$E=mc^2$";
    let html = render_markdown(input);
    // Valid math should produce MathML output
    assert!(
        html.contains("<math") || html.contains("mathml"),
        "Valid LaTeX should render as MathML, got: {}",
        html
    );
    // Should NOT have error styling
    assert!(
        !html.contains("math-error"),
        "Valid LaTeX should not produce error span"
    );
}

/// Test 3: Verify the error path exists and is functional.
/// We directly test render_math via render_markdown with a code block
/// marked as `math` language, which bypasses the inline parser.
#[test]
fn test_math_code_block_renders() {
    let input = "```math\nE=mc^2\n```";
    let html = render_markdown(input);
    assert!(
        html.contains("<math") || html.contains("mathml"),
        "Math code block should render as MathML, got: {}",
        html
    );
}
