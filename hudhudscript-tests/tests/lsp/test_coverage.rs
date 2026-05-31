//! Additional tests for hudhudscript-lsp to increase coverage toward 50%+.
//!
//! Covers: completion (agent/subject/task/block contexts, symbols_from_source,
//! add_statement_keywords, console member access, default member access),
//! hover (function, const, class, enum, agent, builtin keywords/globals, edge cases),
//! definition (collect_definitions for classes, enums, agents, nested blocks),
//! references (find_references walker), symbols (extract_symbols for decl variants),
//! server helpers (parse_diagnostics, position_to_offset edge cases, completion_kind_to_lsp).

use hudhudscript_lsp::{
    collect_definitions, completion_kind_to_lsp, extract_symbols, find_references, goto_definition,
    hover_at, hover_word_at_position, identifier_at_position, parse_diagnostics,
    position_to_offset, span_to_range, CompletionKind, CompletionProvider, CursorContext, Document,
};
use tower_lsp::lsp_types::{CompletionItemKind, Position, Url};

// ═══════════════════════════════════════════════════════════════════════════
// Completion — context detection edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn completion_context_task_body() {
    let provider = CompletionProvider::new();
    let text = "task DoWork {\n  ";
    let ctx = provider.determine_context(text, text.len());
    assert_eq!(ctx, CursorContext::TaskBody);
}

#[test]
fn completion_context_action_body() {
    let provider = CompletionProvider::new();
    let text = "action Run {\n  ";
    let ctx = provider.determine_context(text, text.len());
    assert_eq!(ctx, CursorContext::TaskBody); // action maps to "task" context
}

#[test]
fn completion_context_nested_block() {
    let provider = CompletionProvider::new();
    // A generic block inside an agent body
    let text = "agent A {\n  if (true) {\n    ";
    let ctx = provider.determine_context(text, text.len());
    assert_eq!(ctx, CursorContext::Block);
}

#[test]
fn completion_context_offset_beyond_text() {
    let provider = CompletionProvider::new();
    let text = "let x = 1;";
    // offset way past the end — should not panic, returns TopLevel
    let ctx = provider.determine_context(text, text.len() + 100);
    assert_eq!(ctx, CursorContext::TopLevel);
}

#[test]
fn completion_context_closing_braces_returns_to_top() {
    let provider = CompletionProvider::new();
    let text = "agent A {\n}\n";
    let ctx = provider.determine_context(text, text.len());
    assert_eq!(ctx, CursorContext::TopLevel);
}

// ═══════════════════════════════════════════════════════════════════════════
// Completion — complete() for various contexts
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn completion_agent_body_has_fields() {
    let provider = CompletionProvider::new();
    let text = "agent Bot {\n  ";
    let items = provider.complete(text, text.len());
    // Agent-specific fields
    assert!(items.iter().any(|i| i.label == "model"));
    assert!(items.iter().any(|i| i.label == "provider"));
    assert!(items.iter().any(|i| i.label == "instructions"));
    assert!(items.iter().any(|i| i.label == "tools"));
    assert!(items.iter().any(|i| i.label == "temperature"));
    assert!(items.iter().any(|i| i.label == "max_tokens"));
    // Also statement keywords
    assert!(items.iter().any(|i| i.label == "let"));
    assert!(items.iter().any(|i| i.label == "return"));
}

#[test]
fn completion_subject_body_has_fields() {
    let provider = CompletionProvider::new();
    let text = "subject Player {\n  ";
    let items = provider.complete(text, text.len());
    assert!(items.iter().any(|i| i.label == "state"));
    assert!(items.iter().any(|i| i.label == "can"));
    assert!(items.iter().any(|i| i.label == "intent"));
    assert!(items.iter().any(|i| i.label == "has"));
    assert!(items.iter().any(|i| i.label == "uses"));
    assert!(items.iter().any(|i| i.label == "memory"));
    assert!(items.iter().any(|i| i.label == "perception"));
}

#[test]
fn completion_task_body_has_statement_keywords() {
    let provider = CompletionProvider::new();
    let text = "task Work {\n  ";
    let items = provider.complete(text, text.len());
    assert!(items.iter().any(|i| i.label == "if"));
    assert!(items.iter().any(|i| i.label == "while"));
    assert!(items.iter().any(|i| i.label == "for"));
    assert!(items.iter().any(|i| i.label == "try"));
    assert!(items.iter().any(|i| i.label == "catch"));
    assert!(items.iter().any(|i| i.label == "throw"));
    assert!(items.iter().any(|i| i.label == "switch"));
    assert!(items.iter().any(|i| i.label == "match"));
    assert!(items.iter().any(|i| i.label == "await"));
    assert!(items.iter().any(|i| i.label == "async"));
    assert!(items.iter().any(|i| i.label == "break"));
    assert!(items.iter().any(|i| i.label == "continue"));
}

