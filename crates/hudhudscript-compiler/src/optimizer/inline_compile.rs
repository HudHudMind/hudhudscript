// P3: Compiler-side function inlining helper.
// Called during compilation when the compiler encounters a Call to a known
// pure function that's already been compiled. Imports callee constants into
// the caller pool, remaps registers with checked arithmetic, and emits
// inlined instructions or returns false for normal Call fallback.

use super::inline_remap::{can_remap_regs, remap_single_instr};
use crate::compiler::CompileTarget;
use hudhudscript_bytecode::{FunctionChunk, Instruction};

/// Try to inline a call to `callee` using `target` for constant import and
/// instruction emission. Returns `true` if inlining succeeded; `false` means
/// the caller should emit a normal `Call` instruction.
pub(crate) fn try_inline_call(
    target: &mut dyn CompileTarget,
    callee: &FunctionChunk,
    first_arg: u8,
    arg_count: u8,
    dst: u8,
) -> bool {
    // ---- Phase 1: import callee constants into caller pool ---------------
    // B8: all three constant types need remapping — callee and caller may
    // have different pools during function compilation.
    let const_count = callee.constants.len();
    let mut const_remap: Vec<u16> = Vec::with_capacity(const_count);
    if const_count > 0 {
        for val in &callee.constants {
            let new_idx = target.ct_emit_const(*val);
            const_remap.push(new_idx as u16);
        }
    }
    // B8: remap int and num constants using global snapshots
    let ints = target.ct_int_constants().to_vec();
    let nums = target.ct_numeric_constants().to_vec();
    let mut int_remap: Vec<u16> = Vec::with_capacity(ints.len());
    for v in &ints {
        int_remap.push(target.ct_emit_int_const(*v) as u16);
    }
    let mut num_remap: Vec<u16> = Vec::with_capacity(nums.len());
    for bits in &nums {
        num_remap.push(target.ct_emit_num_const(f64::from_bits(*bits)) as u16);
    }

    // ---- Phase 2: plan (all checks, no emission) ------------------------
    if let Some(plan) = try_inline_plan(
        callee,
        first_arg,
        arg_count,
        dst,
        &const_remap,
        &int_remap,
        &num_remap,
    ) {
        // ---- Phase 3: emit atomically -----------------------------------
        for instr in plan {
            target.ct_emit(instr);
        }
        true
    } else {
        false
    }
}

/// Pure planning function: validate eligibility and build remapped
/// instruction list without modifying any target state.  Exposed as
/// `pub(crate)` so integration tests can exercise edge cases directly.
///
/// `const_remap[i]` is the caller-side index for `callee.constants[i]`.
pub fn try_inline_plan(
    callee: &FunctionChunk,
    first_arg: u8,
    arg_count: u8,
    dst: u8,
    const_remap: &[u16],
    int_remap: &[u16],
    num_remap: &[u16],
) -> Option<Vec<Instruction>> {
    let body = &callee.instructions;

    // Size limit: 2..15 instructions
    if body.is_empty() || body.len() > 15 {
        return None;
    }

    // Purity: reject loops
    if body
        .iter()
        .any(|ci| matches!(ci, Instruction::LoopBegin(_)))
    {
        return None;
    }
    // Reject fused returns (inliner only handles plain Return)
    if body.iter().any(|ci| {
        matches!(
            ci,
            Instruction::IntAddReturn { .. }
                | Instruction::IntSubReturn { .. }
                | Instruction::IntMulReturn { .. }
                | Instruction::IntDivReturn { .. }
                | Instruction::IntCmpIReturn { .. }
                | Instruction::ReturnConst { .. }
        )
    }) {
        return None;
    }
    // Reject side effects
    if body.iter().any(|ci| {
        matches!(
            ci,
            Instruction::StoreGlobal { .. }
                | Instruction::DeclGlobal { .. }
                | Instruction::MethodCall { .. }
                | Instruction::SuperCall { .. }
                | Instruction::Call { .. }
                | Instruction::IndexAssign { .. }
                | Instruction::IndexAssignArray { .. }
                | Instruction::SetProperty { .. }
                | Instruction::Yield { .. }
                | Instruction::Await { .. }
                | Instruction::Spawn { .. }
                | Instruction::Throw { .. }
                | Instruction::LoopBegin(_)
                | Instruction::TryBegin(_)
        )
    }) {
        return None;
    }

    // Reject jumps/conditionals inside the body
    if body.iter().any(|ci| {
        matches!(
            ci,
            Instruction::Jump(..)
                | Instruction::JumpIfFalse { .. }
                | Instruction::JumpIfTrue { .. }
                | Instruction::Break
        )
    }) {
        return None;
    }

    // Last instruction must be plain Return
    let ret_src = match body.last() {
        Some(Instruction::Return { src }) => *src,
        _ => return None,
    };

    // ---- Register remap with checked arithmetic --------------------------
    // params 0..arg_count → first_arg..first_arg+arg_count
    // other callee regs → base + (reg - arg_count)  where base=first_arg+arg_count
    // 255 stays 255 (special VM register, used by compile_complex path)
    let base: u8 = first_arg.checked_add(arg_count)?;

    let map_reg = |r: u8| -> Option<u8> {
        if r == 255 {
            Some(255)
        } else if r < arg_count {
            Some(first_arg.checked_add(r)?)
        } else {
            let offset = r.checked_sub(arg_count)?;
            Some(base.checked_add(offset)?)
        }
    };

    // Pre-validate that all register operands can be remapped without wrapping
    for ci in body.iter() {
        if !can_remap_regs(ci, &map_reg) {
            return None;
        }
    }

    // ---- Build remapped instructions ------------------------------------
    let mut out = Vec::with_capacity(body.len());
    for (i, ci) in body.iter().enumerate() {
        if i == body.len() - 1 {
            let mapped_src = map_reg(ret_src)?;
            if dst != mapped_src {
                out.push(Instruction::Move {
                    dst,
                    src: mapped_src,
                });
            }
        } else {
            out.push(remap_single_instr(
                ci,
                &map_reg,
                const_remap,
                int_remap,
                num_remap,
            )?);
        }
    }

    Some(out)
}

/// Inline edilecek gövdenin yazmaç ihtiyacı (argümanlar + geçiciler, 255 hariç).
/// `try_inline_call(..., first_arg = W, ...)` çıktısındaki TÜM yazmaçlar
/// [W, W + span) aralığına düşer. None = gövdede remap edilemeyen operand var.
///
/// BULGU6 kök nedeni: 1-argümanlı inline çağrıda first_arg, argümanın KENDİ
/// yazmacı yapılıyordu; inline gövdenin geçicileri first_arg+argc'den itibaren
/// çağıranın canlı yerellerinin üstüne yazıyordu (MakeObject dst'sini ezip
/// ObjLitSet'i panikletiyordu). Çözüm: argüman + geçiciler izole temp penceresi.
pub fn inline_window_need(callee: &FunctionChunk) -> Option<u8> {
    let max = std::cell::Cell::new(0u8);
    let rec = |r: u8| -> Option<u8> {
        if r != 255 && r > max.get() {
            max.set(r);
        }
        Some(r)
    };
    for ci in callee.instructions.iter() {
        if !can_remap_regs(ci, &rec) {
            return None;
        }
    }
    Some(max.get().saturating_add(1))
}
