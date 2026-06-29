use super::*;
use hudhudscript_bytecode::Instruction;

pub(super) fn compile_assignment(
    target: &mut impl CompileTarget,
    assign_target: &hudhudscript_ast::Expr,
    value: &hudhudscript_ast::Expr,
    span: &hudhudscript_ast::Span,
) -> CompileResult<()> {
    if let Expr::Identifier(ref name, _) = assign_target {
        // P5d: track global Math reassignment
        if name == "Math" {
            target.ct_set_math_reassigned();
        }
        // ISSUE-2e-1: a top-level symbol assigned inside a function/closure body
        // must be marked shared even if it is never read, because the runtime
        // fallback for non-local assignment is StoreGlobal.
let chain_opt = if target.ct_local_type(name) == crate::compiler::expr::ExprType::Str {
    decompose_add_chain(value, name)
} else {
    None
};
if let Some(chain) = chain_opt {
    let mut regs = crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones");
    let dst = regs.alloc(target.ct_current_ip(), target.ct_current_ip() + 255).expect("out of registers");
    if let Some(local_reg) = target.ct_local_reg(name) {
        target.ct_emit(Instruction::Move { dst, src: local_reg });
    } else {
        let sym = target.ct_intern(name);
        target.ct_emit(Instruction::LoadGlobal { dst, sym: sym as u16 });
    }
    for expr in chain {
        let src = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, expr, &mut regs);
        target.ct_emit(Instruction::StrCatMut { dst, src2: src });
        regs.free_now(src);
    }
    if let Some(local_reg) = target.ct_local_reg(name) {
        if target.ct_is_const_local(name) {
            return Err(compile_codes::generic(format!("Cannot assign to constant variable '{}'", name)));
        }
        target.ct_emit(Instruction::Move { dst: local_reg, src: dst });
        if target.ct_is_top_level() && target.ct_is_shared_top_level(name) {
            let sym = target.ct_intern(name);
            target.ct_emit(Instruction::StoreGlobal { src: dst, sym: sym as u16 });
        }
    } else {
        let sym = target.ct_intern(name);
        target.ct_emit(Instruction::StoreGlobal { src: dst, sym: sym as u16 });
    }
} else {
    // B3: NumMulAddAssign fusion — dest = dest * mul_expr + add_expr
    let fma_opt = if target.ct_local_type(name) != crate::compiler::expr::ExprType::Str {
        crate::compiler::stmt_shared::assignment_fusions::try_fma_pattern(value, name)
    } else {
        None
    };
    if let Some((mul_expr, add_expr)) = fma_opt {
        if let Some(local_reg) = target.ct_local_reg(name) {
            if target.ct_is_const_local(name) {
                return Err(compile_codes::generic(format!("Cannot assign to constant variable '{}'", name)));
            }
            let mut regs = crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones");
            let mul_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, mul_expr, &mut regs);
            let add_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, add_expr, &mut regs);
            target.ct_emit(Instruction::NumMulAddAssign { dst: local_reg, mul: mul_reg, add: add_reg });
            if target.ct_is_top_level() && target.ct_is_shared_top_level(name) {
                let sym = target.ct_intern(name);
                target.ct_emit(Instruction::StoreGlobal { src: local_reg, sym: sym as u16 });
            }
            target.ct_set_local_type(name, crate::compiler::expr::infer_type_with_locals(value, &|n: &str| target.ct_local_type(n)));
            return Ok(());
        }
    }
    // B4
    if let Some(imm) = crate::compiler::stmt_shared::assignment_fusions::try_self_sub_int(value, name) {
        if let Some(local_reg) = target.ct_local_reg(name) {
            if target.ct_is_const_local(name) {
                return Err(compile_codes::generic(format!("Cannot assign to constant variable '{}'", name)));
            }
            target.ct_emit(Instruction::IntSubI { dst: local_reg, src: local_reg, imm });
            if target.ct_is_top_level() && target.ct_is_shared_top_level(name) {
                let sym = target.ct_intern(name);
                target.ct_emit(Instruction::StoreGlobal { src: local_reg, sym: sym as u16 });
            }
            return Ok(());
        }
    }
    // P2-B1: self-add-int — i = i + 1 → IntAddI dst=src=i_reg, no Move
    // Only for numeric locals; string concat must use the generic path.
    if target.ct_local_type(name) != crate::compiler::expr::ExprType::Str {
        if let Some(imm) = crate::compiler::stmt_shared::assignment_fusions::try_self_add_int(value, name) {
            if let Some(local_reg) = target.ct_local_reg(name) {
                if target.ct_is_const_local(name) {
                    return Err(compile_codes::generic(format!("Cannot assign to constant variable '{}'", name)));
                }
                target.ct_emit(Instruction::IntAddI { dst: local_reg, src: local_reg, imm });
                if target.ct_is_top_level() && target.ct_is_shared_top_level(name) {
                    let sym = target.ct_intern(name);
                    target.ct_emit(Instruction::StoreGlobal { src: local_reg, sym: sym as u16 });
                }
                return Ok(());
            }
        }
    }
    let reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
        target, value, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
    );
    if let Some(local_reg) = target.ct_local_reg(name) {
        if target.ct_is_const_local(name) {
            return Err(compile_codes::generic(format!("Cannot assign to constant variable '{}'", name)));
        }
        target.ct_emit(Instruction::Move { dst: local_reg, src: reg });
        if target.ct_is_top_level() && target.ct_is_shared_top_level(name) {
            let sym = target.ct_intern(name);
            target.ct_emit(Instruction::StoreGlobal { src: reg, sym: sym as u16 });
        }
    } else {
        let sym = target.ct_intern(name);
        target.ct_emit(Instruction::StoreGlobal { src: reg, sym: sym as u16 });
    }
}
let resolve = |n: &str| target.ct_local_type(n);
let ty = crate::compiler::expr::infer_type_with_locals(value, &resolve);
target.ct_set_local_type(name, ty);
} else if let hudhudscript_ast::Expr::Member {
object, property, ..
} = assign_target
{
let mut regs = crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones");
let obj_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
    target, object, &mut regs,
);
let val_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
    target, value, &mut regs,
);
let prop_sym = target.ct_sym(property);
let dst = crate::compiler::regalloc::temp_reg();
target.ct_emit(Instruction::SetProperty { dst, obj: obj_reg, val: val_reg, prop_sym: prop_sym.0 as u16 });
if let Some(root_name) = root_var_name(object) {
    if let Some(local_reg) = target.ct_local_reg(&root_name) {
        target.ct_emit(Instruction::Move { dst: local_reg, src: dst });
    } else {
        let sym = target.ct_intern(&root_name);
        target.ct_emit(Instruction::StoreGlobal { src: dst, sym: sym as u16 });
    }
}
} else if let hudhudscript_ast::Expr::Index { object, index, .. } = assign_target {
if let Expr::Identifier(ref arr_name, _) = &**object {
    let fast_regs = if let Expr::Identifier(ref idx_name, _) = &**index {
        match (target.ct_local_reg(arr_name), target.ct_local_reg(idx_name)) {
            (Some(a), Some(i)) => Some((a, i)),
            _ => None,
        }
    } else {
        None
    };
    if let Some((arr_reg, idx_reg)) = fast_regs {
        let mut regs = crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones");
        let val_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
            target, value, &mut regs,
        );
        let arr_ty = target.ct_local_type(arr_name);
        if arr_ty == crate::compiler::expr::ExprType::Array {
            target.ct_emit(Instruction::IndexAssignArray { obj: arr_reg, idx: idx_reg, val: val_reg });
        } else {
            target.ct_emit(Instruction::IndexAssign { obj: arr_reg, idx: idx_reg, val: val_reg });
        }
    } else {
        let mut regs = crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones");
        let arr_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
            target, object, &mut regs,
        );
        let idx_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
            target, index, &mut regs,
        );
        let val_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
            target, value, &mut regs,
        );
        target.ct_emit(Instruction::IndexAssign { obj: arr_reg, idx: idx_reg, val: val_reg });
        if let Some(local_reg) = target.ct_local_reg(arr_name) {
            target.ct_emit(Instruction::Move { dst: local_reg, src: arr_reg });
        } else {
            let sym = target.ct_intern(arr_name);
            target.ct_emit(Instruction::StoreGlobal { src: arr_reg, sym: sym as u16 });
        }
    }
} else if let Some(root) = root_var_name(object) {
    let mut regs = crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones");
    let obj_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
        target, object, &mut regs,
    );
    let idx_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
        target, index, &mut regs,
    );
    let val_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
        target, value, &mut regs,
    );
    target.ct_emit(Instruction::IndexAssign { obj: obj_reg, idx: idx_reg, val: val_reg });
    if let Some(local_reg) = target.ct_local_reg(&root) {
        target.ct_emit(Instruction::Move { dst: local_reg, src: obj_reg });
    } else {
        let sym = target.ct_intern(&root);
        target.ct_emit(Instruction::StoreGlobal { src: obj_reg, sym: sym as u16 });
    }
} else {
    let mut regs = crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones");
    let obj_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
        target, object, &mut regs,
    );
    let idx_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
        target, index, &mut regs,
    );
    let val_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
        target, value, &mut regs,
    );
    target.ct_emit(Instruction::IndexAssign { obj: obj_reg, idx: idx_reg, val: val_reg });
}
} else {
return Err(compile_codes::unsupported_feature_at(
    "Only simple, member, and index assignment supported".to_string(),
    span_pos(span),
));
}
    Ok(())
}
