use hudhudscript_markdown::streaming::*;
use hudhudscript_markdown::theme::dark_theme;

#[test]
fn streaming_complete_blocks() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme);

    let out1 = renderer.push("# Hello\n\n");
    assert!(out1.contains("Hello"));

    let out2 = renderer.push("World.\n\n");
    assert!(out2.contains("World"));

    let final_out = renderer.finish();
    assert!(final_out.is_empty() || final_out.trim().is_empty());
}

#[test]
fn streaming_partial_code_block() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme);

    let out1 = renderer.push("```rust\nfn main() {\n");
    let out2 = renderer.push("    println!(\"hello\");\n");
    let out3 = renderer.push("}\n```\n\n");
    let combined = format!("{}{}{}", out1, out2, out3);
    assert!(combined.contains("main"));
}

#[test]
fn streaming_finish_flushes() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme);

    renderer.push("Some paragraph text");
    let out = renderer.finish();
    assert!(out.contains("paragraph"));
}

#[test]
fn streaming_finish_unclosed_code() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme);

    renderer.push("```python\nprint('hello')\n");
    let out = renderer.finish();
    assert!(out.contains("hello"));
}

#[test]
fn detect_state_normal() {
    assert_eq!(detect_state("hello world"), StreamState::Normal);
}

#[test]
fn detect_state_in_code() {
    assert_eq!(
        detect_state("```rust\nfn main() {}"),
        StreamState::InCodeBlock {
            lang: Some("rust".to_string())
        }
    );
}

#[test]
fn detect_state_closed_code() {
    assert_eq!(
        detect_state("```rust\nfn main() {}\n```"),
        StreamState::Normal
    );
}

#[test]
fn rerender_full() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme);
    renderer.push("# Title\n\nParagraph.\n");
    let full = renderer.rerender();
    assert!(full.contains("Title"));
    assert!(full.contains("Paragraph"));
}

#[test]
fn full_text_accumulates() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme);
    renderer.push("Hello ");
    renderer.push("World");
    assert_eq!(renderer.full_text(), "Hello World");
}

#[test]
fn finish_empty_buffer() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme);
    let out = renderer.finish();
    assert!(out.is_empty());
}

#[test]
fn finish_whitespace_only() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme);
    renderer.push("   \n  \n");
    let out = renderer.finish();
    assert!(out.is_empty() || out.trim().is_empty());
}

#[test]
fn detect_state_no_code_fence() {
    assert_eq!(detect_state("just plain text"), StreamState::Normal);
}

#[test]
fn detect_state_in_code_no_lang() {
    assert_eq!(
        detect_state("```\nsome code"),
        StreamState::InCodeBlock { lang: None }
    );
}

#[test]
fn detect_state_multiple_fences_even() {
    assert_eq!(
        detect_state("```rust\ncode\n```\nmore text"),
        StreamState::Normal
    );
}

#[test]
fn detect_state_multiple_fences_odd() {
    assert_eq!(
        detect_state("```rust\ncode\n```\n```python\nmore code"),
        StreamState::InCodeBlock {
            lang: Some("python".to_string())
        }
    );
}

#[test]
fn find_safe_cut_empty() {
    assert_eq!(find_safe_cut(""), 0);
}

#[test]
fn find_safe_cut_no_blank_line() {
    assert_eq!(find_safe_cut("just text"), 0);
}

#[test]
fn find_safe_cut_blank_line() {
    let text = "line1\n\nline2";
    let cut = find_safe_cut(text);
    assert!(cut > 0);
    assert!(cut <= 7);
}

#[test]
fn find_safe_cut_inside_code_block() {
    let text = "```\nblank line inside:\n\nstill code\n```\n\nafter";
    let cut = find_safe_cut(text);
    assert!(cut > 0);
}

#[test]
fn streaming_incremental_push() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme);
    let out = renderer.push("# Test\n\n");
    assert!(out.contains("Test"));

    let out2 = renderer.push("Some text");
    let out3 = renderer.finish();
    let combined = format!("{}{}", out2, out3);
    assert!(combined.contains("Some text") || combined.contains("text"));
}

#[test]
fn streaming_rerender_after_push() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme);
    renderer.push("# A\n\n# B\n\n");
    let full = renderer.rerender();
    assert!(full.contains("A"));
    assert!(full.contains("B"));
}

#[test]
fn line_offsets_iterator() {
    let text = "abc\ndef\nghi";
    let offsets: Vec<(usize, &str)> = LineOffsets::new(text).collect();
    assert_eq!(offsets.len(), 3);
    assert_eq!(offsets[0], (0, "abc"));
    assert_eq!(offsets[1], (4, "def"));
    assert_eq!(offsets[2], (8, "ghi"));
}

#[test]
fn line_offsets_single_line() {
    let text = "hello";
    let offsets: Vec<(usize, &str)> = LineOffsets::new(text).collect();
    assert_eq!(offsets.len(), 1);
    assert_eq!(offsets[0], (0, "hello"));
}

#[test]
fn line_offsets_empty() {
    let offsets: Vec<(usize, &str)> = LineOffsets::new("").collect();
    assert!(offsets.is_empty());
}

#[test]
fn streaming_finish_unclosed_code_no_rfind() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme);
    renderer.push("Some text before\n```rust\npartial code\n");
    let out = renderer.finish();
    assert!(out.contains("partial code") || out.contains("text before"));
}

#[test]
fn streaming_push_then_finish_renders_all() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme);
    let out1 = renderer.push("# Title\n\nPara text.\n\n");
    let out2 = renderer.push("More text here.");
    let out3 = renderer.finish();
    let combined = format!("{}{}{}", out1, out2, out3);
    assert!(combined.contains("Title"));
    assert!(combined.contains("More text here"));
}

#[test]
fn streaming_multiple_code_blocks() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme);
    let out = renderer.push("```js\nconsole.log('a');\n```\n\n```py\nprint('b')\n```\n\n");
    assert!(out.contains("console"));
    assert!(out.contains("print"));
}

#[test]
fn find_safe_cut_code_block_then_text() {
    let text = "```\ncode\n```\n\ntext after\n\n";
    let cut = find_safe_cut(text);
    assert!(cut > 0);
    assert!(text[..cut].contains("```"));
}

#[test]
fn detect_state_triple_fence_toggling() {
    let text = "```a\n```\n```b\n```\n```c\ncode";
    let state = detect_state(text);
    assert_eq!(
        state,
        StreamState::InCodeBlock {
            lang: Some("c".to_string())
        }
    );
}

#[test]
fn line_offsets_trailing_newline() {
    let text = "abc\ndef\n";
    let offsets: Vec<(usize, &str)> = LineOffsets::new(text).collect();
    assert_eq!(offsets.len(), 3);
    assert_eq!(offsets[0], (0, "abc"));
    assert_eq!(offsets[1], (4, "def"));
    assert_eq!(offsets[2], (8, ""));
}

#[test]
fn streaming_rerender_matches_full_render() {
    let theme = dark_theme();
    let mut renderer = StreamingRenderer::new(theme.clone());
    renderer.push("# Hello\n\nWorld.\n");
    let rerendered = renderer.rerender();
    let direct = hudhudscript_markdown::markdown::render(renderer.full_text(), &theme);
    assert_eq!(rerendered, direct);
}

#[test]
fn streaming_full_text_empty() {
    let theme = dark_theme();
    let renderer = StreamingRenderer::new(theme);
    assert_eq!(renderer.full_text(), "");
}

#[test]
fn find_safe_cut_blank_line_after_text() {
    let text = "paragraph\n\n";
    let cut = find_safe_cut(text);
    assert_eq!(cut, text.len());
}
