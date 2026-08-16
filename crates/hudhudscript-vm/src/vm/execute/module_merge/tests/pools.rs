use super::{add_functions, call_payload, chunk, symbol};
use hudhudscript_bytecode::{
    Bytecode, ClassDeclPayload, ClassStaticDeclPayload, CmpJumpPayload, DefineFunctionPayload,
    DestructObjectPayload, EnumDeclPayload, Instruction, LoadModulePayload, LoopPayload,
    OptSymPayload, SuperInstrPayload, TraitCheckPayload, TwoSymPayload,
};
use std::sync::Arc;

use super::super::merge_module_bytecode;

#[derive(Debug, PartialEq, Eq)]
struct PoolLengths {
    calls: usize,
    load_modules: usize,
    define_functions: usize,
    enum_decls: usize,
    class_decls: usize,
    trait_checks: usize,
    class_static_decls: usize,
    destruct_objects: usize,
    two_syms: usize,
    opt_syms: usize,
    loops: usize,
    super_instrs: usize,
    cmp_jumps: usize,
    char_tables: usize,
    numeric_constants: usize,
    int_constants: usize,
}

impl PoolLengths {
    fn of(bytecode: &Bytecode) -> Self {
        Self {
            calls: bytecode.call_payloads.len(),
            load_modules: bytecode.load_module_payloads.len(),
            define_functions: bytecode.define_function_payloads.len(),
            enum_decls: bytecode.enum_decl_payloads.len(),
            class_decls: bytecode.class_decl_payloads.len(),
            trait_checks: bytecode.trait_check_payloads.len(),
            class_static_decls: bytecode.class_static_decl_payloads.len(),
            destruct_objects: bytecode.destruct_object_payloads.len(),
            two_syms: bytecode.two_sym_payloads.len(),
            opt_syms: bytecode.opt_sym_payloads.len(),
            loops: bytecode.loop_payloads.len(),
            super_instrs: bytecode.super_instr_payloads.len(),
            cmp_jumps: bytecode.cmp_jump_payloads.len(),
            char_tables: bytecode.char_dispatch_tables.len(),
            numeric_constants: bytecode.numeric_constants.len(),
            int_constants: bytecode.int_constants.len(),
        }
    }

    fn plus(&self, other: &Self) -> Self {
        Self {
            calls: self.calls + other.calls,
            load_modules: self.load_modules + other.load_modules,
            define_functions: self.define_functions + other.define_functions,
            enum_decls: self.enum_decls + other.enum_decls,
            class_decls: self.class_decls + other.class_decls,
            trait_checks: self.trait_checks + other.trait_checks,
            class_static_decls: self.class_static_decls + other.class_static_decls,
            destruct_objects: self.destruct_objects + other.destruct_objects,
            two_syms: self.two_syms + other.two_syms,
            opt_syms: self.opt_syms + other.opt_syms,
            loops: self.loops + other.loops,
            super_instrs: self.super_instrs + other.super_instrs,
            cmp_jumps: self.cmp_jumps + other.cmp_jumps,
            char_tables: self.char_tables + other.char_tables,
            numeric_constants: self.numeric_constants + other.numeric_constants,
            int_constants: self.int_constants + other.int_constants,
        }
    }
}

fn populate_every_pool(bytecode: &mut Bytecode, function_name: &str, marker: u32) {
    bytecode.call_payloads.push(call_payload(function_name, 0));
    bytecode.load_module_payloads.push(LoadModulePayload {
        path: format!("module_{marker}.hud"),
        alias: Some(symbol(&format!("alias_{marker}"))),
        base_dir: Some(format!("base_{marker}")),
    });
    bytecode
        .define_function_payloads
        .push(DefineFunctionPayload {
            name: symbol(&format!("binding_{marker}")),
            chunk_name: function_name.to_string(),
        });
    bytecode.enum_decl_payloads.push(EnumDeclPayload {
        name: symbol(&format!("Enum{marker}")),
        variants: vec![symbol(&format!("Variant{marker}"))],
    });
    bytecode.class_decl_payloads.push(ClassDeclPayload {
        name: symbol(&format!("Class{marker}")),
        parent: None,
        methods: vec![],
        method_access: vec![],
        is_abstract: false,
    });
    bytecode.trait_check_payloads.push(TraitCheckPayload {
        class_name: symbol(&format!("Class{marker}")),
        trait_name: symbol(&format!("Trait{marker}")),
        required_methods: vec![],
        class_methods: vec![],
    });
    bytecode
        .class_static_decl_payloads
        .push(ClassStaticDeclPayload {
            class_name: symbol(&format!("Class{marker}")),
            static_methods: vec![],
            static_fields: vec![],
        });
    bytecode
        .destruct_object_payloads
        .push(DestructObjectPayload {
            used_keys: vec![symbol(&format!("key_{marker}"))],
        });
    bytecode.two_sym_payloads.push(TwoSymPayload {
        first: marker,
        second: marker + 1,
    });
    bytecode.opt_sym_payloads.push(OptSymPayload {
        sym: Some(symbol(&format!("store_{marker}"))),
    });
    bytecode.loop_payloads.push(LoopPayload {
        start: marker,
        end: marker + 1,
    });
    bytecode.super_instr_payloads.push(SuperInstrPayload {
        call_idx: 0,
        slot: marker,
        imm: marker as i16,
        offset: marker as i32,
        call_dst: marker,
        arg_reg: marker as u8,
    });
    bytecode.cmp_jump_payloads.push(CmpJumpPayload {
        src1: marker as u8,
        src2: marker as u8 + 1,
        target: marker,
    });
    bytecode.char_dispatch_tables.push(vec![marker as i16; 256]);
    bytecode.numeric_constants.push(f64::from(marker).to_bits());
    bytecode.int_constants.push(i64::from(marker));
}