#[test]
fn completion_member_access_console() {
    // HudHudScript uses `log.info()` / `log.error()` / `log.warn()`, not `console.log()`
    // The LSP provides default member completions (toString, etc.) for any object
    let provider = CompletionProvider::new();
    let text = "log.";
    let items = provider.complete(text, text.len());
    assert!(items.iter().any(|i| i.label == "toString"));
}

#[test]
fn completion_member_access_unknown_object() {
    let provider = CompletionProvider::new();
    let text = "myArray.";
    let items = provider.complete(text, text.len());
    // Default methods for unknown objects
    assert!(items.iter().any(|i| i.label == "length"));
    assert!(items.iter().any(|i| i.label == "push"));
    assert!(items.iter().any(|i| i.label == "pop"));
    assert!(items.iter().any(|i| i.label == "map"));
    assert!(items.iter().any(|i| i.label == "filter"));
    assert!(items.iter().any(|i| i.label == "forEach"));
    assert!(items.iter().any(|i| i.label == "toString"));
}

#[test]
fn completion_symbols_from_source_function_and_var() {
    let provider = CompletionProvider::new();
    let text = "function greet(name) { return name; }\nlet x = 42;\n";
    // TopLevel completions should include user-defined symbols
    let items = provider.complete(text, text.len());
    assert!(items.iter().any(|i| i.label == "greet"));
    assert!(items.iter().any(|i| i.label == "x"));
}

#[test]
fn completion_symbols_from_source_const() {
    let provider = CompletionProvider::new();
    let text = "const PI = 3.14;\n";
    let items = provider.complete(text, text.len());
    assert!(items.iter().any(|i| i.label == "PI"));
}

#[test]
fn completion_default_is_same_as_new() {
    let a = CompletionProvider::new();
    let b = CompletionProvider::default();
    // Both should produce the same completions for the same input
    let items_a = a.complete("", 0);
    let items_b = b.complete("", 0);
    assert_eq!(items_a.len(), items_b.len());
}

