use super::*;

use crate::compiler::expr::compile_reg::compile_expr_to_reg;
use crate::compiler::regalloc;

pub(crate) fn compile_optional_member(
    target: &mut impl CompileTarget,
    object: &Expr,
    property: &str,
    regs: &mut regalloc::RegAlloc,
    last_use: usize,
) -> u8 {
    let obj_reg = compile_expr_to_reg(target, object, regs);
    let null_idx = target.ct_emit_const(Value16::null());
    let null_reg = regs
        .alloc(target.ct_current_ip(), last_use)
        .expect("out of registers");
    target.ct_emit(Instruction::LoadConst {
        dst: null_reg,
        const_idx: null_idx as u16,
    });
    let cmp_reg = regs
        .alloc(target.ct_current_ip(), last_use)
        .expect("out of registers");
    target.ct_emit(Instruction::IntCmp {
        dst: cmp_reg,
        src1: obj_reg,
        src2: null_reg,
        op: 4,
    });
    let jump_null = target.ct_current_ip();
    target.ct_emit(Instruction::JumpIfTrue {
        src: cmp_reg,
        offset: 0,
    });
    let dst = regs
        .alloc(target.ct_current_ip(), last_use)
        .expect("out of registers");
    let prop_sym = target.ct_sym(property);
    target.ct_emit(Instruction::GetProperty {
        dst,
        obj: obj_reg,
        prop_sym: prop_sym.0 as u16,
    });
    let jump_end = target.ct_current_ip();
    target.ct_emit(Instruction::Jump(0));
    let null_case = target.ct_current_ip();
    target.ct_patch(
        jump_null,
        Instruction::JumpIfTrue {
            src: cmp_reg,
            offset: jump_off(jump_null, null_case) as i16,
        },
    );
    target.ct_emit(Instruction::LoadConst {
        dst,
        const_idx: null_idx as u16,
    });
    let end = target.ct_current_ip();
    target.ct_patch(jump_end, Instruction::Jump(jump_off(jump_end, end)));
    regs.free_now(obj_reg);
    regs.free_now(null_reg);
    regs.free_now(cmp_reg);
    dst
}