#[test]
fn module_payload_pools_are_merged_once() {
    let mut target = Bytecode::default();
    target.add_function(
        "parent".to_string(),
        chunk(vec![Instruction::Return { src: 0 }]),
    );
    populate_every_pool(&mut target, "parent", 3);

    let mut source = Bytecode::default();
    add_functions(&source, "module", 3);
    source.action_registry.borrow_mut().insert(
        "Module.first".to_string(),
        chunk(vec![Instruction::Return { src: 0 }]),
    );
    source.action_registry.borrow_mut().insert(
        "Module.second".to_string(),
        chunk(vec![Instruction::Return { src: 0 }]),
    );
    populate_every_pool(&mut source, "module_0", 7);

    let before = PoolLengths::of(&target);
    let source_lengths = PoolLengths::of(&source);
    merge_module_bytecode(&source, &target).expect("module merge must succeed");

    assert_eq!(PoolLengths::of(&target), before.plus(&source_lengths));
    assert_eq!(target.action_registry.borrow().len(), 2);
    assert_eq!(target.super_instr_payloads[1].call_idx, 1);
    assert_eq!(target.char_dispatch_tables[1], vec![7; 256]);
    assert_eq!(target.numeric_constants[1], 7.0_f64.to_bits());
    assert_eq!(target.int_constants[1], 7);
}

#[test]
fn duplicate_module_function_keeps_existing_collision_semantics() {
    let target = Bytecode::default();
    let parent_chunk = chunk(vec![Instruction::Return { src: 1 }]);
    target.add_function("shared_fn".to_string(), Arc::clone(&parent_chunk));

    let mut source = Bytecode::default();
    source.add_function(
        "shared_fn".to_string(),
        chunk(vec![Instruction::Return { src: 2 }]),
    );
    source.call_payloads.push(call_payload("shared_fn", 0));

    merge_module_bytecode(&source, &target).expect("module merge must succeed");

    assert_eq!(target.function_count(), 1);
    let retained = target
        .get_function("shared_fn")
        .expect("parent function must remain registered");
    assert!(Arc::ptr_eq(&retained, &parent_chunk));
    assert_eq!(target.call_payloads[0].function_idx, 0);
    assert_eq!(
        target
            .function_name_at(0)
            .expect("index zero must be valid"),
        "shared_fn"
    );
}

#[test]
fn module_char_dispatch_table_is_rebased() {
    let mut target = Bytecode::default();
    target.char_dispatch_tables.push(vec![11; 256]);

    let mut source = Bytecode::default();
    source.char_dispatch_tables.push(vec![22; 256]);
    source.add_function(
        "dispatch".to_string(),
        chunk(vec![
            Instruction::CharDispatch {
                src: 0,
                table_idx: 0,
            },
            Instruction::Return { src: 0 },
        ]),
    );

    merge_module_bytecode(&source, &target).expect("module merge must succeed");

    let merged = target
        .get_function("dispatch")
        .expect("dispatch function must be copied");
    match &merged.instructions[0] {
        Instruction::CharDispatch { table_idx, .. } => assert_eq!(*table_idx, 1),
        instruction => panic!("expected CharDispatch, got {instruction:?}"),
    }
    assert_eq!(target.char_dispatch_tables[0], vec![11; 256]);
    assert_eq!(target.char_dispatch_tables[1], vec![22; 256]);
}
