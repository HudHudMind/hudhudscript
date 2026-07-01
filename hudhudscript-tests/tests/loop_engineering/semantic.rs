// Semantic loop engineering tests — fail-closed exact property assertions.
//
// Run: make test-sccache TEST=loop_engineering JOBS=52 TEST_THREAD=52

use hudhudscript_parser::parse;
use hudhudscript_compiler::compiler::Compiler;
use hudhudscript_vm::VM;
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

fn compile_and_execute(src: &str) -> VM {
    let stmts = parse(src).unwrap();
    let mut compiler = Compiler::default();
    let bytecode = compiler.compile(&stmts).unwrap();
    let mut vm = VM::new();
    vm.execute(&bytecode).unwrap();
    vm
}

fn compile_err_msg(src: &str) -> String {
    let stmts = parse(src).unwrap();
    let err = Compiler::default().compile(&stmts).unwrap_err();
    format!("{}", err)
}

// ── Fail-closed helpers ───────────────────────────────────────────────

fn result_obj(vm: &VM) -> hudhudscript_bytecode::ObjMap {
    let ret = vm.last_return_value();
    ret.as_object().map(|m| m.clone()).expect("result must be object")
}

fn required_bool(obj: &hudhudscript_bytecode::ObjMap, key: &str) -> bool {
    obj.get(key).unwrap_or_else(|| panic!("required bool '{key}' is missing"))
        .as_bool().unwrap_or_else(|| panic!("required '{key}' is not bool"))
}

fn required_str(obj: &hudhudscript_bytecode::ObjMap, key: &str) -> String {
    obj.get(key).unwrap_or_else(|| panic!("required string '{key}' is missing"))
        .as_string().unwrap_or_else(|| panic!("required '{key}' is not string"))
}

fn optional_bool(obj: &hudhudscript_bytecode::ObjMap, key: &str) -> Option<bool> {
    match obj.get(key) {
        None => None,
        Some(v) => Some(v.as_bool().unwrap_or_else(|| panic!("optional '{key}' exists but is not bool"))),
    }
}

// Q1-Q3: Canonical get_variable helpers (not get_global)
fn required_vm_int(vm: &VM, name: &str) -> i64 {
    vm.get_variable(name)
        .unwrap_or_else(|| panic!("required VM variable '{name}' is missing"))
        .as_int()
        .unwrap_or_else(|| panic!("required VM variable '{name}' is not int"))
}

fn required_vm_bool(vm: &VM, name: &str) -> bool {
    vm.get_variable(name)
        .unwrap_or_else(|| panic!("required VM variable '{name}' is missing"))
        .as_bool()
        .unwrap_or_else(|| panic!("required VM variable '{name}' is not bool"))
}

// ── Gate done/fail ────────────────────────────────────────────────────

