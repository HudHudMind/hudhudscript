//! External tests for hudhudscript-lsp (moved from inline #[cfg(test)] blocks)

use hudhudscript_lsp::{
    collect_definitions, completion_kind_to_lsp, extract_symbols, hover_word_at_position,
    identifier_at_position, parse_diagnostics, position_to_offset, span_to_range, CompletionKind,
    CompletionProvider, CursorContext, Document, HudHudServerCapabilities,
};
use tower_lsp::lsp_types::{CompletionItemKind, DiagnosticSeverity, Position};

// ── lib.rs tests ────────────────────────────────────────────────────

#[test]
fn test_server_capabilities_defaults() {
    let caps = HudHudServerCapabilities::default();
    assert!(caps.completion);
    assert!(caps.hover);
    assert!(caps.goto_definition);
    assert!(caps.formatting);
    assert!(caps.references);
}

#[test]
fn test_document_creation() {
    let doc = Document::new("file:///test.hudhud".to_string(), String::new());
    assert_eq!(doc.version(), 0);
}

// ── document.rs tests ───────────────────────────────────────────────

#[test]
fn test_document() {
    let mut doc = Document::new("file:///test.hudhud".to_string(), "test".to_string());
    assert_eq!(doc.version(), 0);
    doc.update("new text".to_string());
    assert_eq!(doc.version(), 1);
}

// ── completion.rs tests ─────────────────────────────────────────────

#[test]
fn test_completion_top_level() {
    let provider = CompletionProvider::new();
    let items = provider.complete("", 0);
    assert!(!items.is_empty());
    // Should contain top-level keywords
    assert!(items.iter().any(|i| i.label == "agent"));
    assert!(items.iter().any(|i| i.label == "function"));
}

#[test]
fn test_context_detection_agent_body() {
    let provider = CompletionProvider::new();
    let text = "agent MyAgent {\n  ";
    let ctx = provider.determine_context(text, text.len());
    assert_eq!(ctx, CursorContext::AgentBody);
}

#[test]
fn test_context_detection_dot_access() {
    let provider = CompletionProvider::new();
    let text = "let x = Math.";
    let ctx = provider.determine_context(text, text.len());
    assert_eq!(
        ctx,
        CursorContext::MemberAccess {
            object: "Math".to_string()
        }
    );
}

#[test]
fn test_context_detection_subject_body() {
    let provider = CompletionProvider::new();
    let text = "subject Player {\n  ";
    let ctx = provider.determine_context(text, text.len());
    assert_eq!(ctx, CursorContext::SubjectBody);
}

#[test]
fn test_completion_member_access_math() {
    let provider = CompletionProvider::new();
    let text = "let x = Math.";
    let items = provider.complete(text, text.len());
    assert!(items.iter().any(|i| i.label == "PI"));
    assert!(items.iter().any(|i| i.label == "sqrt"));
}

// ── definition.rs tests ─────────────────────────────────────────────

#[test]
fn test_collect_definitions_function() {
    let source = "function greet(name) { return name; }";
    let defs = collect_definitions(source);
    assert!(
        defs.iter().any(|d| d.name == "greet"),
        "Should find definition for 'greet'"
    );
}

#[test]
fn test_collect_definitions_variable() {
    let source = "let x = 42;\nconst y = 100;";
    let defs = collect_definitions(source);
    assert!(defs.iter().any(|d| d.name == "x"));
    assert!(defs.iter().any(|d| d.name == "y"));
}

#[test]
fn test_goto_definition_not_found() {
    use hudhudscript_lsp::goto_definition;
    use tower_lsp::lsp_types::Url;
    let uri = Url::parse("file:///test.hudhud").unwrap();
    let source = "let x = 42;";
    let result = goto_definition(&uri, source, Position::new(0, 10));
    // "42" is not a definition
    assert!(result.is_none());
}

// ── hover.rs tests ──────────────────────────────────────────────────

#[test]
fn test_word_at_position() {
    let source = "let myVar = 42;";
    let word = hover_word_at_position(source, Position::new(0, 5));
    assert_eq!(word, Some("myVar".to_string()));
}

