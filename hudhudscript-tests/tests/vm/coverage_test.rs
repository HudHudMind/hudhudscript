use hudhudscript_bytecode::{Bytecode, Instruction, Value16, PromiseState16, SymId, DestructObjectPayload, LoadModulePayload};
use hudhudscript_shared_builtins::interner::intern;
use std::collections::HashMap;
use hudhudscript_bytecode::Value16;

#[test]
fn test_store_typed_coverage() {
    let mut bytecode = Bytecode::new();
    let sym_n = bytecode.intern_symbol("n");
    let sym_type = bytecode.intern_symbol("number");
    let payload = bytecode.add_two_sym_payload(sym_n, sym_type);
    let _instr = Instruction::StoreTyped(payload);
}

#[test]
fn test_call_coverage() {
    let mut bytecode = Bytecode::new();
    let sym = SymId(intern("Promise.resolve").0);
    let payload = bytecode.add_call_payload(sym, 1);
    let _instr = Instruction::Call(payload);
}

#[test]
fn test_spawn_coverage() {
    let mut bytecode = Bytecode::new();
    let sym = SymId(intern("Agent").0);
    let payload = bytecode.add_call_payload(sym, 0);
    let _instr = Instruction::Spawn { name_sym: 0, first_arg: 0, arg_count: 0 };
}

#[test]
fn test_remember_recall_forget_coverage() {
    let mut bytecode = Bytecode::new();
    let store_sym = SymId(intern("default").0);
    let payload = bytecode.add_opt_sym_payload(Some(store_sym));
    let _instr_remember = Instruction::Remember { store_idx: payload, src: 0 };
    let _instr_recall = Instruction::Recall { store_idx: payload, src: 0, dst: 0 };
    let _instr_forget = Instruction::Forget { store_idx: payload, src: 0 };
}

#[test]
fn test_load_module_coverage() {
    let mut bytecode = Bytecode::new();
    let payload = LoadModulePayload {
        path: "utils/helpers".to_string(),
        alias: Some(SymId(intern("helpers").0)),
    };
    let idx = bytecode.add_load_module_payload(payload);
    let _instr = Instruction::LoadModule(idx);
}

#[test]
fn test_destruct_object_coverage() {
    let mut bytecode = Bytecode::new();
    let payload = DestructObjectPayload {
        used_keys: vec![SymId(intern("x").0), SymId(intern("y").0)],
    };
    let idx = bytecode.add_destruct_object_payload(payload);
    let _instr = Instruction::DestructObject(idx);
}

#[test]
fn test_match_variant_coverage() {
    let mut bytecode = Bytecode::new();
    let sym_enum = bytecode.intern_symbol("Status");
    let sym_variant = bytecode.intern_symbol("Ok");
    let payload = bytecode.add_two_sym_payload(sym_enum, sym_variant);
    let _instr = Instruction::MatchVariant(payload);
}

#[test]
fn test_decl_store_coverage() {
    let mut bytecode = Bytecode::new();
    let sym_var = bytecode.intern_symbol("protocol");
    let sym_type = bytecode.intern_symbol("Protocol");
    let payload = bytecode.add_two_sym_payload(sym_var, sym_type);
    let _instr = Instruction::DeclStore { payload_idx: payload, src: 0 };
}

#[test]
fn test_promise_values() {
    let _v = Value16::promise(PromiseState16::Resolved(Box::new(Value16::number(99.0))));
    let _v2 = Value16::promise(PromiseState16::Pending);
    let _v3 = Value16::promise(PromiseState16::Rejected("nope".to_string()));
    let _v4 = Value16::promise(PromiseState16::AsyncPending("42".to_string()));
}

#[test]
fn test_object_values() {
    let obj = HashMap::new();
    let _v = Value16::object(obj);
}
