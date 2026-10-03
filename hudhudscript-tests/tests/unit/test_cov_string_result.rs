//! Coverage tests for `dispatch_string_result.rs` (VM builtin group 2).
//!
//! Every test compiles a real HudHudScript source, executes it on the VM
//! through the free-function call path (which routes `Call` instructions
//! into `VM::call_builtin` -> `dispatch_builtin_group2`), and pins the
//! exact resulting value or the exact runtime error text.

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
    let v = vm.get_variable_owned(name).unwrap_or_else(|| panic!("{} must be published", name));
    v.as_string().unwrap_or_else(|| panic!("{} must be a string", name))
}

fn global_int(vm: &VM, name: &str) -> i64 {
    let v = vm.get_variable_owned(name).unwrap_or_else(|| panic!("{} must be published", name));
    v.as_int().unwrap_or_else(|| panic!("{} must be an int", name))
}

fn global_bool(vm: &VM, name: &str) -> bool {
    let v = vm.get_variable_owned(name).unwrap_or_else(|| panic!("{} must be published", name));
    v.as_bool().unwrap_or_else(|| panic!("{} must be a bool", name))
}

// ── concat and equality (operand semantics used by every test below) ────────

#[test]
fn concat_operator_builds_exact_string() {
    let vm = run_ok(
        r#"
let word = "ba" + "na" + "na"
let glued = "x" + "" + "y"
"#,
    );
    assert_eq!(global_string(&vm, "word"), "banana");
    assert_eq!(global_string(&vm, "glued"), "xy");
}

#[test]
fn string_equality_compares_contents() {
    let vm = run_ok(
        r#"
let a = "hud"
let b = "hud"
let c = "bat"
let same = 0
if (a == b) { same = 1 }
let diff = 0
if (a == c) { diff = 1 }
"#,
    );
    assert_eq!(global_int(&vm, "same"), 1);
    assert_eq!(global_int(&vm, "diff"), 0);
}

// ── indexOf ─────────────────────────────────────────────────────────────────

#[test]
fn indexof_string_first_index_or_minus_one() {
    let vm = run_ok(
        r#"
let s = "hudhud-script"
let head = indexOf(s, "hud")
let tail = indexOf(s, "script")
let missing = indexOf(s, "zzz")
"#,
    );
    assert_eq!(global_int(&vm, "head"), 0);
    assert_eq!(global_int(&vm, "tail"), 7);
    assert_eq!(global_int(&vm, "missing"), -1);
}

