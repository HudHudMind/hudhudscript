use super::*;

pub(super) fn compile_for_in(
    target: &mut impl CompileTarget,
    variable: &str,
    iterable: &Expr,
    body: &Stmt,
) -> CompileResult<()> {
    let mut regs = RegAlloc::new_with_base(target.ct_next_local_reg())?;
    let ip = target.ct_current_ip();
    let iter_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, iterable, &mut regs);
    let var_sym = target.ct_sym(variable);
    target.ct_declare_local(variable, false)?;
    target.ct_emit(Instruction::ForIn { iter_reg, var_sym_idx: var_sym.0 as u16, end_offset: 0 });
    let loop_top = target.ct_current_ip();
    target.ct_emit(Instruction::IterNext { iter_reg, var_sym_idx: var_sym.0 as u16, end_offset: 0 });
    compile_stmt_shared(target, body)?;
    let back_jump_site = target.ct_current_ip();
    target.ct_emit(Instruction::Jump(jump_off(back_jump_site, loop_top)));
    let end = target.ct_current_ip();
    target.ct_patch(loop_top, Instruction::IterNext { iter_reg, var_sym_idx: var_sym.0 as u16, end_offset: jump_off(loop_top, end) as i16 });
    Ok(())
}

pub(super) fn compile_for_c_style(
    target: &mut impl CompileTarget,
    init: &Option<Box<Stmt>>,
    condition: &Option<Expr>,
    update: &Option<Box<Stmt>>,
    body: &Stmt,
) -> CompileResult<()> {
    if let Some(init_stmt) = init {
        compile_stmt_shared(target, init_stmt)?;
    }
    let loop_start = target.ct_current_ip();

    // P1: fused compare+branch for simple identifier loop conditions
    // (e.g. `i < n` in fib_iterative) instead of IntCmp + JumpIfFalse.
    if let Some(cond) = condition {
        if let Expr::Binary { left, op, right, .. } = cond {
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
                        let payload_idx = target.ct_add_loop_payload(loop_start as u32, 0);
                        target.ct_emit(Instruction::LoopBegin(payload_idx));
                        compile_stmt_shared(target, body)?;
                        target.ct_emit(Instruction::LoopEnd);
                        if let Some(update_stmt) = update {
                            compile_stmt_shared(target, update_stmt)?;
                        }
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
                        return Ok(());
                    }
                }
            }
        }
    }

    let cr = if let Some(cond) = condition {
        crate::compiler::expr::compile_reg::compile_expr_to_reg(
            target, cond, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
        )
    } else {
        let true_idx = target.ct_emit_const(Value16::bool_(true));
        let tr = crate::compiler::regalloc::temp_reg();
        target.ct_emit(Instruction::LoadConst { dst: tr, const_idx: true_idx as u16 });
        tr
    };
    let jump_to_end = target.ct_current_ip();
    target.ct_emit(Instruction::JumpIfFalse { src: cr, offset: 0 });
    let payload_idx = target.ct_add_loop_payload(loop_start as u32, 0);
    target.ct_emit(Instruction::LoopBegin(payload_idx));
    compile_stmt_shared(target, body)?;
    target.ct_emit(Instruction::LoopEnd);
    if let Some(update_stmt) = update {
        compile_stmt_shared(target, update_stmt)?;
    }
    let back_jump_site = target.ct_current_ip();
    target.ct_emit(Instruction::Jump(jump_off(back_jump_site, loop_start)));
    let end = target.ct_current_ip();
    target.ct_patch(
        jump_to_end,
        Instruction::JumpIfFalse { src: cr, offset: jump_off(jump_to_end, end) as i16 },
    );
    target.ct_patch_loop_payload_end(payload_idx, end as u32);
    Ok(())
}

