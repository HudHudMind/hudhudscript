use super::*;
use hudhudscript_bytecode::{Bytecode, CallPayload, FunctionChunk};
use std::ops::Range;

#[derive(Clone, Copy, Debug)]
pub(super) struct PayloadBases {
    call_base: u16,
    load_module_base: u32,
    define_function_base: u32,
    enum_decl_base: u32,
    class_decl_base: u32,
    trait_check_base: u32,
    class_static_decl_base: u32,
    destruct_object_base: u32,
    two_sym_base: u32,
    opt_sym_base: u32,
    loop_base: u32,
    super_instr_base: u32,
    cmp_jump_base: u32,
    char_dispatch_base: u16,
    num_base: u16,
    int_base: u16,
}

impl PayloadBases {
    fn from_bytecode(target: &Bytecode) -> CompileResult<Self> {
        Ok(Self {
            call_base: len_u16("call_payloads", target.call_payloads.len())?,
            load_module_base: len_u32("load_module_payloads", target.load_module_payloads.len())?,
            define_function_base: len_u32(
                "define_function_payloads",
                target.define_function_payloads.len(),
            )?,
            enum_decl_base: len_u32("enum_decl_payloads", target.enum_decl_payloads.len())?,
            class_decl_base: len_u32("class_decl_payloads", target.class_decl_payloads.len())?,
            trait_check_base: len_u32("trait_check_payloads", target.trait_check_payloads.len())?,
            class_static_decl_base: len_u32(
                "class_static_decl_payloads",
                target.class_static_decl_payloads.len(),
            )?,
            destruct_object_base: len_u32(
                "destruct_object_payloads",
                target.destruct_object_payloads.len(),
            )?,
            two_sym_base: len_u32("two_sym_payloads", target.two_sym_payloads.len())?,
            opt_sym_base: len_u32("opt_sym_payloads", target.opt_sym_payloads.len())?,
            loop_base: len_u32("loop_payloads", target.loop_payloads.len())?,
            super_instr_base: len_u32("super_instr_payloads", target.super_instr_payloads.len())?,
            cmp_jump_base: len_u32("cmp_jump_payloads", target.cmp_jump_payloads.len())?,
            char_dispatch_base: len_u16("char_dispatch_tables", target.char_dispatch_tables.len())?,
            num_base: len_u16("numeric_constants", target.numeric_constants.len())?,
            int_base: len_u16("int_constants", target.int_constants.len())?,
        })
    }
}

pub(super) fn merge_module_payload_pools_once(
    source: &Bytecode,
    target: &mut Bytecode,
) -> CompileResult<(PayloadBases, Range<usize>)> {
    let bases = PayloadBases::from_bytecode(target)?;
    validate_pool_growth(source, bases)?;

    let call_start = target.call_payloads.len();
    target
        .call_payloads
        .extend(source.call_payloads.iter().map(|payload| CallPayload {
            sym: payload.sym,
            arg_count: payload.arg_count,
            function_idx: u32::MAX,
            builtin_method_idx: payload.builtin_method_idx,
        }));
    let call_end = target.call_payloads.len();

    target
        .load_module_payloads
        .extend_from_slice(&source.load_module_payloads);
    target
        .define_function_payloads
        .extend_from_slice(&source.define_function_payloads);
    target
        .enum_decl_payloads
        .extend_from_slice(&source.enum_decl_payloads);
    target
        .class_decl_payloads
        .extend_from_slice(&source.class_decl_payloads);
    target
        .trait_check_payloads
        .extend_from_slice(&source.trait_check_payloads);
    target
        .class_static_decl_payloads
        .extend_from_slice(&source.class_static_decl_payloads);
    target
        .destruct_object_payloads
        .extend_from_slice(&source.destruct_object_payloads);
    target
        .two_sym_payloads
        .extend_from_slice(&source.two_sym_payloads);
    target
        .opt_sym_payloads
        .extend_from_slice(&source.opt_sym_payloads);
    target
        .loop_payloads
        .extend_from_slice(&source.loop_payloads);
    target
        .cmp_jump_payloads
        .extend_from_slice(&source.cmp_jump_payloads);

    for payload in &source.super_instr_payloads {
        let mut payload = *payload;
        payload.call_idx = add_u32(payload.call_idx, u32::from(bases.call_base), "super call")?;
        target.super_instr_payloads.push(payload);
    }

    target
        .char_dispatch_tables
        .extend_from_slice(&source.char_dispatch_tables);
    target
        .numeric_constants
        .extend_from_slice(&source.numeric_constants);
    target
        .int_constants
        .extend_from_slice(&source.int_constants);

    Ok((bases, call_start..call_end))
}

