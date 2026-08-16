// RAG `recall` as a value-returning call.
//
// Before this: the `recall "q" from S;` statement computed the hit list in the
// VM and wrote it to a compiler-allocated scratch register that nothing ever
// read (compiler/stmt_shared/special.rs emitted `Recall { src: r, dst: r }`),
// so a script could never see its own recall results. `recall` was already
// declared as a callable builtin in bytecode/registry/globals.rs but had no
// implementation — calling it produced "Unknown action or function: recall".
//
// Now `recall(query[, store])` dispatches to `VM::rag_recall`, the SAME
// implementation `Instruction::Recall` uses (Kural 7 — one code path, two
// surfaces). These tests pin the returned shape and the arity contract, and
// lock the statement form so the shared path can't regress for either caller.
//
// Scores are deliberately NOT asserted: SimpleEmbedding produces
// near-orthogonal vectors, so relative ordering is not meaningful and
// asserting it would be a flaky test rather than a real invariant.
use hudhudscript_vm::VM;

fn run(src: &str) -> Result<VM, String> {
    let stmts = hudhudscript_parser::parse(src).map_err(|e| format!("parse: {}", e))?;
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler
        .compile(&stmts)
        .map_err(|e| format!("compile: {}", e))?;
    let mut vm = VM::new();
    vm.execute(&bc).map_err(|e| format!("{}", e))?;
    Ok(vm)
}

fn eval(src: &str) -> String {
    run(src).unwrap().last_return_value().display_string()
}

fn run_err(src: &str) -> String {
    match run(src) {
        Ok(_) => panic!("expected an error, but the script ran successfully"),
        Err(e) => e,
    }
}

// ======================================================================
// 1 — The actual bug: a script can read its recall results
// ======================================================================
#[test]
fn recall_call_returns_hit_text() {
    let src = r#"
remember "alpha beta" in S;
let hits = recall("alpha beta", "S");
return hits[0].text;
"#;
    assert_eq!(
        eval(src),
        "alpha beta",
        "recall() must return the stored text; an empty/null result means the \
         hit list is still being dropped"
    );
}

// ======================================================================
// 2 — Hit shape is { id, text, score } — three keys, nothing else
// ======================================================================
#[test]
fn recall_hit_has_id_text_score() {
    let src = r#"
remember "alpha" in S;
let hits = recall("alpha", "S");
return len(keys(hits[0]));
"#;
    assert_eq!(
        eval(src),
        "3",
        "each hit must expose exactly id, text, score"
    );
}

#[test]
fn recall_hit_id_is_non_empty() {
    let src = r#"
remember "alpha" in S;
let hits = recall("alpha", "S");
return len(hits[0].id) > 0;
"#;
    assert_eq!(eval(src), "true");
}

// ======================================================================
// 3 — Empty query returns every stored item (the __rag_store shortcut)
// ======================================================================
#[test]
fn recall_empty_query_returns_all() {
    let src = r#"
remember "one" in S;
remember "two" in S;
return len(recall("", "S"));
"#;
    assert_eq!(eval(src), "2");
}

// ======================================================================
// 4 — A store that was never written to yields an empty list, not an error
// ======================================================================
#[test]
fn recall_unknown_store_returns_empty_list() {
    let src = r#"
remember "alpha" in S;
return len(recall("alpha", "NeverWritten"));
"#;
    assert_eq!(eval(src), "0");
}

// ======================================================================
// 5 — top-K is 5: seven stored items still yield at most five hits
// ======================================================================
#[test]
fn recall_caps_results_at_top_k_five() {
    let src = r#"
remember "item one" in S;
remember "item two" in S;
remember "item three" in S;
remember "item four" in S;
remember "item five" in S;
remember "item six" in S;
remember "item seven" in S;
return len(recall("item", "S"));
"#;
    assert_eq!(eval(src), "5", "TOP_K = 5 must bound the returned hit list");
}

// ======================================================================
// 6 — Stores are isolated by name
// ======================================================================
#[test]
fn recall_stores_do_not_leak_into_each_other() {
    let src = r#"
remember "only in one" in S1;
remember "only in two" in S2;
remember "also in two" in S2;
return len(recall("", "S1"));
"#;
    assert_eq!(eval(src), "1");
}

// ======================================================================
// 7 — The store argument is optional and falls back to "default"
// ======================================================================
#[test]
fn recall_without_store_uses_default_store() {
    let src = r#"
remember "in the default store";
return len(recall(""));
"#;
    assert_eq!(eval(src), "1");
}

#[test]
fn recall_without_store_does_not_see_named_store() {
    let src = r#"
remember "named only" in S;
return len(recall(""));
"#;
    assert_eq!(
        eval(src),
        "0",
        "an omitted store must mean \"default\", not \"any store\""
    );
}

// ======================================================================
// 8 — Arity is enforced; no silent default for a missing query (Kural 7c)
// ======================================================================
#[test]
fn recall_with_no_arguments_is_an_error() {
    let err = run_err("let x = recall();");
    assert!(
        err.contains("recall() expects 1 or 2 arguments"),
        "expected an arity error, got: {}",
        err
    );
}

#[test]
fn recall_with_three_arguments_is_an_error() {
    let err = run_err(r#"let x = recall("a", "b", "c");"#);
    assert!(
        err.contains("recall() expects 1 or 2 arguments"),
        "expected an arity error, got: {}",
        err
    );
}

// ======================================================================
// 9 — Both surfaces run through the one shared implementation:
//     the statement form still executes and leaves the store queryable.
// ======================================================================
#[test]
fn recall_statement_form_still_works_alongside_the_call_form() {
    let src = r#"
remember "shared path" in S;
recall "shared path" from S;
return len(recall("shared path", "S"));
"#;
    assert_eq!(
        eval(src),
        "1",
        "the statement form must keep working after extraction into rag_recall"
    );
}
