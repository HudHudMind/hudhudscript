use super::*;
use hudhudscript_ast::Expr;
use hudhudscript_ast::UnaryOp;
use hudhudscript_bytecode::Instruction;

/// C6: detect `var == "X"` or `"X" == var` where X is a single-character string.
fn extract_char_eq(condition: &Expr) -> Option<(&str, u8)> {
    if let Expr::Binary {
        op: hudhudscript_ast::BinaryOp::Eq,
        left,
        right,
        ..
    } = condition
    {
        match (left.as_ref(), right.as_ref()) {
            (Expr::Identifier(name, _), Expr::Literal(hudhudscript_ast::Literal::String(s), _))
                if s.len() == 1 =>
            {
                Some((name.as_str(), s.as_bytes()[0]))
            }
            (Expr::Literal(hudhudscript_ast::Literal::String(s), _), Expr::Identifier(name, _))
                if s.len() == 1 =>
            {
                Some((name.as_str(), s.as_bytes()[0]))
            }
            _ => None,
        }
    } else {
        None
    }
}

/// C6: collect an if-else-if chain of single-char equality into (var, branches, default).
fn collect_char_dispatch<'a>(
    condition: &'a Expr,
    then_branch: &'a Stmt,
    else_branch: Option<&'a Stmt>,
    branches: &mut Vec<(u8, &'a Stmt)>,
    default: &mut Option<&'a Stmt>,
    var_name: &mut Option<String>,
) -> bool {
    let (vn, byte) = match extract_char_eq(condition) {
        Some(x) => x,
        None => return false,
    };
    if let Some(ref expected) = *var_name {
        if expected != vn {
            return false;
        }
    } else {
        *var_name = Some(vn.to_string());
    }
    branches.push((byte, then_branch));
    match else_branch {
        Some(Stmt::If {
            condition,
            then_branch,
            else_branch,
            ..
        }) => collect_char_dispatch(
            condition,
            then_branch,
            else_branch.as_deref(),
            branches,
            default,
            var_name,
        ),
        Some(other) => {
            *default = Some(other);
            true
        }
        None => true,
    }
}

/// C6: compile the collected chain as a byte-indexed CharDispatch.
fn compile_char_dispatch<'a, T: CompileTarget>(
    target: &mut T,
    span: &hudhudscript_ast::Span,
    var_name: &str,
    branches: Vec<(u8, &'a Stmt)>,
    default: Option<&'a Stmt>,
) -> CompileResult<()> {
    let src_reg = match target.ct_local_reg(var_name) {
        Some(r) => r,
        None => return Ok(()), // fallback: caller will compile normally
    };
    target.ct_mark_stmt_pos(span);
    let dispatch_ip = target.ct_current_ip();
    let table_idx = target.ct_add_char_dispatch_table(vec![0i16; 256]);
    target.ct_emit(Instruction::CharDispatch {
        src: src_reg,
        table_idx,
    });

    let mut table = vec![0i16; 256];
    let mut end_jumps: Vec<usize> = Vec::new();
    let mut branch_start_ips: Vec<(u8, usize)> = Vec::new();

    for (byte, body) in &branches {
        let body_start = target.ct_current_ip();
        branch_start_ips.push((*byte, body_start));
        compile_stmt_shared(target, body)?;
        if !ends_with_return(body) {
            let jump_ip = target.ct_current_ip();
            target.ct_emit(Instruction::Jump(0));
            end_jumps.push(jump_ip);
        }
    }

    let default_start_ip = target.ct_current_ip();
    if let Some(default_body) = default {
        compile_stmt_shared(target, default_body)?;
    }
    let end_ip = target.ct_current_ip();

    // Patch end jumps (Jump is ip+offset, no +1 adjustment).
    for jump_ip in end_jumps {
        let off = (end_ip as i64).wrapping_sub(jump_ip as i64) as i32;
        target.ct_patch(jump_ip, Instruction::Jump(off));
    }

    // Build the dispatch table.
    let default_offset = (default_start_ip as i64)
        .wrapping_sub(dispatch_ip as i64)
        .wrapping_sub(1) as i16;
    table.fill(default_offset);
    for (byte, body_start) in branch_start_ips {
        let off = (body_start as i64)
            .wrapping_sub(dispatch_ip as i64)
            .wrapping_sub(1) as i16;
        table[byte as usize] = off;
    }
    target.ct_replace_char_dispatch_table(table_idx, table);

    Ok(())
}

