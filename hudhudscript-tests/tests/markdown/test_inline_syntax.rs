use hudhudscript_markdown::syntax::*;
use hudhudscript_markdown::theme::dark_theme;

#[test]
fn language_detection() {
    assert_eq!(Language::from_tag("rust"), Language::Rust);
    assert_eq!(Language::from_tag("rs"), Language::Rust);
    assert_eq!(Language::from_tag("js"), Language::JavaScript);
    assert_eq!(Language::from_tag("Python"), Language::Python);
    assert_eq!(Language::from_tag("hhs"), Language::HudHudScript);
    assert_eq!(Language::from_tag("unknown"), Language::Generic);
}

#[test]
fn highlight_rust_keyword_inline() {
    let theme = dark_theme();
    let result = highlight_line("fn main() {", Language::Rust, &theme.syntax);
    assert!(result.contains("fn"));
    assert!(result.contains('\x1b'));
}

#[test]
fn highlight_string_literal_inline() {
    let theme = dark_theme();
    let result = highlight_line("let x = \"hello\";", Language::Rust, &theme.syntax);
    assert!(result.contains("hello"));
}

#[test]
fn highlight_comment_inline() {
    let theme = dark_theme();
    let result = highlight_line("// this is a comment", Language::Rust, &theme.syntax);
    assert!(result.contains("comment"));
}

#[test]
fn language_from_tag_typescript() {
    assert_eq!(Language::from_tag("typescript"), Language::TypeScript);
    assert_eq!(Language::from_tag("ts"), Language::TypeScript);
}

#[test]
fn language_from_tag_hudhudscript_inline() {
    assert_eq!(Language::from_tag("hudhudscript"), Language::HudHudScript);
    assert_eq!(Language::from_tag("hudhud"), Language::HudHudScript);
}

#[test]
fn language_from_tag_case_insensitive_inline() {
    assert_eq!(Language::from_tag("RUST"), Language::Rust);
    assert_eq!(Language::from_tag("JavaScript"), Language::JavaScript);
    assert_eq!(Language::from_tag("  Python  "), Language::Python);
}

#[test]
fn highlight_python_comment() {
    let theme = dark_theme();
    let result = highlight_line("# python comment", Language::Python, &theme.syntax);
    assert!(result.contains("python comment"));
    assert!(result.contains(theme.syntax.comment.fg));
}

#[test]
fn highlight_single_quote_string() {
    let theme = dark_theme();
    let result = highlight_line("let x = 'hello';", Language::JavaScript, &theme.syntax);
    assert!(result.contains("hello"));
    assert!(result.contains(theme.syntax.string.fg));
}

#[test]
fn highlight_number_literal_inline() {
    let theme = dark_theme();
    let result = highlight_line("42", Language::Rust, &theme.syntax);
    assert!(result.contains("42"));
    assert!(result.contains(theme.syntax.number.fg));
}

#[test]
fn highlight_function_call() {
    let theme = dark_theme();
    let result = highlight_line("foo()", Language::Rust, &theme.syntax);
    assert!(result.contains("foo"));
    assert!(result.contains(theme.syntax.function.fg));
}

#[test]
fn highlight_type_keyword_rust() {
    let theme = dark_theme();
    let result = highlight_line("let x: i32 = 0;", Language::Rust, &theme.syntax);
    assert!(result.contains("i32"));
    assert!(result.contains(theme.syntax.type_name.fg));
}

#[test]
fn highlight_operator_inline() {
    let theme = dark_theme();
    let result = highlight_line("x = y + z", Language::Rust, &theme.syntax);
    assert!(result.contains(theme.syntax.operator.fg));
}

#[test]
fn highlight_plain_identifier() {
    let theme = dark_theme();
    let result = highlight_line("myvar", Language::Rust, &theme.syntax);
    assert!(result.contains("myvar"));
    assert!(result.contains(theme.syntax.plain.fg));
}

#[test]
fn highlight_block_multiple_lines() {
    let theme = dark_theme();
    let code = "fn main() {\n    println!(\"hello\");\n}";
    let result = highlight_block(code, Language::Rust, &theme.syntax);
    assert!(result.contains("fn"));
    assert!(result.contains("hello"));
    let line_count = result.lines().count();
    assert_eq!(line_count, 3);
}

