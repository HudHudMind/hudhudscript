use super::*;
use hudhudscript_ast::UnaryOp;
use hudhudscript_ast::Expr;
use hudhudscript_bytecode::Instruction;
pub(super) fn compile_stmt_part1(
    target: &mut impl CompileTarget,
    stmt: &Stmt,
) -> CompileResult<()> {
    match stmt {
        Stmt::Let { name, value, .. } => {
            let reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                target, value, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
            );
            target.ct_declare_local(name, false)?;
            if let Some(local_reg) = target.ct_local_reg(name) {
                target.ct_emit(Instruction::Move { dst: local_reg, src: reg });
                if target.ct_is_top_level() && target.ct_is_shared_top_level(name) {
                    let sym = target.ct_intern(name);
                    target.ct_emit(Instruction::DeclGlobal { src: reg, sym: sym as u16 });
                }
            } else {
                let sym = target.ct_intern(name);
                target.ct_emit(Instruction::DeclGlobal { src: reg, sym: sym as u16 });
            }
            let resolve = |n: &str| target.ct_local_type(n);
            let ty = crate::compiler::expr::infer_type_with_locals(value, &resolve);
            target.ct_set_local_type(name, ty);
        }
        Stmt::Const { name, value, .. } => {
            let reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                target, value, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
            );
            target.ct_declare_local(name, true)?;
            if let Some(local_reg) = target.ct_local_reg(name) {
                target.ct_emit(Instruction::Move { dst: local_reg, src: reg });
            }
            let sym = target.ct_intern(name);
            target.ct_emit(Instruction::StoreConst { src: reg, sym: sym as u16 });
        }
        Stmt::Assignment { target: assign_target, value, span, } => {
            crate::compiler::stmt_shared::assignment::compile_assignment(target, assign_target, value, span)?;
        }
        Stmt::Expr(expr) => {
            if let Expr::Unary { op: op @ (UnaryOp::PostIncrement | UnaryOp::PostDecrement), expr: inner, .. } = expr {
                if let Expr::Identifier(name, _) = inner.as_ref() {
                    if let Some(reg) = target.ct_local_reg(name) {
                        let imm: i16 = if matches!(op, UnaryOp::PostIncrement) { 1 } else { -1 };
                        target.ct_emit(Instruction::IntAddI { dst: reg, src: reg, imm });
                        if target.ct_is_top_level() && target.ct_is_shared_top_level(name) {
                            let sym = target.ct_intern(name);
                            target.ct_emit(Instruction::StoreGlobal { src: reg, sym: sym as u16 });
                        }
                        return Ok(());
                    }
                }
            }
            let _reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                target, expr, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
            );
        }
        Stmt::Return { value, .. } => {
            if let Some(Expr::Call { callee, args, .. }) = value.as_ref() {
                if let Some(fn_name) = target.ct_current_fn_name() {
                    if !target.ct_current_has_rest()
                        && !args.iter().any(|a| matches!(a, Expr::Spread { .. }))
                    {
                        if let Expr::Identifier(called, _) = &**callee {
                            if called == fn_name && !target.ct_is_known_generator(called) {
                                let argc = args.len() as u8;
                                let name_sym = target.ct_sym(called);
                                let mut tregs = RegAlloc::new_with_base(target.ct_next_local_reg())?;
                                let first_arg = crate::compiler::regalloc::temp_reg();
                                let func_reg = crate::compiler::regalloc::temp_reg();
                                target.ct_emit(Instruction::LoadGlobal { dst: func_reg, sym: name_sym.0 as u16 });
                                for (i, arg) in args.iter().enumerate() {
                                    let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, arg, &mut tregs);
                                    target.ct_emit(Instruction::Move { dst: first_arg + i as u8, src: r });
                                }
                                target.ct_emit(Instruction::TailCall { func_reg, first_arg_reg: first_arg, arg_count: argc });
                                return Ok(());
                            }
                        }
                    }
                }
            }
            if let Some(expr) = value {
                let reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                    target, expr, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
                );
                target.ct_emit(Instruction::Return { src: reg });
            } else {
                let idx = target.ct_emit_const(Value16::null());
                let tr = crate::compiler::regalloc::temp_reg();
                target.ct_emit(Instruction::LoadConst { dst: tr, const_idx: idx as u16 });
                target.ct_emit(Instruction::Return { src: tr });
            }
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            let cr = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                target, condition, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
            );
            let jump_to_else = target.ct_current_ip();
            target.ct_emit(Instruction::JumpIfFalse { src: cr, offset: 0 });
            let then_ends = ends_with_return(then_branch);
            compile_stmt_shared(target, then_branch)?;
            if let Some(else_stmt) = else_branch {
                let else_ends = ends_with_return(else_stmt);
                if !then_ends {
                    let jump_over_else = target.ct_current_ip();
                    target.ct_emit(Instruction::Jump(0));
                    let else_start = target.ct_current_ip();
                    target.ct_patch(
                        jump_to_else,
                        Instruction::JumpIfFalse { src: cr, offset: jump_off(jump_to_else, else_start) as i16 },
                    );
                    compile_stmt_shared(target, else_stmt)?;
                    let end = target.ct_current_ip();
                    target.ct_patch(
                        jump_over_else,
                        Instruction::Jump(jump_off(jump_over_else, end)),
                    );
                } else {
                    let else_start = target.ct_current_ip();
                    target.ct_patch(
                        jump_to_else,
                        Instruction::JumpIfFalse { src: cr, offset: jump_off(jump_to_else, else_start) as i16 },
                    );
                    compile_stmt_shared(target, else_stmt)?;
                }
            } else {
                let end = target.ct_current_ip();
                target.ct_patch(
                    jump_to_else,
                    Instruction::JumpIfFalse { src: cr, offset: jump_off(jump_to_else, end) as i16 },
                );
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            let loop_start = target.ct_current_ip();
            let mut used_reg_cmp = false;
            if let Expr::Binary { left, op, right, .. } = condition {
                if matches!(op, BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge) {
                    let get_reg = |e: &Expr| -> Option<u8> {
                        if let Expr::Identifier(name, _) = e {
                            target.ct_local_reg(name)
                        } else { None }
                    };
                    if let (Some(a_reg), Some(b_reg)) = (get_reg(left), get_reg(right)) {
                        let resolve = |n: &str| target.ct_local_type(n);
                        let l_ty = crate::compiler::expr::infer_type_with_locals(left, &resolve);
                        let r_ty = crate::compiler::expr::infer_type_with_locals(right, &resolve);
                        let both_int = (l_ty == crate::compiler::expr::ExprType::Int || l_ty == crate::compiler::expr::ExprType::Unknown)
                            && (r_ty == crate::compiler::expr::ExprType::Int || r_ty == crate::compiler::expr::ExprType::Unknown);
                        if both_int {
                            let jump_to_end = target.ct_current_ip();
                            match op {
                                BinaryOp::Lt => target.ct_emit(Instruction::IntLtRRJumpIfFalse { src1: a_reg, src2: b_reg, offset: 0 }),
                                BinaryOp::Le => target.ct_emit(Instruction::IntLeRRJumpIfFalse { src1: a_reg, src2: b_reg, offset: 0 }),
                                BinaryOp::Gt => target.ct_emit(Instruction::IntLtRRJumpIfFalse { src1: b_reg, src2: a_reg, offset: 0 }),
                                BinaryOp::Ge => target.ct_emit(Instruction::IntLeRRJumpIfFalse { src1: b_reg, src2: a_reg, offset: 0 }),
                                _ => unreachable!(),
                            }
                            used_reg_cmp = true;
                            let payload_idx = target.ct_add_loop_payload(loop_start as u32, 0);
                            target.ct_emit(Instruction::LoopBegin(payload_idx));
                            compile_stmt_shared(target, body)?;
                            target.ct_emit(Instruction::LoopEnd);
                            let back_jump_site = target.ct_current_ip();
                            target.ct_emit(Instruction::Jump(jump_off(back_jump_site, loop_start)));
                            let end = target.ct_current_ip();
                            let offset = jump_off(jump_to_end, end);
                            let patch_instr = match op {
                                BinaryOp::Lt => Instruction::IntLtRRJumpIfFalse { src1: a_reg, src2: b_reg, offset: offset as i16 },
                                BinaryOp::Le => Instruction::IntLeRRJumpIfFalse { src1: a_reg, src2: b_reg, offset: offset as i16 },
                                BinaryOp::Gt => Instruction::IntLtRRJumpIfFalse { src1: b_reg, src2: a_reg, offset: offset as i16 },
                                BinaryOp::Ge => Instruction::IntLeRRJumpIfFalse { src1: b_reg, src2: a_reg, offset: offset as i16 },
                                _ => unreachable!(),
                            };
                            target.ct_patch(jump_to_end, patch_instr);
                            target.ct_patch_loop_payload_end(payload_idx, end as u32);
                        }
                    }
                }
            }
            if !used_reg_cmp {
            let cr = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                target, condition, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
            );
            let jump_to_end = target.ct_current_ip();
            target.ct_emit(Instruction::JumpIfFalse { src: cr, offset: 0 });
            let payload_idx = target.ct_add_loop_payload(loop_start as u32, 0);
            target.ct_emit(Instruction::LoopBegin(payload_idx));
            compile_stmt_shared(target, body)?;
            target.ct_emit(Instruction::LoopEnd);
            let back_jump_site = target.ct_current_ip();
            target.ct_emit(Instruction::Jump(jump_off(back_jump_site, loop_start)));
            let end = target.ct_current_ip();
            target.ct_patch(
                jump_to_end,
                Instruction::JumpIfFalse { src: cr, offset: jump_off(jump_to_end, end) as i16 },
            );
            target.ct_patch_loop_payload_end(payload_idx, end as u32);
            }
        }
        Stmt::For { variable, iterable, body, .. } => {
            loops::compile_for_in(target, variable, iterable, body)?;
        }
        Stmt::ForCStyle { init, condition, update, body, .. } => {
            loops::compile_for_c_style(target, init, condition, update, body)?;
        }
        Stmt::ForRange { start, stop, step, body, .. } => {
            loops::compile_for_range(target, start, stop, step, body)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}
