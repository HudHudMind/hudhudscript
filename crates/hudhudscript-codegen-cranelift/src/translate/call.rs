//! Çağrı instruction'ları: CallStatic (uniform ABI, stack slot argümanları)
//! ve CallNative (runtime helper'ları).

use cranelift::prelude::types::{I32, I64, Type};
use cranelift::prelude::{FunctionBuilder, InstBuilder, Value, Variable};
use cranelift_module::Module;

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirFunction, MirInst, MirType};

use super::helpers::{
    bitcast_f64_to_i64, bitcast_i64_to_f64, operand, record, reject, OFF_STATUS, OFF_VALUE,
};

type Env = Vec<Option<(Value, MirType)>>;

pub(super) fn translate_call<M: Module>(
    inst: &MirInst,
    builder: &mut FunctionBuilder,
    env: &mut Env,
    module: &mut M,
    ptr: Type,
    func: &MirFunction,
    module_func_ids: &[cranelift_module::FuncId],
    _out_ptr: Value,
    ov_var: Variable,
    _is_aot: bool,
) -> Result<(), BackendError> {
    match inst {
        MirInst::CallStatic { dst, ty, callee, args, .. } => {
            // Callee'nin uniform ABI'sine çağrı:
            // (argc: i32, args: *i64, out: *JitExit) → void
            let fid = module_func_ids.get(callee.0 as usize).copied()
                .ok_or_else(|| reject(func, "CallStatic", &format!("unknown callee index {}", callee.0)))?;

            // Stack slot'lar: args dizisi + JitExit çıktısı
            let args_bytes = (args.len() * 8) as u32;
            let ss_args = builder.create_sized_stack_slot(
                cranelift::prelude::codegen::ir::StackSlotData::new(
                    cranelift::prelude::codegen::ir::StackSlotKind::ExplicitSlot,
                    args_bytes.max(8),
                    3,
                ),
            );
            let ss_out = builder.create_sized_stack_slot(
                cranelift::prelude::codegen::ir::StackSlotData::new(
                    cranelift::prelude::codegen::ir::StackSlotKind::ExplicitSlot,
                    16,
                    3,
                ),
            );

            // Args'ı stack slot'a yaz
            for (i, a) in args.iter().enumerate() {
                let (v, vt) = operand(env, *a, func)?;
                let v = if vt == MirType::F64 { bitcast_f64_to_i64(builder, v) } else { v };
                builder.ins().stack_store(v, ss_args, (i * 8) as i32);
            }

            // Stack adreslerini al
            let args_addr = builder.ins().stack_addr(ptr, ss_args, 0);
            let out_addr = builder.ins().stack_addr(ptr, ss_out, 0);
            let argc = builder.ins().iconst(I32, args.len() as i64);

            // Callee'yi çağır
            let callee_ref = module.declare_func_in_func(fid, builder.func);
            let _call = builder.ins().call(callee_ref, &[argc, args_addr, out_addr]);

            // JitExit.status'u oku (offset 0) ve ov_var'a akümüle et
            let callee_status = builder.ins().stack_load(I32, ss_out, OFF_STATUS);
            let cur_ov = builder.use_var(ov_var);
            let new_ov = builder.ins().bor(cur_ov, callee_status);
            builder.def_var(ov_var, new_ov);

            // JitExit.value'yı oku (offset 8)
            let raw = builder.ins().stack_load(I64, ss_out, OFF_VALUE);
            let result = if *ty == MirType::F64 { bitcast_i64_to_f64(builder, raw) } else { raw };
            record(env, *dst, result, *ty);
        }
        MirInst::CallNative { dst, ty, helper, args } => {
            super::call_native::translate_call_native(
                *dst, ty, helper, args, builder, env, module, ptr, func,
            )?;
        }
        _ => unreachable!("yalnız CallStatic/CallNative gelmeli"),
    }
    Ok(())
}