pub(super) fn remap_chunk_indices(
    chunk: &mut FunctionChunk,
    bases: PayloadBases,
) -> CompileResult<()> {
    for instruction in &mut chunk.instructions {
        match instruction {
            Instruction::Call { payload_idx, .. }
            | Instruction::MethodCall { payload_idx, .. }
            | Instruction::SuperCall { payload_idx, .. }
            | Instruction::NewInstance { payload_idx, .. }
            | Instruction::MakeGenerator { payload_idx, .. } => {
                *payload_idx = add_u16(*payload_idx, bases.call_base, "call payload")?;
            }
            Instruction::LoadModule(index) => {
                *index = add_u32(*index, bases.load_module_base, "load-module payload")?;
            }
            Instruction::DefineFunction(index) => {
                *index = add_u32(
                    *index,
                    bases.define_function_base,
                    "define-function payload",
                )?;
            }
            Instruction::EnumDecl(index) => {
                *index = add_u32(*index, bases.enum_decl_base, "enum payload")?;
            }
            Instruction::ClassDecl(index) => {
                *index = add_u32(*index, bases.class_decl_base, "class payload")?;
            }
            Instruction::TraitCheck(index) => {
                *index = add_u32(*index, bases.trait_check_base, "trait payload")?;
            }
            Instruction::ClassStaticDecl(index) => {
                *index = add_u32(*index, bases.class_static_decl_base, "class-static payload")?;
            }
            Instruction::DestructObject(index) => {
                *index = add_u32(
                    *index,
                    bases.destruct_object_base,
                    "destruct-object payload",
                )?;
            }
            Instruction::MatchVariant(index) | Instruction::GetStatic(index) => {
                *index = add_u32(*index, bases.two_sym_base, "two-symbol payload")?;
            }
            Instruction::DeclStore { payload_idx, .. } => {
                *payload_idx = add_u16_u32(*payload_idx, bases.two_sym_base, "two-symbol payload")?;
            }
            Instruction::Remember { store_idx, .. }
            | Instruction::Recall { store_idx, .. }
            | Instruction::Forget { store_idx, .. } => {
                *store_idx =
                    add_u16_u32(*store_idx, bases.opt_sym_base, "optional-symbol payload")?;
            }
            Instruction::LoopBegin(index) => {
                *index = add_u32(*index, bases.loop_base, "loop payload")?;
            }
            Instruction::IntSubCall1(index)
            | Instruction::IntAddCall1(index)
            | Instruction::IntLeJumpIfFalse(index)
            | Instruction::IntLtJumpIfFalse(index) => {
                *index = add_u32(*index, bases.super_instr_base, "super-instruction payload")?;
            }
            Instruction::LoadNumConst { const_idx, .. } => {
                *const_idx = add_u16(*const_idx, bases.num_base, "numeric constant")?;
            }
            Instruction::LoadIntConst { const_idx, .. }
            | Instruction::ArrayPushIntConst { const_idx, .. } => {
                *const_idx = add_u16(*const_idx, bases.int_base, "integer constant")?;
            }
            Instruction::IntLtRRJumpPacked(index) | Instruction::IntLeRRJumpPacked(index) => {
                *index = add_u32(*index, bases.cmp_jump_base, "compare-jump payload")?;
            }
            Instruction::IntCmpRRJumpPacked { payload_idx, .. } => {
                *payload_idx =
                    add_u16_u32(*payload_idx, bases.cmp_jump_base, "compare-jump payload")?;
            }
            Instruction::CharDispatch { table_idx, .. } => {
                *table_idx = add_u16(*table_idx, bases.char_dispatch_base, "char-dispatch table")?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_pool_growth(source: &Bytecode, bases: PayloadBases) -> CompileResult<()> {
    validate_u16_growth("call_payloads", bases.call_base, source.call_payloads.len())?;
    validate_u16_growth(
        "char_dispatch_tables",
        bases.char_dispatch_base,
        source.char_dispatch_tables.len(),
    )?;
    validate_u16_growth(
        "numeric_constants",
        bases.num_base,
        source.numeric_constants.len(),
    )?;
    validate_u16_growth("int_constants", bases.int_base, source.int_constants.len())?;

    let pools = [
        (
            "load_module_payloads",
            bases.load_module_base,
            source.load_module_payloads.len(),
        ),
        (
            "define_function_payloads",
            bases.define_function_base,
            source.define_function_payloads.len(),
        ),
        (
            "enum_decl_payloads",
            bases.enum_decl_base,
            source.enum_decl_payloads.len(),
        ),
        (
            "class_decl_payloads",
            bases.class_decl_base,
            source.class_decl_payloads.len(),
        ),
        (
            "trait_check_payloads",
            bases.trait_check_base,
            source.trait_check_payloads.len(),
        ),
        (
            "class_static_decl_payloads",
            bases.class_static_decl_base,
            source.class_static_decl_payloads.len(),
        ),
        (
            "destruct_object_payloads",
            bases.destruct_object_base,
            source.destruct_object_payloads.len(),
        ),
        (
            "two_sym_payloads",
            bases.two_sym_base,
            source.two_sym_payloads.len(),
        ),
        (
            "opt_sym_payloads",
            bases.opt_sym_base,
            source.opt_sym_payloads.len(),
        ),
        ("loop_payloads", bases.loop_base, source.loop_payloads.len()),
        (
            "super_instr_payloads",
            bases.super_instr_base,
            source.super_instr_payloads.len(),
        ),
        (
            "cmp_jump_payloads",
            bases.cmp_jump_base,
            source.cmp_jump_payloads.len(),
        ),
    ];
    for (name, base, source_len) in pools {
        validate_u32_growth(name, base, source_len)?;
    }
    Ok(())
}

fn validate_u16_growth(name: &str, base: u16, source_len: usize) -> CompileResult<()> {
    if source_len > 0 {
        add_u16(base, len_u16(name, source_len - 1)?, name)?;
    }
    Ok(())
}

fn validate_u32_growth(name: &str, base: u32, source_len: usize) -> CompileResult<()> {
    if source_len > 0 {
        add_u32(base, len_u32(name, source_len - 1)?, name)?;
    }
    Ok(())
}

fn len_u16(name: &str, value: usize) -> CompileResult<u16> {
    u16::try_from(value).map_err(|_| merge_error(format!("{} length {} exceeds u16", name, value)))
}

pub(super) fn len_u32(name: &str, value: usize) -> CompileResult<u32> {
    u32::try_from(value).map_err(|_| merge_error(format!("{} length {} exceeds u32", name, value)))
}

fn add_u16(value: u16, base: u16, name: &str) -> CompileResult<u16> {
    value
        .checked_add(base)
        .ok_or_else(|| merge_error(format!("{} index overflow", name)))
}

fn add_u32(value: u32, base: u32, name: &str) -> CompileResult<u32> {
    value
        .checked_add(base)
        .ok_or_else(|| merge_error(format!("{} index overflow", name)))
}

fn add_u16_u32(value: u16, base: u32, name: &str) -> CompileResult<u16> {
    let base = u16::try_from(base)
        .map_err(|_| merge_error(format!("{} base {} exceeds u16", name, base)))?;
    add_u16(value, base, name)
}