// ═══════════════════════════════════════════════════════════════════════════
// Hover — builtin keywords coverage
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn hover_builtin_let() {
    let source = "let x = 1;";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_builtin_const() {
    let source = "const y = 2;";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_builtin_function_keyword() {
    let source = "function foo() {}";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_builtin_class_keyword() {
    let source = "class Foo {}";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_builtin_if_keyword() {
    // Hover over "if" keyword
    let source = "if (true) {}";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_builtin_while_keyword() {
    let source = "while (true) {}";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_builtin_for_keyword() {
    let source = "for (x in items) {}";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_builtin_return_keyword() {
    let source = "return 42;";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_builtin_import_keyword() {
    let source = "import { foo } from \"bar\";";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_builtin_true_false_null() {
    let source = "let a = true;\nlet b = false;\nlet c = null;";
    let h_true = hover_at(source, Position::new(0, 8));
    assert!(h_true.is_some());
    let h_false = hover_at(source, Position::new(1, 8));
    assert!(h_false.is_some());
    let h_null = hover_at(source, Position::new(2, 8));
    assert!(h_null.is_some());
}

#[test]
fn hover_builtin_print() {
    let source = "print(42);";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_builtin_math() {
    let source = "Math.PI;";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_builtin_enum_keyword() {
    let source = "enum Color { Red, Green, Blue }";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_builtin_match_keyword() {
    let source = "match x { }";
    let hover = hover_at(source, Position::new(0, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_builtin_async_await_spawn_send_receive() {
    for kw in &["async", "await", "spawn", "send", "receive"] {
        let source = format!("{} foo;", kw);
        let hover = hover_at(&source, Position::new(0, 0));
        assert!(hover.is_some(), "hover should resolve for builtin '{}'", kw);
    }
}

#[test]
fn hover_builtin_governance_keywords() {
    for kw in &[
        "constitution",
        "governance",
        "protocol",
        "strategy",
        "store",
        "remember",
        "recall",
    ] {
        let source = format!("{} foo;", kw);
        let hover = hover_at(&source, Position::new(0, 0));
        assert!(hover.is_some(), "hover should resolve for builtin '{}'", kw);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Hover — user-defined symbol hover
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn hover_user_function() {
    let source = "function greet(name) { return name; }\ngreet(\"world\");";
    // Hover over "greet" on the second line
    let hover = hover_at(source, Position::new(1, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_user_const_binding() {
    let source = "const MAX = 100;\nMAX;";
    let hover = hover_at(source, Position::new(1, 0));
    assert!(hover.is_some());
}

#[test]
fn hover_on_unknown_identifier() {
    let source = "let x = unknownThing;";
    // "unknownThing" is neither a declaration nor a builtin
    let hover = hover_at(source, Position::new(0, 10));
    assert!(hover.is_none());
}

#[test]
fn hover_word_at_position_out_of_bounds() {
    let source = "let x = 1;";
    // Line beyond source
    let word = hover_word_at_position(source, Position::new(5, 0));
    assert!(word.is_none());
    // Column beyond line length
    let word = hover_word_at_position(source, Position::new(0, 100));
    assert!(word.is_none());
}

#[test]
fn hover_word_at_position_on_operator() {
    let source = "x = y + z;";
    // Position on '=' (col 2)
    let word = hover_word_at_position(source, Position::new(0, 2));
    assert!(word.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// Definition — collect_definitions coverage
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn collect_definitions_class() {
    let source = "class Animal {}";
    let defs = collect_definitions(source);
    assert!(defs.iter().any(|d| d.name == "Animal"));
}

#[test]
fn collect_definitions_enum_and_variants() {
    let source = "enum Color { Red, Green, Blue }";
    let defs = collect_definitions(source);
    assert!(defs.iter().any(|d| d.name == "Color"));
    assert!(defs.iter().any(|d| d.name == "Red"));
    assert!(defs.iter().any(|d| d.name == "Green"));
    assert!(defs.iter().any(|d| d.name == "Blue"));
}

#[test]
fn collect_definitions_nested_function_body() {
    let source = "function outer() { let inner = 1; }";
    let defs = collect_definitions(source);
    assert!(defs.iter().any(|d| d.name == "outer"));
    assert!(defs.iter().any(|d| d.name == "inner"));
}

#[test]
fn collect_definitions_for_loop_variable() {
    let source = "for (item in items) { }";
    let defs = collect_definitions(source);
    assert!(defs.iter().any(|d| d.name == "item"));
}

#[test]
fn collect_definitions_if_else_branches() {
    let source = "if (true) { let a = 1; } else { let b = 2; }";
    let defs = collect_definitions(source);
    assert!(defs.iter().any(|d| d.name == "a"));
    assert!(defs.iter().any(|d| d.name == "b"));
}

#[test]
fn collect_definitions_unparseable_source() {
    let defs = collect_definitions("@@@ not valid code @@@");
    assert!(defs.is_empty());
}

#[test]
fn goto_definition_finds_function() {
    let uri = Url::parse("file:///test.hudhud").unwrap();
    let source = "function greet(name) { return name; }\ngreet(\"hi\");";
    // Hover over "greet" on line 1
    let result = goto_definition(&uri, source, Position::new(1, 0));
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// References — find_references coverage
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn find_references_variable_usage() {
    let uri = Url::parse("file:///test.hudhud").unwrap();
    let source = "let x = 1;\nlet y = x;";
    let refs = find_references(source, "x", &uri);
    // Should find at least 2 occurrences: declaration and usage
    assert!(
        refs.len() >= 2,
        "Expected at least 2 refs, got {}",
        refs.len()
    );
}

#[test]
fn find_references_function_call() {
    let uri = Url::parse("file:///test.hudhud").unwrap();
    let source = "function greet(name) { return name; }\ngreet(\"world\");";
    let refs = find_references(source, "greet", &uri);
    assert!(
        refs.len() >= 2,
        "Expected at least 2 refs for 'greet', got {}",
        refs.len()
    );
}

#[test]
fn find_references_no_match() {
    let uri = Url::parse("file:///test.hudhud").unwrap();
    let source = "let x = 1;";
    let refs = find_references(source, "nonexistent", &uri);
    assert!(refs.is_empty());
}

#[test]
fn find_references_unparseable() {
    let uri = Url::parse("file:///test.hudhud").unwrap();
    let refs = find_references("@@@ bad @@@", "x", &uri);
    assert!(refs.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// Symbols — extract_symbols for various decl types
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn extract_symbols_const() {
    let source = "const PI = 3.14;";
    if let Ok(ast) = hudhudscript_parser::parse(source) {
        let symbols = extract_symbols(&ast);
        assert!(symbols.iter().any(|s| s.name == "PI"));
        let pi = symbols.iter().find(|s| s.name == "PI").unwrap();
        assert_eq!(pi.kind, tower_lsp::lsp_types::SymbolKind::CONSTANT);
    }
}

#[test]
fn extract_symbols_class_with_methods() {
    let source = "class Point { constructor(x, y) { this.x = x; this.y = y; } }";
    let ast = hudhudscript_parser::parse(source);
    assert!(ast.is_ok(), "Failed to parse class: {:?}", ast.err());
    let ast = ast.unwrap();
    let symbols = extract_symbols(&ast);
    let class_sym = symbols.iter().find(|s| s.name == "Point");
    assert!(class_sym.is_some(), "Should find class 'Point' in symbols");
    let class_sym = class_sym.unwrap();
    assert_eq!(class_sym.kind, tower_lsp::lsp_types::SymbolKind::CLASS);
    // Should have children (constructor)
    assert!(class_sym.children.is_some());
    let children = class_sym.children.as_ref().unwrap();
    assert!(children.iter().any(|c| c.name == "constructor"));
}

#[test]
fn extract_symbols_enum_with_variants() {
    let source = "enum Color { Red, Green, Blue }";
    if let Ok(ast) = hudhudscript_parser::parse(source) {
        let symbols = extract_symbols(&ast);
        let enum_sym = symbols.iter().find(|s| s.name == "Color");
        assert!(enum_sym.is_some());
        let enum_sym = enum_sym.unwrap();
        assert_eq!(enum_sym.kind, tower_lsp::lsp_types::SymbolKind::ENUM);
        assert!(enum_sym.children.is_some());
        let children = enum_sym.children.as_ref().unwrap();
        assert_eq!(children.len(), 3);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Server helpers — additional edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn parse_diagnostics_valid_let_statement() {
    let diags = parse_diagnostics("let x = 42;");
    assert!(diags.is_empty());
}

#[test]
fn parse_diagnostics_valid_function() {
    let diags = parse_diagnostics("function foo() { return 1; }");
    assert!(diags.is_empty());
}

#[test]
fn position_to_offset_past_end_of_file() {
    let text = "abc";
    // Line 5 doesn't exist — should return text.len()
    let offset = position_to_offset(text, Position::new(5, 0));
    assert_eq!(offset, text.len());
}

#[test]
fn position_to_offset_col_past_end_of_line() {
    let text = "ab\ncd";
    // Line 0, col 100 — should clamp to end of line 0
    let offset = position_to_offset(text, Position::new(0, 100));
    assert_eq!(offset, 2); // "ab" has length 2
}

#[test]
fn completion_kind_to_lsp_all_variants() {
    assert_eq!(
        completion_kind_to_lsp(CompletionKind::Variable),
        CompletionItemKind::VARIABLE
    );
    assert_eq!(
        completion_kind_to_lsp(CompletionKind::Module),
        CompletionItemKind::MODULE
    );
    assert_eq!(
        completion_kind_to_lsp(CompletionKind::Snippet),
        CompletionItemKind::SNIPPET
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Document — additional coverage
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn document_uri_and_text_accessors() {
    let doc = Document::new("file:///foo.hud".to_string(), "hello world".to_string());
    assert_eq!(doc.uri(), "file:///foo.hud");
    assert_eq!(doc.text(), "hello world");
    assert_eq!(doc.version(), 0);
}

#[test]
fn document_multiple_updates_increment_version() {
    let mut doc = Document::new("file:///a.hud".to_string(), "v0".to_string());
    doc.update("v1".to_string());
    doc.update("v2".to_string());
    doc.update("v3".to_string());
    assert_eq!(doc.version(), 3);
    assert_eq!(doc.text(), "v3");
}

// ═══════════════════════════════════════════════════════════════════════════
// identifier_at_position — edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn identifier_at_position_line_out_of_range() {
    let source = "let x = 1;";
    let id = identifier_at_position(
        source,
        &Position {
            line: 10,
            character: 0,
        },
    );
    assert!(id.is_none());
}

#[test]
fn identifier_at_position_col_out_of_range() {
    let source = "let x = 1;";
    let id = identifier_at_position(
        source,
        &Position {
            line: 0,
            character: 100,
        },
    );
    assert!(id.is_none());
}

#[test]
fn identifier_at_position_underscore_identifier() {
    let source = "let my_var = 1;";
    let id = identifier_at_position(
        source,
        &Position {
            line: 0,
            character: 6,
        },
    );
    assert_eq!(id.as_deref(), Some("my_var"));
}

// ═══════════════════════════════════════════════════════════════════════════
// span_to_range — additional checks
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn span_to_range_saturating_at_zero() {
    // Span with line/col 0 should saturate to 0 (not underflow)
    let span = hudhudscript_ast::Span::new(
        hudhudscript_ast::Position::new(0, 0, 0),
        hudhudscript_ast::Position::new(0, 0, 0),
    );
    let range = span_to_range(span);
    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 0);
    assert_eq!(range.end.line, 0);
    assert_eq!(range.end.character, 0);
}

#[test]
fn span_to_range_multiline() {
    let span = hudhudscript_ast::Span::new(
        hudhudscript_ast::Position::new(3, 5, 20),
        hudhudscript_ast::Position::new(7, 10, 80),
    );
    let range = span_to_range(span);
    assert_eq!(range.start.line, 2);
    assert_eq!(range.start.character, 4);
    assert_eq!(range.end.line, 6);
    assert_eq!(range.end.character, 9);
}
