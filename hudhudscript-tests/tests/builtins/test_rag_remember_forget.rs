// RAG `remember` / `forget` as value-returning calls.
//
// Both were declared as callable builtins in bytecode/registry/globals.rs but
// had no implementation — calling them produced "Unknown action or function".
// Only the statement forms (`remember x in S;`, `forget x from S;`) worked, and
// neither reported anything back: a script could not learn the id it had just
// stored, nor how many entries a forget actually removed.
//
// Now `remember(content[, store])` returns the new entry's id and
// `forget(target[, store])` returns the number of entries removed. Both
// dispatch to `VM::rag_remember` / `VM::rag_forget` — the SAME implementations
// the instructions use (Kural 7 — one code path, two surfaces).
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
// remember() — returns the entry id
// ======================================================================
#[test]
fn remember_returns_a_non_empty_entry_id() {
    let src = r#"
let id = remember("alpha", "S");
return len(id) > 0;
"#;
    assert_eq!(eval(src), "true");
}

#[test]
fn remembered_id_matches_the_id_recall_reports() {
    let src = r#"
let id = remember("alpha", "S");
let hits = recall("alpha", "S");
return hits[0].id == id;
"#;
    assert_eq!(
        eval(src),
        "true",
        "remember() and recall() must be talking about the same entry in the \
         same store — a mismatch means the two surfaces diverged"
    );
}

#[test]
fn remember_makes_the_value_recallable() {
    let src = r#"
let id = remember("findable", "S");
return len(recall("findable", "S"));
"#;
    assert_eq!(eval(src), "1");
}

#[test]
fn remember_without_store_uses_default_store() {
    let src = r#"
let id = remember("in the default store");
return len(recall("", "default"));
"#;
    assert_eq!(eval(src), "1");
}

// ======================================================================
// forget() — returns how many entries were removed
// ======================================================================
#[test]
fn forget_returns_the_removed_count() {
    let src = r#"
let a = remember("x", "S");
let b = remember("y", "S");
return forget("x", "S");
"#;
    assert_eq!(eval(src), "1");
}

#[test]
fn forget_removes_only_the_matching_entry() {
    let src = r#"
let a = remember("x", "S");
let b = remember("y", "S");
let n = forget("x", "S");
return len(recall("", "S"));
"#;
    assert_eq!(eval(src), "1");
}

#[test]
fn forget_makes_the_value_unrecallable() {
    let src = r#"
let a = remember("gone", "S");
let n = forget("gone", "S");
return len(recall("", "S"));
"#;
    assert_eq!(eval(src), "0");
}

#[test]
fn forget_with_empty_target_clears_the_store_and_counts_it() {
    let src = r#"
let a = remember("p", "S");
let b = remember("q", "S");
return forget("", "S");
"#;
    assert_eq!(eval(src), "2", "an empty target clears the whole store");
}

#[test]
fn forget_with_empty_target_leaves_the_store_empty() {
    let src = r#"
let a = remember("p", "S");
let b = remember("q", "S");
let n = forget("", "S");
return len(recall("", "S"));
"#;
    assert_eq!(eval(src), "0");
}

#[test]
fn forget_of_a_missing_entry_returns_zero_not_an_error() {
    let src = r#"
let a = remember("present", "S");
return forget("never stored", "S");
"#;
    assert_eq!(eval(src), "0");
}

#[test]
fn forget_does_not_touch_other_stores() {
    let src = r#"
let a = remember("shared name", "S1");
let b = remember("shared name", "S2");
let n = forget("shared name", "S1");
return len(recall("", "S2"));
"#;
    assert_eq!(eval(src), "1");
}

// ======================================================================
// Arity is enforced on both; no silent default (Kural 7c)
// ======================================================================
#[test]
fn remember_with_no_arguments_is_an_error() {
    let err = run_err("let x = remember();");
    assert!(
        err.contains("remember() expects 1 or 2 arguments"),
        "expected an arity error, got: {}",
        err
    );
}

#[test]
fn remember_with_three_arguments_is_an_error() {
    let err = run_err(r#"let x = remember("a", "b", "c");"#);
    assert!(
        err.contains("remember() expects 1 or 2 arguments"),
        "expected an arity error, got: {}",
        err
    );
}

#[test]
fn forget_with_no_arguments_is_an_error() {
    let err = run_err("let x = forget();");
    assert!(
        err.contains("forget() expects 1 or 2 arguments"),
        "expected an arity error, got: {}",
        err
    );
}

#[test]
fn forget_with_three_arguments_is_an_error() {
    let err = run_err(r#"let x = forget("a", "b", "c");"#);
    assert!(
        err.contains("forget() expects 1 or 2 arguments"),
        "expected an arity error, got: {}",
        err
    );
}

// ======================================================================
// The statement and call surfaces share one implementation: what a statement
// writes, a call can delete — and vice versa.
// ======================================================================
#[test]
fn call_form_forgets_what_the_statement_form_remembered() {
    let src = r#"
remember "written by the statement" in S;
return forget("written by the statement", "S");
"#;
    assert_eq!(
        eval(src),
        "1",
        "the statement wrote into the same store the builtin deletes from"
    );
}

#[test]
fn statement_form_forgets_what_the_call_form_remembered() {
    let src = r#"
let id = remember("written by the call", "S");
forget "written by the call" from S;
return len(recall("", "S"));
"#;
    assert_eq!(eval(src), "0");
}

#[test]
fn statement_forms_still_work_end_to_end() {
    let src = r#"
remember "one" in S;
remember "two" in S;
forget "one" from S;
return len(recall("", "S"));
"#;
    assert_eq!(
        eval(src),
        "1",
        "extracting the instruction bodies into rag_remember/rag_forget must \
         not change statement behaviour"
    );
}
