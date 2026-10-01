//! Member-call emission shared by the complex-expression arm and the
//! chain entry point (B6 zone reclamation).
//!
//! History: every `Call { callee: Member }` reached `compile_call`'s
//! fall-through, which compiled the WHOLE expression in a fresh 16-register
//! RegAlloc zone (`compile_expr_complex`).  For a chained receiver
//! (`"s".substring(0,8).substring(0,8)...`) each chain level therefore kept
//! a full zone alive while its receiver compiled — peak demand
//! `floor + 16 * depth`, exhausting the 14 available zones (MAX_BASE=224)
//! around depth 14 ("RegAlloc: out of register zones").  The emission
//! itself only needs receiver + temp-window args in ONE zone (Issue #7
//! design note in the original arm).  This module keeps that emission in
//! the CALLER's zone, so chains recurse through `compile_expr_to_reg`
//! without stacking zones.

use super::*;

/// Emit one `object.property(args)` call level into the given zone.
/// The result is relayed through register 255 (same protocol as the
/// original complex-expression arm).  The receiver register is freed
/// right after its single read — chain levels otherwise permanently
/// consume one zone register each.
pub(crate) fn emit_member_call_level(
    target: &mut impl CompileTarget,
    object: &Expr,
    property: &str,
    args: &[Expr],
    regs: &mut RegAlloc,
) {
    // FIX: stash receiver in a safe register so nested
    // MethodCall arguments cannot clobber reg255.
    // Issue #7: alıcı VE argümanlar inner'ın TEK bölgesini
    // paylaşır (yeni bölge YOK).  Alıcı canlılığı alloc last_use
    // ile korunur (argüman derlemesi onu geri kazanamaz).
    let receiver_reg = compile_expr_to_reg(target, object, regs);
    let argc = args.len() as u8;
    let first_arg = crate::compiler::regalloc::temp_reg_window(argc);
    for (i, arg) in args.iter().enumerate() {
        let r = compile_expr_to_reg(target, arg, regs);
        target.emit_move(first_arg + i as u8, r);
        // B6: the arg value is dead after the window move — free it so a
        // shared-zone chain does not accumulate one register per argument
        // per level (the old per-level fresh zone dropped them wholesale).
        regs.free_now(r);
    }
    // G4B: eliminate stash — use receiver_reg directly, preserve dst == arr for fusions
    if property == "push" && argc == 1 {
        target.ct_emit(Instruction::ArrayPush {
            dst: receiver_reg,
            arr: receiver_reg,
            val: first_arg,
        });
        target.emit_move(255, receiver_reg);
        regs.free_now(receiver_reg);
    } else if property == "pop" && argc == 0 {
        let is_typed_array = root_var_name(object)
            .map(|n| target.ct_local_type(&n) == crate::compiler::expr::ExprType::Array)
            .unwrap_or(false);
        if is_typed_array {
            target.ct_emit(Instruction::ArrayPop { dst: 255, obj: receiver_reg });
        } else {
            let prop_sym = target.ct_sym(property);
            let idx = target.ct_add_call_payload_with_builtin(
                prop_sym,
                argc,
                hudhudscript_bytecode::builtin_method::NONE,
            );
            target.emit_move(255, receiver_reg);
            target.ct_emit(Instruction::MethodCall {
                dst: 255,
                obj: 255,
                payload_idx: idx as u16,
                first_arg,
                arg_count: argc,
            });
        }
        regs.free_now(receiver_reg);
    } else if property == "indexOf" && argc == 1 {
        target.ct_emit(Instruction::StringIndexOf {
            dst: 255,
            haystack: receiver_reg,
            needle: first_arg,
        });
        regs.free_now(receiver_reg);
    } else if property == "contains" && argc == 1 {
        target.ct_emit(Instruction::StringContains {
            dst: 255,
            haystack: receiver_reg,
            needle: first_arg,
        });
        regs.free_now(receiver_reg);
    } else {
        // G4B: generic MethodCall — use receiver_reg directly, no stash
        let prop_sym = target.ct_sym(property);
        let builtin_idx = if let Expr::Identifier(obj_name, _) = object {
            hudhudscript_bytecode::builtin_method::resolve(obj_name, property)
        } else {
            u32::MAX
        };
        let idx = if builtin_idx != u32::MAX {
            target
                .ct_add_call_payload_with_builtin(prop_sym, argc, builtin_idx)
        } else {
            target.ct_add_call_payload(prop_sym, argc)
        };
        target.emit_move(255, receiver_reg);
        regs.free_now(receiver_reg);
        target.ct_emit(Instruction::MethodCall {
            dst: 255,
            obj: 255,
            payload_idx: idx as u16,
            first_arg,
            arg_count: argc,
        });
    }
    // Bug 4: this.call() implicitly references 'provider'
    if property == "call" && root_var_name(object).as_deref() == Some("this") {
        target.ct_track_reference("provider");
    }
}

/// Compile a `object.property(args)` call whose receiver is itself a
/// non-trivial expression (chain link), INSIDE the caller's zone.
/// Chain receivers recurse through `compile_expr_to_reg`'s dispatch, so
/// every level reuses the same zone instead of stacking a fresh
/// 16-register zone per link (B6).  Returns the register holding the
/// result.
pub(crate) fn compile_chained_member_call(
    target: &mut impl CompileTarget,
    object: &Expr,
    property: &str,
    args: &[Expr],
    regs: &mut RegAlloc,
    ip: usize,
    last_use: usize,
) -> u8 {
    emit_member_call_level(target, object, property, args, regs);
    // Result relayed via 255 → caller-zone register (same protocol as the
    // generic compile_reg fallback arm).
    let dst = regs.alloc(ip, last_use).expect("out of registers");
    target.emit_move(dst, 255);
    dst
}
