//! Blok terminator'ları: Branch/CondBranch (phi argümanları), Return
//! (bayrak select zinciri + JitExit yazımı), Trap, ReturnVoid.

use std::collections::HashMap;


use cranelift::prelude::codegen::ir::condcodes::IntCC;
use cranelift::prelude::types::{I32, I64};
use cranelift::prelude::{Block, FunctionBuilder, InstBuilder, MemFlags, Value};

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirBlock, MirFunction, MirTerminator, MirType};
use hudhudscript_native_abi::JIT_EXIT_OVERFLOW;

use super::helpers::{emit_return_exit, operand, reject, OFF_STATUS, OFF_VALUE};

type Env = Vec<Option<(Value, MirType)>>;

pub(super) fn emit_terminator(
    builder: &mut FunctionBuilder,
    env: &Env,
    func: &MirFunction,
    block: &MirBlock,
    block_map: &HashMap<u32, Block>,
    out_ptr: Value,
    _ov_var: cranelift::prelude::Variable,
    dz_var: cranelift::prelude::Variable,
) -> Result<(), BackendError> {
    // ── Blok terminator'u ──
    match &block.terminator {
        Some(MirTerminator::Branch { target, args }) => {
            let dst = *block_map.get(&target.0).ok_or_else(|| {
                reject(func, "Branch", &format!("unknown block {}", target.0))
            })?;
            let clif_args: Vec<_> = args.iter()
                .map(|a| operand(env, *a, func).map(|(v, _)| v))
                .collect::<Result<Vec<_>, _>>()?;
            let block_args: Vec<_> = clif_args.iter()
                .map(|v| cranelift::prelude::codegen::ir::instructions::BlockArg::Value(*v))
                .collect();
            builder.ins().jump(dst, &block_args);
        }
        Some(MirTerminator::CondBranch { cond, then_block, then_args, else_block, else_args }) => {
            let (c, ct) = operand(env, *cond, func)?;
            if ct != MirType::Bool && ct != MirType::I64 {
                return Err(reject(func, "CondBranch", &format!("condition type {ct}, expected Bool")));
            }
            // Bool şeridi i64 0|1 taşır; brif i1 ister → != 0 çevir
            let c = if ct == MirType::I64 {
                let zero = builder.ins().iconst(I64, 0);
                builder.ins().icmp(IntCC::NotEqual, c, zero)
            } else {
                c
            };
            let t = *block_map.get(&then_block.0).ok_or_else(|| {
                reject(func, "CondBranch", &format!("unknown then block {}", then_block.0))
            })?;
            let e = *block_map.get(&else_block.0).ok_or_else(|| {
                reject(func, "CondBranch", &format!("unknown else block {}", else_block.0))
            })?;
            let ta: Vec<_> = then_args.iter()
                .map(|a| operand(env, *a, func).map(|(v, _)| cranelift::prelude::codegen::ir::instructions::BlockArg::Value(v)))
                .collect::<Result<Vec<_>, _>>()?;
            let ea: Vec<_> = else_args.iter()
                .map(|a| operand(env, *a, func).map(|(v, _)| cranelift::prelude::codegen::ir::instructions::BlockArg::Value(v)))
                .collect::<Result<Vec<_>, _>>()?;
            builder.ins().brif(c, t, &ta, e, &ea);
        }
        Some(MirTerminator::ReturnVoid) => {
            let cur_dz = builder.use_var(dz_var);
            let zero32 = builder.ins().iconst(I32, 0);
            let is_dz = builder.ins().icmp(IntCC::NotEqual, cur_dz, zero32);
            let z = builder.ins().iconst(I64, 0);
            emit_return_exit(builder, out_ptr, z, MirType::I64, func, None, Some(is_dz))?;
        }
        Some(MirTerminator::Return(ret_v)) => {
            let (val, ty) = operand(env, *ret_v, func)?;
            let cur_ov = builder.use_var(_ov_var);
            let cur_dz = builder.use_var(dz_var);
            let zero32 = builder.ins().iconst(I32, 0);
            let is_ov = builder.ins().icmp(IntCC::NotEqual, cur_ov, zero32);
            let is_dz = builder.ins().icmp(IntCC::NotEqual, cur_dz, zero32);
            emit_return_exit(builder, out_ptr, val, ty, func, Some(is_ov), Some(is_dz))?;
        }
        Some(MirTerminator::Trap(kind)) => {
            let _ = kind;
            let st = builder.ins().iconst(I32, JIT_EXIT_OVERFLOW as i64);
            let z = builder.ins().iconst(I64, 0);
            builder.ins().store(MemFlags::new(), st, out_ptr, OFF_STATUS);
            builder.ins().store(MemFlags::new(), z, out_ptr, OFF_VALUE);
            builder.ins().return_(&[]);
        }
        Some(MirTerminator::Unreachable) | None => {
            // No terminator: belki diğer bloklara düşecek — atla
        }
    }
    Ok(())
}