pub(super) fn compile_stmt_part1(
    target: &mut impl CompileTarget,
    stmt: &Stmt,
) -> CompileResult<()> {
    match stmt {
        Stmt::Let { name, value, .. } => {
            // G12: f-loop bağlamında aday let → f-domain (register yazılmaz).
            if crate::compiler::floop::compile_let(target, name, value)? {
                return Ok(());
            }
            let reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                target,
                value,
                &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg())
                    .expect("out of register zones"),
            );
            target.ct_declare_local(name, false)?;
            if let Some(local_reg) = target.ct_local_reg(name) {
                target.emit_move(local_reg, reg);
                if target.ct_is_top_level() && target.ct_is_shared_top_level(name) {
                    let sym = target.ct_intern(name);
                    target.ct_emit(Instruction::DeclGlobal {
                        src: reg,
                        sym: sym as u16,
                    });
                }
            } else {
                let sym = target.ct_intern(name);
                target.ct_emit(Instruction::DeclGlobal {
                    src: reg,
                    sym: sym as u16,
                });
            }
            let resolve = |n: &str| target.ct_local_type(n);
            let ty = crate::compiler::expr::infer_type_with_locals(value, &resolve);
            target.ct_set_local_type(name, ty);
        }
        Stmt::Const { name, value, .. } => {
            let reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                target,
                value,
                &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg())
                    .expect("out of register zones"),
            );
            target.ct_declare_local(name, true)?;
            if let Some(local_reg) = target.ct_local_reg(name) {
                target.emit_move(local_reg, reg);
                if target.ct_is_top_level() && target.ct_is_shared_top_level(name) {
                    let sym = target.ct_intern(name);
                    target.ct_emit(Instruction::StoreConst {
                        src: reg,
                        sym: sym as u16,
                    });
                }
            } else {
                let sym = target.ct_intern(name);
                target.ct_emit(Instruction::StoreConst {
                    src: reg,
                    sym: sym as u16,
                });
            }
        }
        Stmt::Assignment {
            target: assign_target,
            value,
            span,
        } => {
            crate::compiler::stmt_shared::assignment::compile_assignment(
                target,
                assign_target,
                value,
                span,
            )?;
        }
        Stmt::Expr(expr) => {
            if let Expr::Unary {
                op: op @ (UnaryOp::PostIncrement | UnaryOp::PostDecrement),
                expr: inner,
                ..
            } = expr
            {
                if let Expr::Identifier(name, _) = inner.as_ref() {
                    if let Some(reg) = target.ct_local_reg(name) {
                        let imm: i16 = if matches!(op, UnaryOp::PostIncrement) {
                            1
                        } else {
                            -1
                        };
                        target.ct_emit(Instruction::IntAddI {
                            dst: reg,
                            src: reg,
                            imm,
                        });
                        if target.ct_is_top_level() && target.ct_is_shared_top_level(name) {
                            let sym = target.ct_intern(name);
                            target.ct_emit(Instruction::StoreGlobal {
                                src: reg,
                                sym: sym as u16,
                            });
                        }
                        return Ok(());
                    }
                }
            }
            let _reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                target,
                expr,
                &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg())
                    .expect("out of register zones"),
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
                                let mut tregs =
                                    RegAlloc::new_with_base(target.ct_next_local_reg())?;
                                let first_arg = crate::compiler::regalloc::temp_reg();
                                let func_reg = crate::compiler::regalloc::temp_reg();
                                target.ct_emit(Instruction::LoadGlobal {
                                    dst: func_reg,
                                    sym: name_sym.0 as u16,
                                });
                                for (i, arg) in args.iter().enumerate() {
                                    let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                                        target, arg, &mut tregs,
                                    );
                                    target.emit_move(first_arg + i as u8, r);
                                }
                                target.ct_emit(Instruction::TailCall {
                                    func_reg,
                                    first_arg_reg: first_arg,
                                    arg_count: argc,
                                });
                                return Ok(());
                            }
                        }
                    }
                }
            }
            if let Some(expr) = value {
                let reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                    target,
                    expr,
                    &mut crate::compiler::regalloc::RegAlloc::new_with_base(
                        target.ct_next_local_reg(),
                    )
                    .expect("out of register zones"),
                );
                target.ct_emit(Instruction::Return { src: reg });
            } else {
                let idx = target.ct_emit_const(Value16::null());
                let tr = crate::compiler::regalloc::temp_reg();
                target.ct_emit(Instruction::LoadConst {
                    dst: tr,
                    const_idx: idx as u16,
                });
                target.ct_emit(Instruction::Return { src: tr });
            }
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            span,
        } => {
            // C6: try to compile an if-else-if chain of single-char equality as CharDispatch.
            {
                let mut branches: Vec<(u8, &Stmt)> = Vec::new();
                let mut default: Option<&Stmt> = None;
                let mut var_name: Option<String> = None;
                if collect_char_dispatch(
                    condition,
                    then_branch,
                    else_branch.as_deref(),
                    &mut branches,
                    &mut default,
                    &mut var_name,
                ) {
                    if branches.len() >= 3 {
                        if let Some(ref vn) = var_name {
                            compile_char_dispatch(target, span, vn, branches, default)?;
                            return Ok(());
                        }
                    }
                }
            }
            let cr = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                target,
                condition,
                &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg())
                    .expect("out of register zones"),
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
                        Instruction::JumpIfFalse {
                            src: cr,
                            offset: jump_off(jump_to_else, else_start) as i16,
                        },
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
                        Instruction::JumpIfFalse {
                            src: cr,
                            offset: jump_off(jump_to_else, else_start) as i16,
                        },
                    );
                    compile_stmt_shared(target, else_stmt)?;
                }
            } else {
                let end = target.ct_current_ip();
                target.ct_patch(
                    jump_to_else,
                    Instruction::JumpIfFalse {
                        src: cr,
                        offset: jump_off(jump_to_else, end) as i16,
                    },
                );
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            // G12: unboxed float pass — kanıtlanırsa prolog emit eder ve
            // f-loop bağlamını kurar; kanıtlanamazsa None (eski yol).
            let floop_plan = crate::compiler::floop::enter(target, condition, body);
            let loop_start = target.ct_current_ip();
            let mut used_reg_cmp = false;
            if let Expr::Binary {
                left, op, right, ..
            } = condition
            {
                if matches!(
                    op,
                    BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge
                ) {
                    let get_reg = |e: &Expr| -> Option<u8> {
                        if let Expr::Identifier(name, _) = e {
                            target.ct_local_reg(name)
                        } else {
                            None
                        }
                    };
                    if let (Some(a_reg), Some(b_reg)) = (get_reg(left), get_reg(right)) {
                        let resolve = |n: &str| target.ct_local_type(n);
                        let l_ty = crate::compiler::expr::infer_type_with_locals(left, &resolve);
                        let r_ty = crate::compiler::expr::infer_type_with_locals(right, &resolve);
                        let both_int = (l_ty == crate::compiler::expr::ExprType::Int
                            || l_ty == crate::compiler::expr::ExprType::Unknown)
                            && (r_ty == crate::compiler::expr::ExprType::Int
                                || r_ty == crate::compiler::expr::ExprType::Unknown);
                        if both_int {
                            let jump_to_end = target.ct_current_ip();
                            match op {
                                BinaryOp::Lt => target.ct_emit(Instruction::IntLtRRJumpIfFalse {
                                    src1: a_reg,
                                    src2: b_reg,
                                    offset: 0,
                                }),
                                BinaryOp::Le => target.ct_emit(Instruction::IntLeRRJumpIfFalse {
                                    src1: a_reg,
                                    src2: b_reg,
                                    offset: 0,
                                }),
                                BinaryOp::Gt => target.ct_emit(Instruction::IntLtRRJumpIfFalse {
                                    src1: b_reg,
                                    src2: a_reg,
                                    offset: 0,
                                }),
                                BinaryOp::Ge => target.ct_emit(Instruction::IntLeRRJumpIfFalse {
                                    src1: b_reg,
                                    src2: a_reg,
                                    offset: 0,
                                }),
                                _ => unreachable!(),
                            }
                            used_reg_cmp = true;
                            let needs_break =
                                crate::compiler::helpers::body_contains_loop_exit(body);
                            let payload_idx = if needs_break {
                                Some(target.ct_add_loop_payload(loop_start as u32, 0))
                            } else {
                                None
                            };
                            if let Some(idx) = payload_idx {
                                target.ct_emit(Instruction::LoopBegin(idx));
                            }
                            target.ct_push_break_target(crate::compiler::target::BreakTarget::Loop);
                            compile_stmt_shared(target, body)?;
                            target.ct_pop_break_target();

                            if let Some(idx) = payload_idx {
                                target.ct_emit(Instruction::LoopEnd);
                            }
                            let back_jump_site = target.ct_current_ip();
                            target.ct_emit(Instruction::Jump(jump_off(back_jump_site, loop_start)));
                            let end = target.ct_current_ip();
                            if let Some(idx) = payload_idx {
                                target.ct_patch_loop_payload_end(idx, end as u32);
                            }
                            let offset = jump_off(jump_to_end, end);
                            let patch_instr = match op {
                                BinaryOp::Lt => Instruction::IntLtRRJumpIfFalse {
                                    src1: a_reg,
                                    src2: b_reg,
                                    offset: offset as i16,
                                },
                                BinaryOp::Le => Instruction::IntLeRRJumpIfFalse {
                                    src1: a_reg,
                                    src2: b_reg,
                                    offset: offset as i16,
                                },
                                BinaryOp::Gt => Instruction::IntLtRRJumpIfFalse {
                                    src1: b_reg,
                                    src2: a_reg,
                                    offset: offset as i16,
                                },
                                BinaryOp::Ge => Instruction::IntLeRRJumpIfFalse {
                                    src1: b_reg,
                                    src2: a_reg,
                                    offset: offset as i16,
                                },
                                _ => unreachable!(),
                            };
                            target.ct_patch(jump_to_end, patch_instr);
                            if let Some(idx) = payload_idx {
                                target.ct_patch_loop_payload_end(idx, end as u32);
                            }
                        }
                    }
                }
            }
            if !used_reg_cmp {
                let cr = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                    target,
                    condition,
                    &mut crate::compiler::regalloc::RegAlloc::new_with_base(
                        target.ct_next_local_reg(),
                    )
                    .expect("out of register zones"),
                );
                let jump_to_end = target.ct_current_ip();
                target.ct_emit(Instruction::JumpIfFalse { src: cr, offset: 0 });
                let needs_break = crate::compiler::helpers::body_contains_loop_exit(body);
                let payload_idx = if needs_break {
                    Some(target.ct_add_loop_payload(loop_start as u32, 0))
                } else {
                    None
                };
                if let Some(idx) = payload_idx {
                    target.ct_emit(Instruction::LoopBegin(idx));
                }
                target.ct_push_break_target(crate::compiler::target::BreakTarget::Loop);
                compile_stmt_shared(target, body)?;
                target.ct_pop_break_target();

                if let Some(idx) = payload_idx {
                    target.ct_emit(Instruction::LoopEnd);
                }
                let back_jump_site = target.ct_current_ip();
                target.ct_emit(Instruction::Jump(jump_off(back_jump_site, loop_start)));
                let end = target.ct_current_ip();
                target.ct_patch(
                    jump_to_end,
                    Instruction::JumpIfFalse {
                        src: cr,
                        offset: jump_off(jump_to_end, end) as i16,
                    },
                );
                if let Some(idx) = payload_idx {
                    target.ct_patch_loop_payload_end(idx, end as u32);
                }
            }
            // G12: epilog — döngü çıkışı (her iki yol da burada biter).
            crate::compiler::floop::exit(target, floop_plan);
        }
        Stmt::For {
            variable,
            iterable,
            body,
            ..
        } => {
            loops::compile_for_in(target, variable, iterable, body)?;
        }
        Stmt::ForCStyle {
            init,
            condition,
            update,
            body,
            ..
        } => {
            loops::compile_for_c_style(target, init, condition, update, body)?;
        }
        Stmt::ForRange {
            start,
            stop,
            step,
            body,
            ..
        } => {
            loops::compile_for_range(target, start, stop, step, body)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}
