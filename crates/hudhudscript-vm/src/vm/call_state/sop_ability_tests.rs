//! G06D acceptance tests for the SOP continuation phase order.

use crate::vm::call_state::{DeferredCallSite, MethodDispatchOutcome};
use crate::vm::sop_types::{CompositionMode, CompositionRule};
use crate::vm::VM;
use hudhudscript_bytecode::{Bytecode, SymId, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

const SUBJECTS_SCRIPT: &str = r#"
let trace = []
subject Main { can act, on act() { trace.push("base"); return "base" } }
subject Pre { can act, on act() { trace.push("before"); return "before" } }
subject Post { can act, on act() { trace.push("after"); return "after" } }
subject Alt { can act, on act() { trace.push("override"); return "override" } }
subject C1 { can act, on act() { trace.push("c1"); return "c1" } }
subject C2 { can act, on act() { trace.push("c2"); return "c2" } }
fn effect_log() { trace.push("effect"); return 0 }
"#;

fn compile_subjects() -> Bytecode {
    let ast = parse(SUBJECTS_SCRIPT).expect("subjects script must parse");
    let mut compiler = Compiler::new();
    compiler
        .compile(&ast)
        .expect("subjects script must compile")
}

fn subject_receiver(template: &str) -> Value16 {
    let mut object = hudhudscript_bytecode::ObjMap::default();
    object.insert(
        "__type".to_string(),
        Value16::string("subject_instance".to_string()),
    );
    object.insert(
        "__template".to_string(),
        Value16::string(template.to_string()),
    );
    object.insert(
        "__instance_id".to_string(),
        Value16::string("test-1".to_string()),
    );
    Value16::object(object)
}

fn run_sop(
    bytecode: &Bytecode,
    rules: Vec<CompositionRule>,
    with_effect: bool,
) -> (VM, Value16, Vec<String>) {
    let mut vm = VM::new();
    vm.execute(bytecode)
        .expect("subjects script must execute first");
    VM::reset_driver_entry_count_for_test();
    if !rules.is_empty() {
        vm.composition_rules.insert("Main::act".to_string(), rules);
    }
    if with_effect {
        vm.effects
            .insert("act".to_string(), "effect_log".to_string());
    }

    let receiver = subject_receiver("Main");
    let method_sym = SymId(hudhudscript_bytecode::interner::intern("act").0);
    let outcome = vm
        .dispatch_sop_method(
            &receiver,
            "act",
            method_sym,
            &[],
            bytecode,
            DeferredCallSite {
                dst: 200,
                origin_ip: 0,
            },
        )
        .expect("sop dispatch must succeed")
        .expect("sop dispatch must handle the subject");
    assert!(matches!(outcome, MethodDispatchOutcome::Deferred));
    vm.run_frame_loop(bytecode, &[], 0)
        .expect("sop sequence must run on the trampoline");

    let result = vm.registers[200];
    let trace = vm
        .get_variable_owned("trace")
        .expect("trace must be published")
        .as_array()
        .expect("trace must be an array")
        .iter()
        .map(|item| item.as_string().unwrap_or_default())
        .collect();
    (vm, result, trace)
}

fn rule(mode: CompositionMode) -> CompositionRule {
    CompositionRule {
        ability_name: "act".to_string(),
        mode,
    }
}

#[test]
fn sop_continuation_preserves_phase_order() {
    let bytecode = compile_subjects();
    let (vm, result, trace) = run_sop(
        &bytecode,
        vec![rule(CompositionMode::Before("Pre".to_string()))],
        false,
    );
    assert_eq!(result.as_string(), Some("base".to_string()));
    assert_eq!(trace, vec!["before".to_string(), "base".to_string()]);
    assert_eq!(VM::driver_entry_count_for_test(), 1);
}

#[test]
fn sop_override_replaces_base_but_keeps_before_after_effect() {
    let bytecode = compile_subjects();
    let (_vm, result, trace) = run_sop(
        &bytecode,
        vec![
            rule(CompositionMode::Before("Pre".to_string())),
            rule(CompositionMode::Override("Alt".to_string())),
            rule(CompositionMode::After("Post".to_string())),
        ],
        true,
    );
    assert_eq!(result.as_string(), Some("override".to_string()));
    assert_eq!(
        trace,
        vec![
            "before".to_string(),
            "override".to_string(),
            "after".to_string(),
            "effect".to_string(),
        ]
    );
}

#[test]
fn sop_combine_last_result_wins() {
    let bytecode = compile_subjects();
    let (_vm, result, trace) = run_sop(
        &bytecode,
        vec![rule(CompositionMode::Combine(vec![
            "C1".to_string(),
            "C2".to_string(),
        ]))],
        false,
    );
    assert_eq!(result.as_string(), Some("c2".to_string()));
    assert_eq!(
        trace,
        vec!["base".to_string(), "c1".to_string(), "c2".to_string(),]
    );
}

#[test]
fn sop_sequence_uses_single_native_driver() {
    let bytecode = compile_subjects();
    let (vm, result, _trace) = run_sop(
        &bytecode,
        vec![
            rule(CompositionMode::Before("Pre".to_string())),
            rule(CompositionMode::After("Post".to_string())),
        ],
        true,
    );
    assert_eq!(result.as_string(), Some("base".to_string()));
    assert_eq!(VM::driver_entry_count_for_test(), 1);
}
