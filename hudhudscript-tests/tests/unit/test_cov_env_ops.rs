//! Coverage tests for `dispatch_env.rs` (VM builtin group 3).
//!
//! Every test compiles a real HudHudScript source, executes it on the VM
//! through the free-function call path (which routes `Call` instructions
//! into `VM::call_builtin` -> `dispatch_builtin_group3`), and pins the
//! exact resulting value or the exact runtime error text. Env reads use
//! host-set process variables with unique names that are removed again,
//! so no test depends on machine state; the missing-key probe relies on
//! no `.env` file existing on the search path with that unique key.

use hudhudscript_bytecode::{PromiseState16, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::vm::VM;

// ── helpers ─────────────────────────────────────────────────────────────────

fn run_vm(source: &str) -> Result<VM, hudhudscript_errors::Error> {
    let ast = parse(source).expect("test source must parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("test source must compile");
    let mut vm = VM::new();
    vm.execute(&bytecode)?;
    Ok(vm)
}

fn run_ok(source: &str) -> VM {
    run_vm(source).unwrap_or_else(|e| panic!("script must succeed: {}", e.message))
}

fn run_error(source: &str) -> String {
    match run_vm(source) {
        Ok(_) => panic!("script must fail, but succeeded"),
        Err(e) => e.message,
    }
}

fn global_string(vm: &VM, name: &str) -> String {
    vm.get_variable_owned(name)
        .unwrap_or_else(|| panic!("{} must be published", name))
        .as_string()
        .unwrap_or_else(|| panic!("{} must be a string", name))
}

fn global_int(vm: &VM, name: &str) -> i64 {
    vm.get_variable_owned(name)
        .unwrap_or_else(|| panic!("{} must be published", name))
        .as_int()
        .unwrap_or_else(|| panic!("{} must be an int", name))
}

fn global_bool(vm: &VM, name: &str) -> bool {
    vm.get_variable_owned(name)
        .unwrap_or_else(|| panic!("{} must be published", name))
        .as_bool()
        .unwrap_or_else(|| panic!("{} must be a bool", name))
}

fn resolved_inner(vm: &VM, name: &str) -> Value16 {
    let v = vm
        .get_variable_owned(name)
        .unwrap_or_else(|| panic!("{} must be published", name));
    match v.as_promise_state() {
        Some(PromiseState16::Resolved(inner)) => (**inner).clone(),
        other => panic!("{} must be resolved, got {:?}", name, other),
    }
}

fn rejected_reason(vm: &VM, name: &str) -> String {
    let v = vm
        .get_variable_owned(name)
        .unwrap_or_else(|| panic!("{} must be published", name));
    match v.as_promise_state() {
        Some(PromiseState16::Rejected(reason)) => reason.clone(),
        other => panic!("{} must be rejected, got {:?}", name, other),
    }
}

// ── env() ───────────────────────────────────────────────────────────────────

#[test]
fn env_returns_process_value_set_from_host() {
    std::env::set_var("HUDHUD_COV_ENV_STR_7311", "alpha-bravo-42");
    let vm = run_ok(r#"let key = "HUDHUD_COV_ENV_STR_7311"; let v = env(key);"#);
    assert_eq!(global_string(&vm, "v"), "alpha-bravo-42");
    std::env::remove_var("HUDHUD_COV_ENV_STR_7311");
}

#[test]
fn env_missing_unique_key_yields_null() {
    let vm = run_ok(r#"let v = env("HUDHUD_COV_MISSING_8442"); let absent = is_none(v);"#);
    let raw = vm.get_variable_owned("v").expect("v must be published");
    assert!(raw.is_null(), "unset unique key must read as null");
    assert_eq!(global_bool(&vm, "absent"), true);
}

#[test]
fn env_empty_value_is_empty_string_not_null() {
    std::env::set_var("HUDHUD_COV_ENV_EMPTY_7311", "");
    let vm = run_ok(r#"let key = "HUDHUD_COV_ENV_EMPTY_7311"; let v = env(key);"#);
    let raw = vm.get_variable_owned("v").expect("v must be published");
    assert!(!raw.is_null(), "an empty value is still a string");
    assert_eq!(global_string(&vm, "v"), "");
    std::env::remove_var("HUDHUD_COV_ENV_EMPTY_7311");
}

#[test]
fn env_rejects_non_string_key() {
    let msg = run_error(r#"let k = 42; let v = env(k);"#);
    assert!(msg.contains("env() requires a string key"), "{}", msg);
}

#[test]
fn env_rejects_extra_argument() {
    let msg = run_error(r#"let v = env("HUDHUD_COV_ARITY_7311", "extra");"#);
    assert!(msg.contains("env() expects 1 argument, got 2"), "{}", msg);
}

// ── is_some / is_none ───────────────────────────────────────────────────────

#[test]
fn is_some_and_is_none_classify_option_and_null() {
    let vm = run_ok(
        r#"
let o = Some(5)
let n = null
let a = is_some(o)
let b = is_none(o)
let c = is_some(n)
let d = is_none(n)
"#,
    );
    assert_eq!(global_bool(&vm, "a"), true);
    assert_eq!(global_bool(&vm, "b"), false);
    // A bare null is treated as None by both predicates.
    assert_eq!(global_bool(&vm, "c"), false);
    assert_eq!(global_bool(&vm, "d"), true);
}

#[test]
fn is_some_rejects_plain_numbers() {
    let msg = run_error(r#"let n = 5; let x = is_some(n);"#);
    assert!(msg.contains("is_some() requires Option"), "{}", msg);
}

#[test]
fn is_none_rejects_strings() {
    let msg = run_error(r#"let s = "txt"; let x = is_none(s);"#);
    assert!(msg.contains("is_none() requires Option"), "{}", msg);
}

// ── is_ok / is_err ──────────────────────────────────────────────────────────

#[test]
fn is_ok_and_is_err_classify_results() {
    let vm = run_ok(
        r#"
let good = Ok(10)
let bad = Err("failure")
let a = is_ok(good)
let b = is_err(good)
let c = is_ok(bad)
let d = is_err(bad)
"#,
    );
    assert_eq!(global_bool(&vm, "a"), true);
    assert_eq!(global_bool(&vm, "b"), false);
    assert_eq!(global_bool(&vm, "c"), false);
    assert_eq!(global_bool(&vm, "d"), true);
}

#[test]
fn is_ok_rejects_options() {
    let msg = run_error(r#"let o = Some(1); let x = is_ok(o);"#);
    assert!(msg.contains("is_ok() requires Result"), "{}", msg);
}

#[test]
fn is_err_rejects_numbers() {
    let msg = run_error(r#"let n = 3; let x = is_err(n);"#);
    assert!(msg.contains("is_err() requires Result"), "{}", msg);
}

// ── exception / istisna ─────────────────────────────────────────────────────

#[test]
fn exception_full_form_builds_canonical_shape() {
    let vm = run_ok(
        r#"
let e = exception("E_COV", "Coverage Title", "coverage description", 42)
let code = e.code
let title = e.title
let description = e.description
let value = e.value
let fields = length(e)
"#,
    );
    assert_eq!(global_string(&vm, "code"), "E_COV");
    assert_eq!(global_string(&vm, "title"), "Coverage Title");
    assert_eq!(global_string(&vm, "description"), "coverage description");
    assert_eq!(global_int(&vm, "value"), 42);
    // code, title, description, value, cause, stack, __hudhud_exception
    assert_eq!(global_int(&vm, "fields"), 7);
}

#[test]
fn exception_two_arg_form_defaults_description_and_value() {
    let vm = run_ok(
        r#"
let e = exception("E_MIN", "Minimal")
let d = e.description
let v = e.value
"#,
    );
    assert_eq!(global_string(&vm, "d"), "");
    let value = vm.get_variable_owned("v").expect("v must be published");
    assert!(value.is_null(), "2-arg exception value must default to null");
}

#[test]
fn exception_non_string_description_falls_back_to_empty() {
    // The optional description slot uses `unwrap_or_default` on the string.
    let vm = run_ok(r#"let e = exception("E_D", "T", 42); let d = e.description;"#);
    assert_eq!(global_string(&vm, "d"), "");
}

#[test]
fn exception_requires_code_and_title() {
    let msg = run_error(r#"let e = exception("E_ONLY_CODE");"#);
    assert!(
        msg.contains("exception() requires at least 2 arguments: code, title"),
        "{}",
        msg
    );
}

#[test]
fn exception_code_must_be_string() {
    let msg = run_error(r#"let e = exception(42, "title");"#);
    assert!(msg.contains("exception() code must be string"), "{}", msg);
}

#[test]
fn exception_title_must_be_string() {
    let msg = run_error(r#"let e = exception("E_T", 42);"#);
    assert!(msg.contains("exception() title must be string"), "{}", msg);
}

#[test]
fn istisna_alias_builds_the_same_shape() {
    let vm = run_ok(
        r#"
let e = istisna("E_TR", "Başlık", "Açıklama")
let code = e.code
let fields = length(e)
"#,
    );
    assert_eq!(global_string(&vm, "code"), "E_TR");
    assert_eq!(global_int(&vm, "fields"), 7);
}

// ── Promise constructors and combinators ────────────────────────────────────

#[test]
fn promise_resolve_wraps_payload() {
    let vm = run_ok(r#"let p = Promise.resolve(41);"#);
    assert_eq!(resolved_inner(&vm, "p").as_int(), Some(41));
}

#[test]
fn promise_reject_stringifies_payload() {
    let vm = run_ok(
        r#"
let a = Promise.reject("boom")
let b = Promise.reject(42)
"#,
    );
    assert_eq!(rejected_reason(&vm, "a"), "boom");
    assert_eq!(rejected_reason(&vm, "b"), "42");
}

#[test]
fn promise_all_passes_plain_values_through_in_order() {
    let vm = run_ok(r#"let xs = [1, "x", true]; let p = Promise.all(xs);"#);
    let inner = resolved_inner(&vm, "p");
    let arr = inner.as_array().expect("all() must resolve to an array");
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0].as_int(), Some(1));
    assert_eq!(arr[1].as_string(), Some("x".to_string()));
    assert_eq!(arr[2].as_bool(), Some(true));
}

#[test]
fn promise_all_short_circuits_on_rejection() {
    let vm = run_ok(
        r#"
let bad = Promise.reject("kaput")
let p = Promise.all([bad, 1])
"#,
    );
    assert_eq!(rejected_reason(&vm, "p"), "kaput");
}

#[test]
fn promise_all_requires_an_array() {
    let msg = run_error(r#"let p = Promise.all(5);"#);
    assert!(msg.contains("Promise.all() requires an array"), "{}", msg);
}

#[test]
fn promise_race_returns_first_settled_plain_value() {
    let vm = run_ok(r#"let xs = [7, 8]; let p = Promise.race(xs);"#);
    assert_eq!(resolved_inner(&vm, "p").as_int(), Some(7));
}

#[test]
fn promise_race_on_empty_array_rejects_instead_of_hanging() {
    let vm = run_ok(r#"let xs = []; let p = Promise.race(xs);"#);
    assert_eq!(rejected_reason(&vm, "p"), "Promise.race() on empty array");
}

#[test]
fn promise_all_settled_is_unreachable_from_scripts() {
    // Ground truth (source-verified): `dispatch_builtin_group3` implements a
    // full "Promise.allSettled" arm and `builtin_name_set` registers the
    // dotted name, but a script-level `Promise.x()` call is a method call on
    // the Promise module object and routes to `call_promise_method`
    // (builtin_value.rs), which has no "allSettled" arm. The group3
    // implementation is therefore dead code for script calls; the observable
    // behavior is the unknown-method error pinned below. Suspected source
    // bug — see the task report.
    let msg = run_error(
        r#"
let good = Promise.resolve(1)
let bad = Promise.reject("nope")
let xs = [good, bad, 5]
let p = Promise.allSettled(xs)
"#,
    );
    assert!(msg.contains("Unknown Promise method: allSettled"), "{}", msg);
}

#[test]
fn promise_resolve_defaults_missing_argument_to_null_and_ignores_extras() {
    // Ground truth (source-verified): `call_promise_method` resolves with
    // `args.into_iter().next().unwrap_or(Value16::null())` — a missing
    // argument resolves null and extra arguments are ignored in favor of
    // the first. (The arity-checking "Promise.resolve" arm in group3 is
    // unreachable from script syntax, same routing as allSettled above.)
    let vm = run_ok(r#"let a = Promise.resolve();"#);
    assert!(
        resolved_inner(&vm, "a").is_null(),
        "Promise.resolve() must default to Resolved(null)"
    );
    let vm = run_ok(r#"let b = Promise.resolve(1, 2);"#);
    assert_eq!(
        resolved_inner(&vm, "b").as_int(),
        Some(1),
        "extra arguments must be ignored in favor of the first"
    );
}