pub(super) fn compile_for_range(
    target: &mut impl CompileTarget,
    start: &Expr,
    stop: &Expr,
    step: &Option<Expr>,
    body: &Stmt,
) -> CompileResult<()> {
    let iter_var = "__range_i".to_string();
    let dir_var = "__range_ascending".to_string();
    target.ct_declare_local(&iter_var, false)?;
    target.ct_declare_local(&dir_var, false)?;
    let iter_reg = target.ct_local_reg(&iter_var).unwrap_or(0);
    let dir_reg = target.ct_local_reg(&dir_var).unwrap_or(0);

    let start_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
        target, start, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
    );
    target.ct_emit(Instruction::Move { dst: iter_reg, src: start_reg });

    let start_reg2 = crate::compiler::expr::compile_reg::compile_expr_to_reg(
        target, start, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
    );
    let stop_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
        target, stop, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
    );
    let dir_reg = crate::compiler::regalloc::temp_reg();
    target.ct_emit(Instruction::IntCmp { dst: dir_reg, src1: start_reg2, src2: stop_reg, op: 1 });
    target.ct_emit(Instruction::Move { dst: dir_reg, src: dir_reg });

    let loop_start = target.ct_current_ip();

    let dir_load = crate::compiler::regalloc::temp_reg();
    target.ct_emit(Instruction::Move { dst: dir_load, src: dir_reg });
    let asc_jump = target.ct_current_ip();
    target.ct_emit(Instruction::JumpIfFalse { src: dir_load, offset: 0 });

    let iter_load1 = crate::compiler::regalloc::temp_reg();
    target.ct_emit(Instruction::Move { dst: iter_load1, src: iter_reg });
    let stop_reg2 = crate::compiler::expr::compile_reg::compile_expr_to_reg(
        target, stop, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
    );
    let cond_asc = crate::compiler::regalloc::temp_reg();
    target.ct_emit(Instruction::IntLeRRJumpIfFalse { src1: iter_load1, src2: stop_reg2, offset: 0 });
    let cond_done_jump = target.ct_current_ip();
    target.ct_emit(Instruction::Jump(0));

    let desc_start = target.ct_current_ip();
    target.ct_patch(asc_jump,
        Instruction::JumpIfFalse { src: dir_load, offset: jump_off(asc_jump, desc_start) as i16 });

    let iter_load2 = crate::compiler::regalloc::temp_reg();
    target.ct_emit(Instruction::Move { dst: iter_load2, src: iter_reg });
    let stop_reg3 = crate::compiler::expr::compile_reg::compile_expr_to_reg(
        target, stop, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
    );
    let cond_desc = crate::compiler::regalloc::temp_reg();
    target.ct_emit(Instruction::IntLeRRJumpIfFalse { src1: stop_reg3, src2: iter_load2, offset: 0 });

    let cond_check = target.ct_current_ip();
    target.ct_patch(cond_done_jump, Instruction::Jump(jump_off(cond_done_jump, cond_check)));

    let jump_to_end = target.ct_current_ip();
    target.ct_emit(Instruction::JumpIfFalse { src: cond_desc, offset: 0 });

    compile_stmt_shared(target, body)?;

    let iter_load3 = crate::compiler::regalloc::temp_reg();
    target.ct_emit(Instruction::Move { dst: iter_load3, src: iter_reg });

    let step_reg = if let Some(step_expr) = step {
        crate::compiler::expr::compile_reg::compile_expr_to_reg(
            target, step_expr, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
        )
    } else {
        let dir_load2 = crate::compiler::regalloc::temp_reg();
        target.ct_emit(Instruction::Move { dst: dir_load2, src: dir_reg });
        let step_asc_jump = target.ct_current_ip();
        target.ct_emit(Instruction::JumpIfFalse { src: dir_load2, offset: 0 });

        let one_reg = crate::compiler::regalloc::temp_reg();
        let pos_one_idx = target.ct_emit_num_const(1.0);
        target.ct_emit(Instruction::LoadNumConst { dst: one_reg, const_idx: pos_one_idx as u16 });

        let step_done_jump = target.ct_current_ip();
        target.ct_emit(Instruction::Jump(0));

        let neg_step_start = target.ct_current_ip();
        target.ct_patch(step_asc_jump,
            Instruction::JumpIfFalse { src: dir_load2, offset: jump_off(step_asc_jump, neg_step_start) as i16 });

        let neg_one_reg = crate::compiler::regalloc::temp_reg();
        let neg_one_idx = target.ct_emit_num_const(-1.0);
        target.ct_emit(Instruction::LoadNumConst { dst: neg_one_reg, const_idx: neg_one_idx as u16 });

        let step_done = target.ct_current_ip();
        target.ct_patch(step_done_jump, Instruction::Jump(jump_off(step_done_jump, step_done)));

        let final_step = crate::compiler::regalloc::temp_reg();
        target.ct_emit(Instruction::LoadNumConst { dst: final_step, const_idx: pos_one_idx as u16 });
        final_step
    };

    let new_iter = crate::compiler::regalloc::temp_reg();
    target.ct_emit(Instruction::NumAdd { dst: new_iter, src1: iter_load3, src2: step_reg });
    target.ct_emit(Instruction::Move { dst: iter_reg, src: new_iter });

    let back_jump_site = target.ct_current_ip();
    target.ct_emit(Instruction::Jump(jump_off(back_jump_site, loop_start)));
    let end = target.ct_current_ip();
    target.ct_patch(jump_to_end,
        Instruction::JumpIfFalse { src: cond_desc, offset: jump_off(jump_to_end, end) as i16 });
    Ok(())
}