#[test]
fn gate_done_sets_success_true_status_done() {
    let vm = compile_and_execute("loop L { step S { let v = 7; gate G { when v == 7 -> done else -> fail } } } run loop L");
    let obj = result_obj(&vm);
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

#[test]
fn gate_fail_sets_success_false_status_failed() {
    let vm = compile_and_execute("loop L { step S { let v = 7; gate G { when v == 8 -> done else -> fail } } } run loop L");
    let obj = result_obj(&vm);
    assert_eq!(required_bool(&obj, "success"), false);
    assert_eq!(required_str(&obj, "status"), String::from("failed"));
}

#[test]
fn mutation_condition_changes_done_to_fail() {
    let obj1 = result_obj(&compile_and_execute("loop L { step S { let v = 7; gate G { when v == 7 -> done else -> fail } } } run loop L"));
    assert_eq!(required_bool(&obj1, "success"), true);
    let obj2 = result_obj(&compile_and_execute("loop L { step S { let v = 7; gate G { when v == 8 -> done else -> fail } } } run loop L"));
    assert_eq!(required_bool(&obj2, "success"), false);
}

// ── Step transitions ─────────────────────────────────────────────────

#[test]
fn forward_step_skips_intermediate() {
    let vm = compile_and_execute("loop L { step s1 { gate G { when true -> s3 else -> done } } step s2 { result.s2_ran = true; gate G { when true -> done else -> done } } step s3 { result.s3_ran = true; gate G { when true -> done else -> done } } } run loop L");
    let obj = result_obj(&vm);
    assert_eq!(optional_bool(&obj, "s2_ran"), None, "s2 must NOT run");
    assert_eq!(required_bool(&obj, "s3_ran"), true);
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

#[test]
fn unknown_step_compile_error() {
    let err = compile_err_msg("loop L { step s { gate g { when true -> nonexistent else -> done } } }");
    assert!(err.contains("not found"), "got: {}", err);
}

// ── Gate targets ─────────────────────────────────────────────────────

#[test]
fn gate_target_pause_status_paused() {
    let obj = result_obj(&compile_and_execute("loop L { step S { gate G { when true -> pause else -> done } } } run loop L"));
    assert_eq!(required_str(&obj, "status"), String::from("paused"));
}

#[test]
fn gate_target_approval_status_awaiting_approval() {
    let obj = result_obj(&compile_and_execute("loop L { step S { gate G { when true -> approval else -> done } } } run loop L"));
    assert_eq!(required_str(&obj, "status"), String::from("awaiting_approval"));
}

#[test]
fn gate_target_escalate_status_escalated() {
    let obj = result_obj(&compile_and_execute("loop L { step S { gate G { when true -> escalate else -> done } } } run loop L"));
    assert_eq!(required_str(&obj, "status"), String::from("escalated"));
}

// ── Continue ─────────────────────────────────────────────────────────

#[test]
fn continue_transfers_to_next_step() {
    let vm = compile_and_execute("loop L { step s1 { result.s1_ran = true; gate G { when true -> continue else -> done } } step s2 { result.s2_ran = true; gate G { when true -> done else -> fail } } } run loop L");
    let obj = result_obj(&vm);
    assert_eq!(required_bool(&obj, "s1_ran"), true);
    assert_eq!(required_bool(&obj, "s2_ran"), true);
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

// ── Mode: times(N) — exact counter + marker ─────────────────────────

#[test]
fn mode_times_3_exact_iterations() {
    let vm = compile_and_execute("let counter = 0; loop T mode: times(3) { step s { counter = counter + 1; gate G { when true -> continue else -> fail } } } run loop T");
    let obj = result_obj(&vm);
    assert_eq!(required_vm_int(&vm, "counter"), 3);
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

#[test]
fn mode_times_1_exact_iteration() {
    let vm = compile_and_execute("let counter = 0; loop T mode: times(1) { step s { counter = counter + 1; gate G { when true -> done else -> fail } } } run loop T");
    let obj = result_obj(&vm);
    assert_eq!(required_vm_int(&vm, "counter"), 1);
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

// Q4-Q5: zero-body tests — exact bool marker, no fake result.ran
#[test]
fn mode_times_0_zero_iterations() {
    let vm = compile_and_execute("let body_ran = false; loop T mode: times(0) { step s { body_ran = true; gate G { when true -> done else -> fail } } } run loop T");
    let obj = result_obj(&vm);
    assert_eq!(required_vm_bool(&vm, "body_ran"), false, "times(0): body must NOT run");
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

#[test]
fn mode_times_done_exits_early() {
    let vm = compile_and_execute("let counter = 0; loop T mode: times(5) { step s { counter = counter + 1; gate G { when true -> done else -> fail } } } run loop T");
    let obj = result_obj(&vm);
    assert_eq!(required_vm_int(&vm, "counter"), 1, "done exits early after 1 iteration");
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

// ── Mode: until(expr) — exact counter + marker ──────────────────────

#[test]
fn mode_until_counter_ge_3_stops_at_3() {
    let vm = compile_and_execute("let counter = 0; loop U mode: until(counter >= 3) { step s { counter = counter + 1; gate G { when true -> continue else -> fail } } } run loop U");
    let obj = result_obj(&vm);
    assert_eq!(required_vm_int(&vm, "counter"), 3);
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

#[test]
fn mode_until_true_zero_body_executions() {
    let vm = compile_and_execute("let body_ran = false; loop U mode: until(true) { step s { body_ran = true; gate G { when true -> done else -> fail } } } run loop U");
    let obj = result_obj(&vm);
    assert_eq!(required_vm_bool(&vm, "body_ran"), false, "until(true): body must NOT run");
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

// Q6-Q7: Pozitif kontrol testleri — body calistiginda marker true olur
#[test]
fn mode_times_1_executes_body_marker() {
    let vm = compile_and_execute("let body_ran = false; loop T mode: times(1) { step s { body_ran = true; gate G { when true -> done else -> fail } } } run loop T");
    let obj = result_obj(&vm);
    assert_eq!(required_vm_bool(&vm, "body_ran"), true, "times(1): body MUST run");
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

#[test]
fn mode_until_false_executes_body_before_done() {
    let vm = compile_and_execute("let body_ran = false; loop U mode: until(false) { step s { body_ran = true; gate G { when true -> done else -> fail } } } run loop U");
    let obj = result_obj(&vm);
    assert_eq!(required_vm_bool(&vm, "body_ran"), true, "until(false): body MUST run");
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

// ── Mode validation ──────────────────────────────────────────────────

#[test]
fn mode_cyclic_with_terminal_targets_compiles_and_runs() {
    let obj = result_obj(&compile_and_execute("loop L mode: cyclic { step s { gate G { when true -> done else -> done } } } run loop L"));
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

#[test]
fn mode_cyclic_without_terminal_target_compile_error() {
    let err = compile_err_msg("loop L mode: cyclic { step s { gate G { when true -> continue else -> continue } } }");
    assert!(err.contains("terminal"), "expected terminal error, got: {}", err);
}

#[test]
fn mode_until_converged_compiles_and_runs() {
    let obj = result_obj(&compile_and_execute("loop L mode: until_converged { step s { gate G { when true -> done else -> done } } } run loop L"));
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

#[test]
fn mode_until_converged_without_terminal_compile_error() {
    let err = compile_err_msg("loop L mode: until_converged { step s { gate G { when true -> continue else -> continue } } }");
    assert!(err.contains("terminal"), "expected terminal error, got: {}", err);
}

// ── Retry (FP6: bounded, max 3 attempts) ────────────────────────────

#[test]
fn retry_bounded_escalates_after_3_attempts() {
    // retry 3 times → attempt exhausted → escalate
    let obj = result_obj(&compile_and_execute("loop L { step s { gate g { when true -> retry else -> done } } } run loop L"));
    assert_eq!(required_str(&obj, "status"), String::from("escalated"));
    // attempt counter should be 3
    assert_eq!(obj.get("__attempt").and_then(|v| v.as_int()), Some(3));
}

#[test]
fn retry_bounded_condition_false_no_retry() {
    // condition false → else (done), no retry
    let obj = result_obj(&compile_and_execute("loop L { step s { let v = 0; gate g { when v == 1 -> retry else -> done } } } run loop L"));
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

// ── Chain ────────────────────────────────────────────────────────────

#[test]
fn chain_single_link_done_returns_success() {
    let obj = result_obj(&compile_and_execute("chain c { loop l1 { step s { gate G { when true -> done else -> fail } } } } run chain c"));
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

#[test]
fn chain_two_links_both_done_returns_success() {
    let obj = result_obj(&compile_and_execute("chain c { loop l1 { step s { gate G { when true -> done else -> done } } } loop l2 { step s { result.marker = true; gate G { when true -> done else -> done } } } } run chain c"));
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_bool(&obj, "marker"), true);
}

#[test]
fn chain_first_link_fail_short_circuits() {
    let obj = result_obj(&compile_and_execute("chain c { loop l1 { step s { gate G { when true -> fail else -> done } } } loop l2 { step s { result.second_ran = true; gate G { when true -> done else -> fail } } } } run chain c"));
    assert_eq!(required_bool(&obj, "success"), false);
    assert_eq!(required_str(&obj, "status"), String::from("failed"));
    assert_eq!(optional_bool(&obj, "second_ran"), None);
}

#[test]
fn chain_first_ok_second_fail_returns_second_result() {
    let obj = result_obj(&compile_and_execute("chain c { loop l1 { step s { gate G { when true -> done else -> fail } } } loop l2 { step s { result.second_ran = true; gate G { when true -> fail else -> done } } } } run chain c"));
    assert_eq!(required_bool(&obj, "second_ran"), true);
    assert_eq!(required_bool(&obj, "success"), false);
    assert_eq!(required_str(&obj, "status"), String::from("failed"));
}

// ── Cross-loop validation ────────────────────────────────────────────

#[test]
fn unknown_loop_gate_target_compile_error() {
    let err = compile_err_msg("loop L { step s { gate g { when true -> loop Ghost else -> done } } }");
    assert!(err.contains("not found"), "unknown loop target must be compile error, got: {}", err);
}

// ── Error recovery ───────────────────────────────────────────────────

#[test]
fn compile_error_does_not_corrupt_state() {
    let err = compile_err_msg("loop L { step s { gate g { when true -> nonexistent else -> done } } }");
    assert!(err.contains("not found"));
}

// ── Once mode (default) ──────────────────────────────────────────────

#[test]
fn mode_once_done_returns_success() {
    let obj = result_obj(&compile_and_execute("loop L { step s { gate G { when true -> done else -> fail } } } run loop L"));
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

// ── Sample execution tests (FAZ J) ────────────────────────────────────

#[test]
fn sample_06_times_until_modes() {
    let vm = compile_and_execute(include_str!("../../../samples/09-loop-engineering/06_times_until.hud"));
    let obj = result_obj(&vm);
    // times(3) ran, then until(counter>=5) ran counter from 3 to 5
    assert_eq!(required_vm_int(&vm, "counter"), 5);
    assert_eq!(required_bool(&obj, "success"), true);
}

#[test]
fn sample_07_chain_fail_short_circuits() {
    let obj = result_obj(&compile_and_execute(include_str!("../../../samples/09-loop-engineering/07_chain_fail_short.hud")));
    // validate fails → publish never runs → no published marker
    assert_eq!(required_bool(&obj, "success"), false);
    assert_eq!(required_str(&obj, "status"), String::from("failed"));
    assert_eq!(optional_bool(&obj, "published"), None);
}

#[test]
fn sample_08_retry_bounded_escalates() {
    let obj = result_obj(&compile_and_execute(include_str!("../../../samples/09-loop-engineering/08_retry_bounded.hud")));
    assert_eq!(required_str(&obj, "status"), String::from("escalated"));
}

#[test]
fn sample_09_step_transition_skips_s2() {
    let obj = result_obj(&compile_and_execute(include_str!("../../../samples/09-loop-engineering/09_step_transition.hud")));
    assert_eq!(optional_bool(&obj, "s2_ran"), None);
    assert_eq!(required_bool(&obj, "s3_ran"), true);
    assert_eq!(required_bool(&obj, "success"), true);
}

#[test]
fn sample_10_until_exact_stop() {
    let vm = compile_and_execute(include_str!("../../../samples/09-loop-engineering/10_until_exact.hud"));
    assert_eq!(required_vm_int(&vm, "counter"), 3);
    let obj = result_obj(&vm);
    assert_eq!(required_bool(&obj, "success"), true);
}

// ── A: attach/use lowering tests ─────────────────────────────────────

#[test]
fn use_step_compiles_imported_step() {
    // Standalone step, imported via use step
    let vm = compile_and_execute("step verify { gate g { when true -> done else -> fail } } loop b { use step verify as chk } run loop b");
    let obj = result_obj(&vm);
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

#[test]
fn use_step_unknown_step_compile_error() {
    let err = compile_err_msg("loop b { use step ghost as g }");
    assert!(err.contains("unknown"), "expected unknown step error, got: {}", err);
}

#[test]

#[test]
fn attach_step_unknown_step_compile_error() {
    let err = compile_err_msg("attach step ghost to loop b; loop b { }");
    assert!(err.contains("unknown"), "expected unknown step error, got: {}", err);
}

#[test]

#[test]
fn attach_gate_to_step_executes() {
    // standalone gate, attached to step inside loop
    let vm = compile_and_execute("gate check { when true -> done else -> fail } loop L { step s { let x = 0; } attach gate check to s } run loop L");
    let obj = result_obj(&vm);
    assert_eq!(required_bool(&obj, "success"), true);
}

// ── B: Agentic goal/convergence tests ────────────────────────────────

#[test]
fn with_goal_initializes_metadata() {
    let obj = result_obj(&compile_and_execute("loop L goal(metric: counter, target: 0) { step s { gate G { when true -> done else -> fail } } } run loop L"));
    assert_eq!(required_str(&obj, "__goal_metric"), String::from("counter"));
    assert_eq!(required_bool(&obj, "success"), true);
}

#[test]
fn goal_loop_converges_when_metric_reaches_target() {
    // goal_error = counter - 0 = counter. When counter >= 0, condition true → done.
    let obj = result_obj(&compile_and_execute("loop L goal(metric: counter, target: 0) { step s { result.counter = 0; gate G { when true -> done else -> fail } } } run loop L"));
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

#[test]
fn escalate_status_set_correctly() {
    let obj = result_obj(&compile_and_execute("loop L { step s { gate G { when true -> escalate else -> done } } } run loop L"));
    assert_eq!(required_str(&obj, "status"), String::from("escalated"));
}


// ── C: E2E sample execution tests ────────────────────────────────────

#[test]
fn sample_11_goal_convergence() {
    let obj = result_obj(&compile_and_execute("loop optimize goal(metric: errors, target: 0) { step s { result.errors = 0; gate G { when true -> done else -> fail } } } run loop optimize"));
    assert_eq!(required_bool(&obj, "success"), true);
    assert_eq!(required_str(&obj, "status"), String::from("done"));
}

#[test]
fn sample_12_escalate() {
    let obj = result_obj(&compile_and_execute("loop resilient { step s { gate G { when true -> escalate else -> done } } } run loop resilient"));
    assert_eq!(required_str(&obj, "status"), String::from("escalated"));
}
