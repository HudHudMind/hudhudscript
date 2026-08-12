use super::*;

pub(crate) fn compile_expr_complex_inner_extra(
    target: &mut impl CompileTarget,
    expr: &Expr,
    regs: &mut RegAlloc,
    base_ip: usize,
) -> CompileResult<u8> {
    match expr {
        Expr::Await { expr, span } => {
            target.ct_check_await(span)?;
            {
                        let r = compile_expr_to_reg(target, expr, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                        target.emit_move(255, r );
                    }
            target.ct_emit(Instruction::Await { src: 255, dst: 255 });
        }

        Expr::Spread { expr, .. } => {
            {
                        let r = compile_expr_to_reg(target, expr, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                        target.emit_move(255, r );
                    }
        }

        Expr::Yield { value, .. } => {
            if let Some(val_expr) = value {
                {
                        let r = compile_expr_to_reg(target, val_expr, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                        target.emit_move(255, r );
                    }
            } else {
                let idx = target.ct_emit_const(Value16::null());
                { let tr = crate::compiler::regalloc::temp_reg(); target.ct_emit(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); target.emit_move(255, tr ); }
            }
            target.ct_emit(Instruction::Yield { src: 255 });
        }

        Expr::Spawn {
            subject_name, args, ..
        } => {
            let name_sym = target.ct_sym(subject_name);
            let argc = args.len() as u8;
            let first_arg = crate::compiler::regalloc::temp_reg();
            for (i, arg) in args.iter().enumerate() {
                let r = compile_expr_to_reg(target, arg, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                target.emit_move(first_arg + i as u8, r );
            }
            target.ct_emit(Instruction::Spawn { name_sym: name_sym.0, first_arg, arg_count: argc });
        }

        Expr::ViewAs { instance, view_name, .. } => {
            let obj_reg = compile_expr_to_reg(target, instance, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            let view_sym = target.ct_sym(view_name).0 as u16;
            target.ct_emit(Instruction::ViewAs { obj: obj_reg, view_sym });
        }

        Expr::OptionalMember {
            object, property, ..
        } => {
            let obj_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(
                target, object, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
            );
            let null_idx = target.ct_emit_const(Value16::null());
            let null_reg = crate::compiler::regalloc::temp_reg();
            target.ct_emit(Instruction::LoadConst { dst: null_reg, const_idx: null_idx as u16 });
            let cmp_reg = crate::compiler::regalloc::temp_reg();
            target.ct_emit(Instruction::IntCmp { dst: cmp_reg, src1: obj_reg, src2: null_reg, op: 4 });
            let jump_null = target.ct_current_ip();
            target.ct_emit(Instruction::JumpIfTrue { src: cmp_reg, offset: 0 });
            let dst_reg = crate::compiler::regalloc::temp_reg();
            let prop_sym = target.ct_sym(property);
            target.ct_emit(Instruction::GetProperty { dst: dst_reg, obj: obj_reg, prop_sym: prop_sym.0 as u16 });
            target.emit_move(255, dst_reg );
            let jump_end = target.ct_current_ip();
            target.ct_emit(Instruction::Jump(0));
            let null_case = target.ct_current_ip();
            target.ct_patch(
                jump_null,
                Instruction::JumpIfTrue { src: cmp_reg, offset: jump_off(jump_null, null_case) as i16 },
            );
            { let tr = crate::compiler::regalloc::temp_reg(); target.ct_emit(Instruction::LoadConst { dst: tr, const_idx: null_idx as u16 }); target.emit_move(255, tr ); }
            let end = target.ct_current_ip();
            target.ct_patch(jump_end, Instruction::Jump(jump_off(jump_end, end)));
        }

        _ => {
            let pos = span_pos(&expr.span());
            return Err(compile_codes::unsupported_feature_at(
                format!("Expression type not yet supported: {:?}", expr),
                pos,
            ));
        }
    }
    Ok(255)
}