#[test]
fn indexof_reports_byte_offset_for_multibyte_haystack() {
    // 'é' occupies two UTF-8 bytes, so "llo" starts at byte 3 (char index 2).
    // The builtin delegates to str::find, which returns byte offsets.
    let vm = run_ok(r#"let s = "héllo"; let at = indexOf(s, "llo");"#);
    assert_eq!(global_int(&vm, "at"), 3);
}

#[test]
fn indexof_array_position_or_minus_one() {
    let vm = run_ok(
        r#"
let xs = [10, 20, 30]
let hit = indexOf(xs, 20)
let miss = indexOf(xs, 99)
"#,
    );
    assert_eq!(global_int(&vm, "hit"), 1);
    assert_eq!(global_int(&vm, "miss"), -1);
}

#[test]
fn indexof_rejects_number_haystack() {
    let msg = run_error(r#"let n = 3; let x = indexOf(n, "a");"#);
    assert!(msg.contains("indexOf() requires string or array"), "{}", msg);
}

// ── contains ────────────────────────────────────────────────────────────────

#[test]
fn contains_string_matches_substring() {
    let vm = run_ok(
        r#"
let s = "hudhud-script"
let yes = contains(s, "script")
let no = contains(s, "rust")
"#,
    );
    assert_eq!(global_bool(&vm, "yes"), true);
    assert_eq!(global_bool(&vm, "no"), false);
}

#[test]
fn contains_array_tests_membership() {
    let vm = run_ok(
        r#"
let xs = [1, 2, 3]
let yes = contains(xs, 2)
let no = contains(xs, 9)
"#,
    );
    assert_eq!(global_bool(&vm, "yes"), true);
    assert_eq!(global_bool(&vm, "no"), false);
}

#[test]
fn contains_on_number_is_false_not_an_error() {
    // Neither string nor array haystack falls through to `false` by design.
    let vm = run_ok(r#"let n = 42; let c = contains(n, 42);"#);
    assert_eq!(global_bool(&vm, "c"), false);
}

// ── split and join ──────────────────────────────────────────────────────────

#[test]
fn split_multichar_delimiter_yields_exact_parts() {
    let vm = run_ok(
        r#"
let s = "a::b::c"
let parts = split(s, "::")
let n = length(parts)
let first = parts[0]
let last = parts[2]
"#,
    );
    assert_eq!(global_int(&vm, "n"), 3);
    assert_eq!(global_string(&vm, "first"), "a");
    assert_eq!(global_string(&vm, "last"), "c");
}

#[test]
fn split_without_match_yields_whole_string() {
    let vm = run_ok(
        r#"
let s = "abc"
let parts = split(s, "-")
let n = length(parts)
let only = parts[0]
"#,
    );
    assert_eq!(global_int(&vm, "n"), 1);
    assert_eq!(global_string(&vm, "only"), "abc");
}

#[test]
fn split_requires_string_arguments() {
    let msg = run_error(r#"let n = 7; let x = split(n, ",");"#);
    assert!(msg.contains("split() requires string arguments"), "{}", msg);
}

#[test]
fn join_stringifies_elements_between_delimiter() {
    let vm = run_ok(
        r#"
let ints = [1, 2, 3]
let nums = join(ints, "-")
let words = join(["a", "b"], "")
let mixed = join([1, 2.5], ",")
"#,
    );
    assert_eq!(global_string(&vm, "nums"), "1-2-3");
    assert_eq!(global_string(&vm, "words"), "ab");
    assert_eq!(global_string(&vm, "mixed"), "1,2.5");
}

#[test]
fn join_requires_array_and_string() {
    let msg = run_error(r#"let s = "abc"; let x = join(s, "-");"#);
    assert!(msg.contains("join() requires array and string"), "{}", msg);
}

// ── replace and trim ────────────────────────────────────────────────────────

#[test]
fn replace_substitutes_every_occurrence() {
    let vm = run_ok(r#"let s = replace("aXbXc", "X", "-");"#);
    assert_eq!(global_string(&vm, "s"), "a-b-c");
}

#[test]
fn replace_requires_three_arguments() {
    let msg = run_error(r#"let s = replace("a", "b");"#);
    assert!(msg.contains("replace() expects 3 arguments, got 2"), "{}", msg);
}

#[test]
fn trim_strips_surrounding_spaces_and_keeps_empty() {
    let vm = run_ok(r#"let pad = trim("  hud  "); let empty = trim("");"#);
    assert_eq!(global_string(&vm, "pad"), "hud");
    assert_eq!(global_string(&vm, "empty"), "");
}

#[test]
fn trim_requires_exactly_one_argument() {
    let msg = run_error(r#"let s = trim("a", "b");"#);
    assert!(msg.contains("trim() expects 1 argument, got 2"), "{}", msg);
}

// ── toUpperCase and toLowerCase ─────────────────────────────────────────────

#[test]
fn case_conversion_roundtrips_ascii() {
    let vm = run_ok(
        r#"
let up = toUpperCase("hud-9")
let down = toLowerCase("MiXeD")
"#,
    );
    assert_eq!(global_string(&vm, "up"), "HUD-9");
    assert_eq!(global_string(&vm, "down"), "mixed");
}

#[test]
fn to_uppercase_applies_full_unicode_mapping() {
    // German sharp s expands to "SS" under the locale-neutral mapping.
    let vm = run_ok(r#"let up = toUpperCase("straße");"#);
    assert_eq!(global_string(&vm, "up"), "STRASSE");
}

#[test]
fn to_uppercase_requires_a_string() {
    let msg = run_error(r#"let n = 42; let x = toUpperCase(n);"#);
    assert!(msg.contains("toUpperCase() requires a string"), "{}", msg);
}

// ── startsWith and endsWith ─────────────────────────────────────────────────
// Ground truth: startsWith/endsWith are string *methods* (string.rs); the
// group2 free-function arms are dead (names missing from builtin_name_set),
// so scripts must use the method form below (suspected source bug — report).

#[test]
fn starts_and_ends_with_match_exact_edges() {
    let vm = run_ok(
        r#"
let s = "hudhud-script"
let sw_yes = s.startsWith("hudhud")
let sw_no = s.startsWith("script")
let ew_yes = s.endsWith("script")
let ew_no = s.endsWith("hudhud")
"#,
    );
    assert_eq!(global_bool(&vm, "sw_yes"), true);
    assert_eq!(global_bool(&vm, "sw_no"), false);
    assert_eq!(global_bool(&vm, "ew_yes"), true);
    assert_eq!(global_bool(&vm, "ew_no"), false);
}

#[test]
fn starts_with_on_a_number_receiver_names_the_type() {
    let msg = run_error(r#"let n = 5; let x = n.startsWith("a");"#);
    assert!(msg.contains("Cannot call method 'startsWith' on number"), "{}", msg);
}

// ── length ──────────────────────────────────────────────────────────────────

#[test]
fn length_counts_ascii_chars_and_empty_as_zero() {
    let vm = run_ok(r#"let n = length("hudhud"); let z = length("");"#);
    assert_eq!(global_int(&vm, "n"), 6);
    assert_eq!(global_int(&vm, "z"), 0);
}

#[test]
fn length_counts_bytes_for_non_ascii_strings() {
    let vm = run_ok(r#"let tr = length("İstanbul"); let emoji = length("🚀");"#);
    // 'İ' is 2 UTF-8 bytes + 7 ASCII; the rocket is a 4-byte code point.
    assert_eq!(global_int(&vm, "tr"), 9);
    assert_eq!(global_int(&vm, "emoji"), 4);
}

#[test]
fn length_counts_array_elements_and_object_fields() {
    let vm = run_ok(
        r#"
let xs = [1, 2, 3]
let arr = length(xs)
let o = { a: 1, b: 2, c: 3 }
let obj = length(o)
"#,
    );
    assert_eq!(global_int(&vm, "arr"), 3);
    assert_eq!(global_int(&vm, "obj"), 3);
}

#[test]
fn length_rejects_numbers() {
    let msg = run_error(r#"let n = 42; let x = length(n);"#);
    assert!(msg.contains("length() not supported for type number"), "{}", msg);
}

// ── Some / Ok / Err constructors and unwrap ─────────────────────────────────

#[test]
fn some_and_ok_roundtrip_through_unwrap() {
    let vm = run_ok(
        r#"
let o = Some(41)
let n = unwrap(o)
let w = Some("hud")
let s = unwrap(w)
let r = Ok(9)
let k = unwrap(r)
"#,
    );
    assert_eq!(global_int(&vm, "n"), 41);
    assert_eq!(global_string(&vm, "s"), "hud");
    assert_eq!(global_int(&vm, "k"), 9);
}

#[test]
fn unwrap_err_reports_stringified_payload() {
    // Err() stringifies its payload, so a numeric 42 surfaces as "Err(42)".
    let msg = run_error(r#"let e = Err(42); let x = unwrap(e);"#);
    assert!(msg.contains("called unwrap() on Err(42)"), "{}", msg);
}

#[test]
fn unwrap_on_null_reports_none() {
    let msg = run_error(r#"let n = null; let x = unwrap(n);"#);
    assert!(msg.contains("called unwrap() on None"), "{}", msg);
}

#[test]
fn unwrap_on_plain_number_is_rejected() {
    let msg = run_error(r#"let n = 5; let x = unwrap(n);"#);
    assert!(msg.contains("unwrap() requires Option or Result"), "{}", msg);
}

#[test]
fn unwrap_or_keeps_present_value_and_supplies_default() {
    let vm = run_ok(
        r#"
let o = Some(3)
let a = unwrap_or(o, 7)
let n = null
let b = unwrap_or(n, 7)
let e = Err("nope")
let c = unwrap_or(e, 7)
let r = Ok(9)
let d = unwrap_or(r, 7)
"#,
    );
    assert_eq!(global_int(&vm, "a"), 3);
    assert_eq!(global_int(&vm, "b"), 7);
    assert_eq!(global_int(&vm, "c"), 7);
    assert_eq!(global_int(&vm, "d"), 9);
}

#[test]
fn unwrap_or_requires_option_or_result() {
    let msg = run_error(r#"let n = 5; let x = unwrap_or(n, 1);"#);
    assert!(msg.contains("unwrap_or() requires Option or Result"), "{}", msg);
}