#[test]
fn test_word_at_position_keyword() {
    let source = "let myVar = 42;";
    let word = hover_word_at_position(source, Position::new(0, 0));
    assert_eq!(word, Some("let".to_string()));
}

#[test]
fn test_hover_builtin_keyword() {
    use hudhudscript_lsp::hover_at;
    let source = "agent MyAgent { }";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn test_hover_user_defined_var() {
    use hudhudscript_lsp::hover_at;
    let source = "let greeting = \"hello\";";
    let hover = hover_at(source, Position::new(0, 5));
    assert!(hover.is_some());
}

// ── server.rs tests ─────────────────────────────────────────────────

#[test]
fn test_parse_diagnostics_valid() {
    // An empty program produces no diagnostics.
    let diags = parse_diagnostics("");
    assert!(diags.is_empty());
}

#[test]
fn test_parse_diagnostics_invalid() {
    // Garbage input should produce at least one diagnostic.
    let diags = parse_diagnostics("@@@@invalid@@@@");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].severity, Some(DiagnosticSeverity::ERROR));
}

#[test]
fn test_position_to_offset() {
    let text = "line one\nline two\nline three";
    // Line 0, col 0 → offset 0
    assert_eq!(position_to_offset(text, Position::new(0, 0)), 0);
    // Line 1, col 0 → offset 9 (after "line one\n")
    assert_eq!(position_to_offset(text, Position::new(1, 0)), 9);
    // Line 1, col 4 → offset 13
    assert_eq!(position_to_offset(text, Position::new(1, 4)), 13);
}

#[test]
fn test_completion_kind_mapping() {
    assert_eq!(
        completion_kind_to_lsp(CompletionKind::Keyword),
        CompletionItemKind::KEYWORD
    );
    assert_eq!(
        completion_kind_to_lsp(CompletionKind::Function),
        CompletionItemKind::FUNCTION
    );
    assert_eq!(
        completion_kind_to_lsp(CompletionKind::Field),
        CompletionItemKind::FIELD
    );
}

// ── symbols.rs tests ────────────────────────────────────────────────

#[test]
fn test_extract_symbols_function() {
    let source = "function greet(name) { return name; }";
    if let Ok(ast) = hudhudscript_parser::parse(source) {
        let symbols = extract_symbols(&ast);
        assert!(
            symbols.iter().any(|s| s.name == "greet"),
            "Should find function 'greet' in symbols"
        );
    }
}

#[test]
fn test_extract_symbols_variable() {
    let source = "let x = 42;";
    if let Ok(ast) = hudhudscript_parser::parse(source) {
        let symbols = extract_symbols(&ast);
        assert!(
            symbols.iter().any(|s| s.name == "x"),
            "Should find variable 'x' in symbols"
        );
    }
}

#[test]
fn test_extract_symbols_empty() {
    let symbols = extract_symbols(&[]);
    assert!(symbols.is_empty());
}

// ── references.rs tests ─────────────────────────────────────────────

#[test]
fn test_identifier_at_position() {
    use tower_lsp::lsp_types::Position as LspPosition;
    let src = "let foo = bar;";
    let id = identifier_at_position(
        src,
        &LspPosition {
            line: 0,
            character: 4,
        },
    );
    assert_eq!(id.as_deref(), Some("foo"));

    let id = identifier_at_position(
        src,
        &LspPosition {
            line: 0,
            character: 10,
        },
    );
    assert_eq!(id.as_deref(), Some("bar"));

    // On the '=' sign — not an identifier
    let id = identifier_at_position(
        src,
        &LspPosition {
            line: 0,
            character: 8,
        },
    );
    assert_eq!(id, None);
}

#[test]
fn test_span_to_range() {
    let span = hudhudscript_ast::Span::new(
        hudhudscript_ast::Position::new(1, 1, 0),
        hudhudscript_ast::Position::new(1, 4, 3),
    );
    let range = span_to_range(span);
    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 0);
    assert_eq!(range.end.line, 0);
    assert_eq!(range.end.character, 3);
}
