use hudhudscript_bytecode::{
    Bytecode, ClassDeclPayload, EnumDeclPayload, Instruction, PromiseState16, SymId, Value16,
};
use hudhudscript_shared_builtins::interner::intern;
use std::collections::HashMap;

#[test]
fn test_enum_operations() {
    let mut bytecode = Bytecode::new();
    let payload = EnumDeclPayload {
        name: SymId(intern("Color").0),
        variants: vec![
            SymId(intern("Red").0),
            SymId(intern("Green").0),
            SymId(intern("Blue").0),
        ],
    };
    let enum_idx = bytecode.add_enum_decl_payload(payload);
    let _enum_instr = Instruction::EnumDecl(enum_idx);

    let sym_enum = bytecode.intern_symbol("Color");
    let sym_variant = bytecode.intern_symbol("Red");
    let match_payload = bytecode.add_two_sym_payload(sym_enum, sym_variant);
    let _match_instr = Instruction::MatchVariant(match_payload);
}

#[test]
fn test_spawn_operations() {
    let mut bytecode = Bytecode::new();
    let sym = SymId(intern("Worker").0);
    let payload = bytecode.add_call_payload(sym, 0) as u16;
    let _instr = Instruction::Spawn { payload_idx: payload as u16, first_arg: 0, arg_count: 0 };
}

#[test]
fn test_memory_operations() {
    let mut bytecode = Bytecode::new();
    let store_sym = SymId(intern("mystore").0);
    let payload = bytecode.add_opt_sym_payload(Some(store_sym));
    let _remember = Instruction::Remember { store_idx: payload as u16, src: 0 };
    let _forget = Instruction::Forget { store_idx: payload as u16, src: 0 };
    let _recall = Instruction::Recall { store_idx: payload as u16, src: 0, dst: 0 };
}

#[test]
fn test_tvar_operations() {
    let mut bytecode = Bytecode::new();
    let sym_new = SymId(intern("tvar_new").0);
    let payload_new = bytecode.add_call_payload(sym_new, 0) as u16;
    let _new_call = Instruction::Call { dst: 0, payload_idx: payload_new, first_arg: 0, arg_count: 0 };

    let sym_read = SymId(intern("tvar_read").0);
    let payload_read = bytecode.add_call_payload(sym_read, 0) as u16;
    let _read_call = Instruction::Call { dst: 0, payload_idx: payload_read, first_arg: 0, arg_count: 0 };

    let sym_write = SymId(intern("tvar_write").0);
    let payload_write = bytecode.add_call_payload(sym_write, 0) as u16;
    let _write_call = Instruction::Call { dst: 0, payload_idx: payload_write, first_arg: 0, arg_count: 0 };
}

#[test]
fn test_option_operations() {
    let mut bytecode = Bytecode::new();
    let sym_some = SymId(intern("Some").0);
    let payload_some = bytecode.add_call_payload(sym_some, 1) as u16;
    let _some_call = Instruction::Call { dst: 0, payload_idx: payload_some, first_arg: 0, arg_count: 0 };

    let sym_is_some = SymId(intern("is_some").0);
    let payload_is_some = bytecode.add_call_payload(sym_is_some, 1) as u16;
    let _is_some_call = Instruction::Call { dst: 0, payload_idx: payload_is_some, first_arg: 0, arg_count: 0 };

    let sym_is_none = SymId(intern("is_none").0);
    let payload_is_none = bytecode.add_call_payload(sym_is_none, 1) as u16;
    let _is_none_call = Instruction::Call { dst: 0, payload_idx: payload_is_none, first_arg: 0, arg_count: 0 };
}

#[test]
fn test_result_operations() {
    let mut bytecode = Bytecode::new();
    let sym_err = SymId(intern("Err").0);
    let payload_err = bytecode.add_call_payload(sym_err, 1) as u16;
    let _err_call = Instruction::Call { dst: 0, payload_idx: payload_err, first_arg: 0, arg_count: 0 };

    let sym_is_ok = SymId(intern("is_ok").0);
    let payload_is_ok = bytecode.add_call_payload(sym_is_ok, 1) as u16;
    let _is_ok_call = Instruction::Call { dst: 0, payload_idx: payload_is_ok, first_arg: 0, arg_count: 0 };

    let sym_is_err = SymId(intern("is_err").0);
    let payload_is_err = bytecode.add_call_payload(sym_is_err, 1) as u16;
    let _is_err_call = Instruction::Call { dst: 0, payload_idx: payload_is_err, first_arg: 0, arg_count: 0 };

    let sym_unwrap_or = SymId(intern("unwrap_or").0);
    let payload_unwrap_or = bytecode.add_call_payload(sym_unwrap_or, 2) as u16;
    let _unwrap_or_call = Instruction::Call { dst: 0, payload_idx: payload_unwrap_or, first_arg: 0, arg_count: 0 };
}