#[test]
fn highlight_escape_in_double_quote_string() {
    let theme = dark_theme();
    let result = highlight_line(r#"let s = "hello\"world";"#, Language::Rust, &theme.syntax);
    assert!(result.contains("hello"));
    assert!(result.contains(theme.syntax.string.fg));
}

#[test]
fn highlight_escape_in_single_quote_string() {
    let theme = dark_theme();
    let result = highlight_line(r"let c = 'a\'b';", Language::JavaScript, &theme.syntax);
    assert!(result.contains(theme.syntax.string.fg));
}

#[test]
fn highlight_generic_language_comment() {
    let theme = dark_theme();
    let result1 = highlight_line("// comment", Language::Generic, &theme.syntax);
    assert!(result1.contains(theme.syntax.comment.fg));
    let result2 = highlight_line("# comment", Language::Generic, &theme.syntax);
    assert!(result2.contains(theme.syntax.comment.fg));
}

#[test]
fn highlight_indented_comment() {
    let theme = dark_theme();
    let result = highlight_line("    // indented comment", Language::Rust, &theme.syntax);
    assert!(result.contains("indented comment"));
    assert!(result.contains(theme.syntax.comment.fg));
    assert!(result.starts_with("    "));
}

#[test]
fn highlight_typescript_type_keyword() {
    let theme = dark_theme();
    let result = highlight_line("let x: string = y;", Language::TypeScript, &theme.syntax);
    assert!(result.contains(theme.syntax.type_name.fg));
}

#[test]
fn highlight_hudhudscript_keyword() {
    let theme = dark_theme();
    let result = highlight_line("spawn agent", Language::HudHudScript, &theme.syntax);
    assert!(result.contains("spawn"));
    assert!(result.contains(theme.syntax.keyword.fg));
}

#[test]
fn highlight_empty_line_inline() {
    let theme = dark_theme();
    let result = highlight_line("", Language::Rust, &theme.syntax);
    assert_eq!(result, "");
}

#[test]
fn highlight_whitespace_and_brackets() {
    let theme = dark_theme();
    let result = highlight_line("  { } ( )", Language::Rust, &theme.syntax);
    assert!(result.contains("{"));
    assert!(result.contains("}"));
}

#[test]
fn rust_keywords_include_async() {
    assert!(Language::Rust.keywords().contains(&"async"));
    assert!(Language::Rust.keywords().contains(&"await"));
}

#[test]
fn python_keywords_include_def() {
    assert!(Language::Python.keywords().contains(&"def"));
    assert!(Language::Python.keywords().contains(&"class"));
}

#[test]
fn javascript_has_no_type_keywords() {
    assert!(Language::JavaScript.type_keywords().is_empty());
}

#[test]
fn python_type_keywords() {
    assert!(Language::Python.type_keywords().contains(&"int"));
    assert!(Language::Python.type_keywords().contains(&"float"));
}

#[test]
fn hudhudscript_type_keywords() {
    assert!(Language::HudHudScript.type_keywords().contains(&"string"));
    assert!(Language::HudHudScript.type_keywords().contains(&"any"));
}

#[test]
fn generic_has_no_type_keywords() {
    assert!(Language::Generic.type_keywords().is_empty());
}

#[test]
fn hudhudscript_line_comment_prefix() {
    assert_eq!(Language::HudHudScript.line_comment_prefix(), &["//"]);
}

#[test]
fn highlight_block_empty_inline() {
    let theme = dark_theme();
    let result = highlight_block("", Language::Rust, &theme.syntax);
    assert_eq!(result, "");
}

#[test]
fn highlight_block_single_line_inline() {
    let theme = dark_theme();
    let result = highlight_block("let x = 1;", Language::Rust, &theme.syntax);
    assert!(!result.contains('\n'));
    assert!(result.contains("let"));
}

#[test]
fn highlight_number_at_line_start() {
    let theme = dark_theme();
    let result = highlight_line("42 + x", Language::Rust, &theme.syntax);
    assert!(result.contains("42"));
    assert!(result.contains(theme.syntax.number.fg));
}

#[test]
fn highlight_underscore_identifier() {
    let theme = dark_theme();
    let result = highlight_line("my_var", Language::Rust, &theme.syntax);
    assert!(result.contains("my_var"));
}

#[test]
fn highlight_number_with_dot_and_underscore() {
    let theme = dark_theme();
    let result = highlight_line("3.14_f64", Language::Rust, &theme.syntax);
    assert!(result.contains("3.14_f64"));
    assert!(result.contains(theme.syntax.number.fg));
}

#[test]
fn highlight_python_type_keyword() {
    let theme = dark_theme();
    let result = highlight_line("x: int = 0", Language::Python, &theme.syntax);
    assert!(result.contains("int"));
    assert!(result.contains(theme.syntax.type_name.fg));
}

#[test]
fn highlight_hudhudscript_type() {
    let theme = dark_theme();
    let result = highlight_line("let x: any = 0", Language::HudHudScript, &theme.syntax);
    assert!(result.contains(theme.syntax.type_name.fg));
}

#[test]
fn generic_keywords_include_common() {
    let kws = Language::Generic.keywords();
    assert!(kws.contains(&"if"));
    assert!(kws.contains(&"function"));
    assert!(kws.contains(&"class"));
}

#[test]
fn rust_type_keywords_include_common_types() {
    let types = Language::Rust.type_keywords();
    assert!(types.contains(&"String"));
    assert!(types.contains(&"Vec"));
    assert!(types.contains(&"Option"));
    assert!(types.contains(&"Result"));
    assert!(types.contains(&"bool"));
}

#[test]
fn typescript_keywords_include_typescript_specific() {
    let kws = Language::TypeScript.keywords();
    assert!(kws.contains(&"typeof"));
    assert!(kws.contains(&"instanceof"));
}

#[test]
fn typescript_type_keywords_exist() {
    let types = Language::TypeScript.type_keywords();
    assert!(types.contains(&"string"));
    assert!(types.contains(&"number"));
    assert!(types.contains(&"boolean"));
    assert!(types.contains(&"Promise"));
}

#[test]
fn highlight_multiple_operators() {
    let theme = dark_theme();
    let result = highlight_line("a + b - c * d / e", Language::Rust, &theme.syntax);
    let op_count = result.matches(theme.syntax.operator.fg).count();
    assert!(op_count >= 4);
}

#[test]
fn highlight_tab_indented_comment() {
    let theme = dark_theme();
    let result = highlight_line("\t// tab comment", Language::Rust, &theme.syntax);
    assert!(result.contains("tab comment"));
    assert!(result.contains(theme.syntax.comment.fg));
    assert!(result.starts_with('\t'));
}