pub(crate) fn compile_binary(
    target: &mut impl CompileTarget,
    left: &Expr,
    op: &crate::compiler::BinaryOp,
    right: &Expr,
    regs: &mut regalloc::RegAlloc,
    last_use: usize,
) -> u8 {
    if matches!(op, BinaryOp::Add) {
        let resolve = |n: &str| target.ct_local_type(n);
        let l_ty = infer_type_with_locals(left, &resolve);
        let r_ty = infer_type_with_locals(right, &resolve);
        if l_ty == ExprType::Str || r_ty == ExprType::Str {
            let l_reg = compile_expr_to_reg(target, left, regs);
            let r_reg = compile_expr_to_reg(target, right, regs);
            let dst = regs
                .alloc(target.ct_current_ip(), last_use)
                .expect("out of registers");
            target.ct_emit(Instruction::StrCat {
                dst,
                src1: l_reg,
                src2: r_reg,
            });
            regs.free_now(l_reg);
            regs.free_now(r_reg);
            return dst;
        }
    }
    if let (Expr::Identifier(name, _), Expr::Literal(lit, _)) = (left, right) {
        let imm_i64 = match lit {
            Literal::Int(i) => Some(*i),
            Literal::Number(n, false) => Some(*n as i64),
            _ => None,
        };
        if let Some(imm) = imm_i64 {
            if target.ct_local_type(name) != ExprType::Number {
                if let Ok(imm_i16) = i16::try_from(imm) {
                    if let Some(reg) = target.ct_local_reg(name) {
                        let src = if target.ct_is_top_level() && target.ct_is_shared_top_level(name)
                        {
                            let dst = regs
                                .alloc(target.ct_current_ip(), last_use)
                                .expect("out of registers");
                            let sym = target.ct_intern(name);
                            target.ct_emit(Instruction::LoadGlobal {
                                dst,
                                sym: sym as u16,
                            });
                            dst
                        } else {
                            reg
                        };
                        let dst = regs
                            .alloc(target.ct_current_ip(), last_use)
                            .expect("out of registers");
                        match op {
                            BinaryOp::Add => {
                                target.ct_emit(Instruction::IntAddI {
                                    dst,
                                    src,
                                    imm: imm_i16,
                                });
                                return dst;
                            }
                            BinaryOp::Sub => {
                                target.ct_emit(Instruction::IntSubI {
                                    dst,
                                    src,
                                    imm: imm_i16,
                                });
                                return dst;
                            }
                            BinaryOp::Mul => {
                                target.ct_emit(Instruction::IntMulI {
                                    dst,
                                    src,
                                    imm: imm_i16,
                                });
                                return dst;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    let l_reg = compile_expr_to_reg(target, left, regs);
    let dst = regs
        .alloc(target.ct_current_ip(), last_use)
        .expect("out of registers");
    match op {
        BinaryOp::And => {
            let skip = target.ct_current_ip();
            target.ct_emit(Instruction::JumpIfFalse {
                src: l_reg,
                offset: 0,
            });
            let r_reg = compile_expr_to_reg(target, right, regs);
            target.emit_move(dst, r_reg);
            let over = target.ct_current_ip();
            target.ct_emit(Instruction::Jump(0));
            let false_path = target.ct_current_ip();
            target.ct_patch(
                skip,
                Instruction::JumpIfFalse {
                    src: l_reg,
                    offset: jump_off(skip, false_path) as i16,
                },
            );
            target.emit_move(dst, l_reg);
            let after = target.ct_current_ip();
            target.ct_patch(over, Instruction::Jump(jump_off(over, after)));
            regs.free_now(l_reg);
            regs.free_now(r_reg);
            return dst;
        }
        BinaryOp::Or => {
            let skip = target.ct_current_ip();
            target.ct_emit(Instruction::JumpIfTrue {
                src: l_reg,
                offset: 0,
            });
            let r_reg = compile_expr_to_reg(target, right, regs);
            target.emit_move(dst, r_reg);
            let over = target.ct_current_ip();
            target.ct_emit(Instruction::Jump(0));
            let true_path = target.ct_current_ip();
            target.ct_patch(
                skip,
                Instruction::JumpIfTrue {
                    src: l_reg,
                    offset: jump_off(skip, true_path) as i16,
                },
            );
            target.emit_move(dst, l_reg);
            let after = target.ct_current_ip();
            target.ct_patch(over, Instruction::Jump(jump_off(over, after)));
            regs.free_now(l_reg);
            regs.free_now(r_reg);
            return dst;
        }
        _ => {}
    }
    let r_reg = compile_expr_to_reg(target, right, regs);
    match op {
        BinaryOp::Add => {
            let resolve = |n: &str| target.ct_local_type(n);
            let l_ty = infer_type_with_locals(left, &resolve);
            let r_ty = infer_type_with_locals(right, &resolve);
            if l_ty == ExprType::Int && r_ty == ExprType::Int {
                target.ct_emit(Instruction::IntAdd {
                    dst,
                    src1: l_reg,
                    src2: r_reg,
                })
            } else if (l_ty == ExprType::Int || l_ty == ExprType::Number)
                && (r_ty == ExprType::Int || r_ty == ExprType::Number)
            {
                target.ct_emit(Instruction::NumAdd {
                    dst,
                    src1: l_reg,
                    src2: r_reg,
                })
            } else {
                target.ct_emit(Instruction::IntAdd {
                    dst,
                    src1: l_reg,
                    src2: r_reg,
                })
            }
        }
        BinaryOp::Sub => {
            let resolve = |n: &str| target.ct_local_type(n);
            let l_ty = infer_type_with_locals(left, &resolve);
            let r_ty = infer_type_with_locals(right, &resolve);
            if l_ty == ExprType::Int && r_ty == ExprType::Int {
                target.ct_emit(Instruction::IntSub {
                    dst,
                    src1: l_reg,
                    src2: r_reg,
                })
            } else if (l_ty == ExprType::Int || l_ty == ExprType::Number)
                && (r_ty == ExprType::Int || r_ty == ExprType::Number)
            {
                target.ct_emit(Instruction::NumSub {
                    dst,
                    src1: l_reg,
                    src2: r_reg,
                })
            } else {
                target.ct_emit(Instruction::IntSub {
                    dst,
                    src1: l_reg,
                    src2: r_reg,
                })
            }
        }
        BinaryOp::Mul => {
            let resolve = |n: &str| target.ct_local_type(n);
            let l_ty = infer_type_with_locals(left, &resolve);
            let r_ty = infer_type_with_locals(right, &resolve);
            if l_ty == ExprType::Int && r_ty == ExprType::Int {
                target.ct_emit(Instruction::IntMul {
                    dst,
                    src1: l_reg,
                    src2: r_reg,
                })
            } else if (l_ty == ExprType::Int || l_ty == ExprType::Number)
                && (r_ty == ExprType::Int || r_ty == ExprType::Number)
            {
                target.ct_emit(Instruction::NumMul {
                    dst,
                    src1: l_reg,
                    src2: r_reg,
                })
            } else {
                target.ct_emit(Instruction::IntMul {
                    dst,
                    src1: l_reg,
                    src2: r_reg,
                })
            }
        }
        BinaryOp::Div => {
            let resolve = |n: &str| target.ct_local_type(n);
            let l_ty = infer_type_with_locals(left, &resolve);
            let r_ty = infer_type_with_locals(right, &resolve);
            let l_is_int = l_ty == ExprType::Int || l_ty == ExprType::Unknown;
            let r_is_int = r_ty == ExprType::Int || r_ty == ExprType::Unknown;
            if l_is_int && r_is_int {
                target.ct_emit(Instruction::IntDiv {
                    dst,
                    src1: l_reg,
                    src2: r_reg,
                })
            } else {
                target.ct_emit(Instruction::NumDiv {
                    dst,
                    src1: l_reg,
                    src2: r_reg,
                })
            }
        }
        BinaryOp::Mod => {
            let resolve = |n: &str| target.ct_local_type(n);
            let l_ty = infer_type_with_locals(left, &resolve);
            let r_ty = infer_type_with_locals(right, &resolve);
            let l_is_int = l_ty == ExprType::Int || l_ty == ExprType::Unknown;
            let r_is_int = r_ty == ExprType::Int || r_ty == ExprType::Unknown;
            if l_is_int && r_is_int {
                target.ct_emit(Instruction::IntMod {
                    dst,
                    src1: l_reg,
                    src2: r_reg,
                })
            } else {
                target.ct_emit(Instruction::NumMod {
                    dst,
                    src1: l_reg,
                    src2: r_reg,
                })
            }
        }
        BinaryOp::Lt => target.ct_emit(Instruction::IntCmp {
            dst,
            src1: l_reg,
            src2: r_reg,
            op: 0,
        }),
        BinaryOp::Le => target.ct_emit(Instruction::IntCmp {
            dst,
            src1: l_reg,
            src2: r_reg,
            op: 1,
        }),
        BinaryOp::Gt => target.ct_emit(Instruction::IntCmp {
            dst,
            src1: l_reg,
            src2: r_reg,
            op: 2,
        }),
        BinaryOp::Ge => target.ct_emit(Instruction::IntCmp {
            dst,
            src1: l_reg,
            src2: r_reg,
            op: 3,
        }),
        BinaryOp::Eq => target.ct_emit(Instruction::IntCmp {
            dst,
            src1: l_reg,
            src2: r_reg,
            op: 4,
        }),
        BinaryOp::Ne => target.ct_emit(Instruction::IntCmp {
            dst,
            src1: l_reg,
            src2: r_reg,
            op: 5,
        }),
        BinaryOp::And => { /* handled above with short-circuit */ }
        BinaryOp::Or => { /* handled above with short-circuit */ }
        _ => {
            regs.free_now(l_reg);
            regs.free_now(r_reg);
            // B5: don't free dst — it's the result returned to parent expression
            return dst;
        }
    }
    regs.free_now(l_reg);
    regs.free_now(r_reg);
    dst
}
