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
                if target.ct_is_top_level() {
                    let sym = target.ct_intern(name);
                    target.ct_emit(Instruction::DeclGlobal { src: reg, sym: sym as u16 });
                }
            } else {
                let sym = target.ct_intern(name);
                target.ct_emit(Instruction::DeclGlobal { src: reg, sym: sym as u16 });
            }
            // ISSUE-1: record the local's type from its initializer.
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

        Stmt::Assignment {
            target: assign_target,
            value,
            span,
        } => {
            if let Expr::Identifier(name, _) = assign_target {
                // StrCatMut optimization: s = s + a + b + c → LoadLocal s,
                // StrCatMut a, StrCatMut b, StrCatMut c, StoreLocal s
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
                            return Err(compile_codes::generic(format!(
                                "Cannot assign to constant variable '{}'", name
                            )));
                        }
                        target.ct_emit(Instruction::Move { dst: local_reg, src: dst });
                        if target.ct_is_top_level() {
                            let sym = target.ct_intern(name);
                            target.ct_emit(Instruction::StoreGlobal { src: dst, sym: sym as u16 });
                        }
                    } else {
                        let sym = target.ct_intern(name);
                        target.ct_emit(Instruction::StoreGlobal { src: dst, sym: sym as u16 });
                    }
                } else {
                    let reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                        target, value, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
                    );
                    if let Some(local_reg) = target.ct_local_reg(name) {
                        if target.ct_is_const_local(name) {
                            return Err(compile_codes::generic(format!(
                                "Cannot assign to constant variable '{}'", name
                            )));
                        }
                        target.ct_emit(Instruction::Move { dst: local_reg, src: reg });
                        if target.ct_is_top_level() {
                            let sym = target.ct_intern(name);
                            target.ct_emit(Instruction::StoreGlobal { src: reg, sym: sym as u16 });
                        }
                    } else {
                        let sym = target.ct_intern(name);
                        target.ct_emit(Instruction::StoreGlobal { src: reg, sym: sym as u16 });
                    }
                }
                // ISSUE-1: propagate type through assignment.
                let resolve = |n: &str| target.ct_local_type(n);
                let ty = crate::compiler::expr::infer_type_with_locals(value, &resolve);
                target.ct_set_local_type(name, ty);
            } else if let Expr::Member {
                object, property, ..
            } = assign_target
            {
                // obj.field = value → compile obj, compile value, SetProperty(field)
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
            } else if let Expr::Index { object, index, .. } = assign_target {
                // obj[idx] = value
                //
                // Interpreter parity: evaluate `object` → `index` → `value`
                // in that order so any side effects in those sub-expressions
                // run left-to-right (preserved
                // `assignment_evaluates_left_to_right_index` test).
                if let Expr::Identifier(arr_name, _) = &**object {
                    // ISSUE-3: IndexAssignFast for slot-mapped local array + local index.
                    let fast_regs = if let Expr::Identifier(idx_name, _) = &**index {
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
                        target.ct_emit(Instruction::IndexAssign { obj: arr_reg, idx: idx_reg, val: val_reg });
                    } else {
                        // Register-based index assignment: arr[i] = val
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
                    // Complex but rooted: obj.arr[i] = val, a[b][c] = val, …
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
                    // No variable root (e.g. `f(x)[i] = val`).
                    // Evaluate all three sub-expressions in left-to-right
                    // order for side-effect parity, then assign.
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
        }

        Stmt::Expr(expr) => {
            // Postfix ++/-- in statement position: just emit IntAddI/IntSubI directly
            if let Expr::Unary { op: op @ (UnaryOp::PostIncrement | UnaryOp::PostDecrement), expr: inner, .. } = expr {
                if let Expr::Identifier(name, _) = inner.as_ref() {
                    if let Some(reg) = target.ct_local_reg(name) {
                        let imm: i16 = if matches!(op, UnaryOp::PostIncrement) { 1 } else { -1 };
                        target.ct_emit(Instruction::IntAddI { dst: reg, src: reg, imm });
                        return Ok(());
                    }
                }
            }
            let _reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                target, expr, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
            );
        }

        Stmt::Return { value, .. } => {
            // CROSS-1 (TCO): self-recursive tail call optimisation.
            //
            // Pattern: `return <fn_name>(args...)` where `<fn_name>`
            // equals the enclosing function's name, with NO trailing
            // operation (no `+ …`, no member access, no spread, no
            // cast).  Emit `TailCall(sym, argc)` in place of
            // `Call(sym, argc) + Return`: the VM rebinds params and
            // resets ip to 0, skipping the stack-frame setup entirely.
            //
            // Over-detection would trigger a runtime invariant panic
            // (callee != current_fn), so we stay conservative:
            //   - skip when current fn has a rest-parameter
            //   - skip on spread-arg calls (CallSpread path)
            //   - skip on method calls / super calls / generators
            //   - skip when callee name differs from current fn name
            // Anything outside these stays on the regular Call + Return
            // path and is 100% behaviour-preserving.
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
                                // Load function reference
                                target.ct_emit(Instruction::LoadGlobal { dst: func_reg, sym: name_sym.0 as u16 });
                                // Compile args to sequential registers
                                for (i, arg) in args.iter().enumerate() {
                                    let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, arg, &mut tregs);
                                    target.ct_emit(Instruction::Move { dst: first_arg + i as u8, src: r });
                                }
                                target.ct_emit(Instruction::TailCall { func_reg, first_arg_reg: first_arg, arg_count: argc });
                                // NOTE: no Return emitted — TailCall
                                // is self-terminating (jumps ip=0).
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

            // Check if then_branch always returns (ends with Return/Break/Continue/Throw)
            let then_ends = ends_with_return(then_branch);

            compile_stmt_shared(target, then_branch)?;
            if let Some(else_stmt) = else_branch {
                let else_ends = ends_with_return(else_stmt);

                // Only emit skip-else Jump if then_branch does NOT always return
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
                    // then_branch always returns — jump_to_else goes directly to else_start
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

            // Register VM fused compare+branch for simple Int comparisons.
            // Pattern: while (a <= b) where both a,b are slot-mapped locals.
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
                            // K1-1: locals are already in registers — use directly.
                            // Emit fused compare+branch (offset patched below)
                            let jump_to_end = target.ct_current_ip();
                            match op {
                                BinaryOp::Lt => target.ct_emit(Instruction::IntLtRRJumpIfFalse { src1: a_reg, src2: b_reg, offset: 0 }),
                                BinaryOp::Le => target.ct_emit(Instruction::IntLeRRJumpIfFalse { src1: a_reg, src2: b_reg, offset: 0 }),
                                BinaryOp::Gt => target.ct_emit(Instruction::IntLtRRJumpIfFalse { src1: b_reg, src2: a_reg, offset: 0 }),
                                BinaryOp::Ge => target.ct_emit(Instruction::IntLeRRJumpIfFalse { src1: b_reg, src2: a_reg, offset: 0 }),
                                _ => unreachable!(),
                            }
                            used_reg_cmp = true;
                            // Compile body and patch
                            let payload_idx = target.ct_add_loop_payload(loop_start as u32, 0);
                            target.ct_emit(Instruction::LoopBegin(payload_idx));
                            compile_stmt_shared(target, body)?;
                            target.ct_emit(Instruction::LoopEnd);
                            let back_jump_site = target.ct_current_ip();
                            target.ct_emit(Instruction::Jump(jump_off(back_jump_site, loop_start)));
                            let end = target.ct_current_ip();
                            // Patch the register fused instruction's offset
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

        // Issue #243: for-in loops
        Stmt::For { variable, iterable, body, .. } => {
            loops::compile_for_in(target, variable, iterable, body)?;
        }

        Stmt::ForCStyle { init, condition, update, body, .. } => {
            loops::compile_for_c_style(target, init, condition, update, body)?;
        }

        // Issue #243/#344: range-based for loop with direction detection
        Stmt::ForRange { start, stop, step, body, .. } => {
            loops::compile_for_range(target, start, stop, step, body)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}