#[test]
fn test_len_operations() {
    let mut bytecode = Bytecode::new();
    let sym = SymId(intern("len").0);
    let payload = bytecode.add_call_payload(sym, 1) as u16;
    let _len_call = Instruction::Call { dst: 0, payload_idx: payload as u16, first_arg: 0, arg_count: 0 };
}

#[test]
fn test_relation_operations() {
    let mut bytecode = Bytecode::new();
    let sym_var = bytecode.intern_symbol("relation");
    let sym_type = bytecode.intern_symbol("Relation");
    let decl_payload = bytecode.add_two_sym_payload(sym_var, sym_type);
    let _decl = Instruction::DeclStore { payload_idx: decl_payload as u16, src: 0 };

    let sym_get = SymId(intern("get_relation").0);
    let payload_get = bytecode.add_call_payload(sym_get, 1) as u16;
    let _get_call = Instruction::Call { dst: 0, payload_idx: payload_get, first_arg: 0, arg_count: 0 };

    let sym_update = SymId(intern("update_relation").0);
    let payload_update = bytecode.add_call_payload(sym_update, 2) as u16;
    let _update_call = Instruction::Call { dst: 0, payload_idx: payload_update, first_arg: 0, arg_count: 0 };

    let sym_enforce = SymId(intern("enforce_relation").0);
    let payload_enforce = bytecode.add_call_payload(sym_enforce, 2) as u16;
    let _enforce_call = Instruction::Call { dst: 0, payload_idx: payload_enforce, first_arg: 0, arg_count: 0 };
}

#[test]
fn test_constitution_operations() {
    let mut bytecode = Bytecode::new();
    let sym_register = SymId(intern("register_constitution").0);
    let payload_register = bytecode.add_call_payload(sym_register, 1) as u16;
    let _register = Instruction::Call { dst: 0, payload_idx: payload_register, first_arg: 0, arg_count: 0 };

    let sym_activate = SymId(intern("activate_constitution").0);
    let payload_activate = bytecode.add_call_payload(sym_activate, 1) as u16;
    let _activate = Instruction::Call { dst: 0, payload_idx: payload_activate, first_arg: 0, arg_count: 0 };

    let sym_deactivate = SymId(intern("deactivate_constitution").0);
    let payload_deactivate = bytecode.add_call_payload(sym_deactivate, 1) as u16;
    let _deactivate = Instruction::Call { dst: 0, payload_idx: payload_deactivate, first_arg: 0, arg_count: 0 };

    let sym_check = SymId(intern("check_constitution_compliance").0);
    let payload_check = bytecode.add_call_payload(sym_check, 1) as u16;
    let _check = Instruction::Call { dst: 0, payload_idx: payload_check, first_arg: 0, arg_count: 0 };
}

#[test]
fn test_mcp_operations() {
    let mut bytecode = Bytecode::new();
    let sym = SymId(intern("mcp_call").0);
    let payload = bytecode.add_call_payload(sym, 1) as u16;
    let _mcp_call = Instruction::Call { dst: 0, payload_idx: payload as u16, first_arg: 0, arg_count: 0 };
}

#[test]
fn test_effect_operations() {
    let mut bytecode = Bytecode::new();
    let sym_var = bytecode.intern_symbol("effect");
    let sym_type = bytecode.intern_symbol("Effect");
    let payload = bytecode.add_two_sym_payload(sym_var, sym_type);
    let _decl = Instruction::DeclStore { payload_idx: payload as u16, src: 0 };
}

#[test]
fn test_class_operations() {
    let mut bytecode = Bytecode::new();
    let payload = ClassDeclPayload {
        is_abstract: false,
        method_access: vec![],
        name: SymId(intern("Animal").0),
        parent: None,
        methods: vec![SymId(intern("speak").0)],
    };
    let idx = bytecode.add_class_decl_payload(payload);
    let _decl = Instruction::ClassDecl(idx);
}

#[test]
fn test_promise_creation() {
    let _v = Value16::promise(PromiseState16::Rejected("fail!".to_string()));
    let _v2 = Value16::promise(PromiseState16::AsyncPending("42".to_string()));
}

#[test]
fn test_object_creation() {
    let fields = HashMap::new();
    let _v = Value16::object(fields);
}
