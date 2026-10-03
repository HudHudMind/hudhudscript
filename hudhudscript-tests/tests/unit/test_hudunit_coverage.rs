//! Tests for hudhudscript-hudunit coverage — mark absorption, AST
//! instrumentation (file + module), formatter round-trip, summarization.

use std::path::Path;

use hudhudscript_hudunit::coverage::{
    instrument, instrument_module, summarize, Marks, MarkMap,
};

// ── marks ─────────────────────────────────────────────────────────

#[test]
fn absorb_seen_parses_ids() {
    let marks = Marks::new();
    marks.absorb_seen("3,7,12,");
    assert_eq!(marks.get(3), 1);
    assert_eq!(marks.get(7), 1);
    assert_eq!(marks.get(12), 1);
    assert_eq!(marks.get(99), 0);
}

#[test]
fn absorb_seen_tolerates_garbage() {
    let marks = Marks::new();
    marks.absorb_seen("x,,3,y");
    assert_eq!(marks.get(3), 1);
}

// ── instrumentation ───────────────────────────────────────────────

#[test]
fn instrument_marks_each_statement() {
    let mut ast = hudhudscript_parser::parse(
        "fn helper(x) {\n    return x + 1;\n}\nlet a = 1;\n",
    )
    .unwrap();
    let mut map = MarkMap::new();
    instrument(&mut ast, Path::new("/t/test_a.hud"), &mut map);
    // helper body (entry + return) + top-level let = at least 3 marks
    assert!(map.all().len() >= 3);
    // Every mark call references the prelude helper
    let rendered = format!("{:?}", ast);
    assert!(rendered.contains("__hudunit_mark"));
}

#[test]
fn prelude_helpers_not_instrumented() {
    let mut ast = hudhudscript_parser::parse(
        "fn __hudunit_mark(id) {\n    return 1;\n}\n",
    )
    .unwrap();
    let mut map = MarkMap::new();
    instrument(&mut ast, Path::new("/t/test_a.hud"), &mut map);
    assert!(map.all().is_empty(), "prelude helpers must be skipped");
}

#[test]
fn module_instrumentation_skips_top_level() {
    let mut ast = hudhudscript_parser::parse(
        "let a = 1;\nfn helper(x) {\n    return x + 1;\n}\n",
    )
    .unwrap();
    let mut map = MarkMap::new();
    instrument_module(&mut ast, Path::new("/t/src/mod.hud"), &mut map);
    // Only the function body (entry + return) got marks; `let a` did not.
    let top_level_marked = map
        .all()
        .values()
        .any(|info| info.start_line == 1);
    assert!(!top_level_marked, "module top level must stay unmarked");
    assert!(map.all().len() >= 2);
}

#[test]
fn module_instrumentation_round_trips_through_formatter() {
    let source = "fn helper(x) {\n    if (x > 0) {\n        return x;\n    }\n    return -x;\n}\n";
    let mut ast = hudhudscript_parser::parse(source).unwrap();
    let mut map = MarkMap::new();
    instrument_module(&mut ast, Path::new("/t/src/mod.hud"), &mut map);
    let formatted = hudhudscript_formatter::Formatter::new().format_program(&ast);
    let reparsed = hudhudscript_parser::parse(&formatted)
        .expect("instrumented module must re-parse for the VM loader");
    assert!(!reparsed.is_empty());
}

// ── summarize ─────────────────────────────────────────────────────

#[test]
fn summarize_reports_lines() {
    let mut ast =
        hudhudscript_parser::parse("fn helper() {\n    return 1;\n}\n").unwrap();
    let mut map = MarkMap::new();
    instrument(&mut ast, Path::new("/t/test_a.hud"), &mut map);
    let marks = Marks::new();
    for id in map.all().keys() {
        marks.hit(*id);
    }
    let files = summarize(&map, &marks);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "/t/test_a.hud");
    assert!(files[0].line_percent() > 99.0);
}
