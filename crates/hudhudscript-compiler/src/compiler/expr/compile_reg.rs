use super::*;
use crate::compiler::regalloc;

pub(crate) fn compile_expr_to_reg(
    target: &mut impl CompileTarget,
    expr: &Expr,
    regs: &mut regalloc::RegAlloc,
) -> u8 {
    let ip = target.ct_current_ip();
    // Use a generous last_use to prevent premature register expiry
    // during recursive expression compilation (e.g. function calls
    // inside binary operands can add many instructions).
    let last_use = ip + 255;

    match expr {
        Expr::Literal(Literal::Number(n, is_float), _) => {
            if *is_float {
                let idx = target.ct_emit_num_const(*n);
                let dst = regs.alloc(ip, last_use).expect("out of registers");
                target.ct_emit(Instruction::LoadNumConst { dst, const_idx: idx as u16 });
                dst
            } else {
                let i = *n as i64;
                let idx = target.ct_emit_int_const(i);
                let dst = regs.alloc(ip, last_use).expect("out of registers");
                target.ct_emit(Instruction::LoadIntConst { dst, const_idx: idx as u16 });
                dst
            }
        }
        Expr::Literal(_, _) => {
            let v = literal_to_value(expr);
            let idx = target.ct_emit_const(v);
            let dst = regs.alloc(ip, last_use).expect("out of registers");
            target.ct_emit(Instruction::LoadConst { dst, const_idx: idx as u16 });
            dst
        }
        Expr::This(_) => {
            // SOP: If "self" or "this" is a local variable (e.g. ability param),
            // use the local register instead of globals.
            if let Some(reg) = target.ct_local_reg("self")
                .or_else(|| target.ct_local_reg("this"))
            {
                let dst = regs.alloc(ip, last_use).expect("out of registers");
                target.ct_emit(Instruction::Move { dst, src: reg });
                dst
            } else {
                let sym = target.ct_intern("this");
                let dst = regs.alloc(ip, last_use).expect("out of registers");
                target.ct_emit(Instruction::LoadGlobal { dst, sym: sym as u16 });
                dst
            }
        }
        Expr::Identifier(name, _) => {
            target.ct_track_reference(name);
            let dst = regs.alloc(ip, last_use).expect("out of registers");
            if let Some(reg) = target.ct_local_reg(name) {
                target.ct_emit(Instruction::Move { dst, src: reg });
            } else {
                let sym = target.ct_intern(name);
                target.ct_emit(Instruction::LoadGlobal { dst, sym: sym as u16 });
            }
            dst
        }
        Expr::Member { object, property, .. } => {
            let obj_reg = compile_expr_to_reg(target, object, regs);
            let dst = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
            let prop_sym = target.ct_sym(property);
            target.ct_emit(Instruction::GetProperty { dst, obj: obj_reg, prop_sym: prop_sym.0 as u16 });
            regs.free_now(obj_reg);
            dst
        }
        Expr::Array { elements, .. } => {
            let has_spread = elements.iter().any(|e| matches!(e, Expr::Spread { .. }));
            let dst = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
            if has_spread {
                target.ct_emit(Instruction::MakeArray { dst, count: 0 });
                for elem in elements {
                    if let Expr::Spread { expr, .. } = elem {
                        let src = compile_expr_to_reg(target, expr, &mut RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"));
                        target.ct_emit(Instruction::SpreadIntoArray { dst, src });
                    } else {
                        let val = compile_expr_to_reg(target, elem, &mut RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"));
                        target.ct_emit(Instruction::ArrayPush { dst, arr: dst, val });
                    }
                }
            } else {
                target.ct_emit(Instruction::MakeArray { dst, count: 0 });
                for elem in elements {
                    let r = compile_expr_to_reg(target, elem, regs);
                    target.ct_emit(Instruction::ArrayPush { dst, arr: dst, val: r });
                    regs.free_now(r);
                }
            }
            dst
        }
        Expr::Object { properties, .. } => {
            let has_spread = properties.iter().any(|(_, v)| matches!(v, Expr::Spread { .. }));
            let dst = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
            if has_spread {
                target.ct_emit(Instruction::MakeObject { dst, count: 0 });
                for (key, value) in properties {
                    if let Expr::Spread { expr, .. } = value {
                        let src = compile_expr_to_reg(target, expr, &mut RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"));
                        target.ct_emit(Instruction::SpreadIntoObject { dst, src });
                    } else {
                        let val_reg = compile_expr_to_reg(target, value, &mut RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"));
                        let key_sym = target.ct_sym(key);
                        target.ct_emit(Instruction::SetProperty { dst, obj: dst, val: val_reg, prop_sym: key_sym.0 as u16 });
                    }
                }
            } else {
                target.ct_emit(Instruction::MakeObject { dst, count: 0 });
                for (key, value) in properties {
                    let val_reg = compile_expr_to_reg(target, value, regs);
                    let key_sym = target.ct_sym(key);
                    target.ct_emit(Instruction::SetProperty { dst, obj: dst, val: val_reg, prop_sym: key_sym.0 as u16 });
                    regs.free_now(val_reg);
                }
            }
            dst
        }
        Expr::Call { callee, args, .. } => {
            let has_spread = args.iter().any(|a| matches!(a, Expr::Spread { .. }));
            if !has_spread {
                if let Expr::Identifier(name, _) = &**callee {
                    if name != "super" && !target.ct_is_known_generator(name) {
                        let argc = args.len() as u8;
                        // Compile args first, then allocate a contiguous block
                        let mut arg_regs = Vec::with_capacity(args.len());
                        for arg in args.iter() {
                            let r = compile_expr_to_reg(target, arg, regs);
                            arg_regs.push(r);
                        }
                        let call_ip = target.ct_current_ip();
                        let first_arg = regs.alloc_contiguous(argc, call_ip, call_ip + 1)
                            .expect("out of contiguous registers");
                        for (i, r) in arg_regs.iter().enumerate() {
                            target.ct_emit(Instruction::Move { dst: first_arg + i as u8, src: *r });
                            regs.free_now(*r);
                        }
                        let name_sym = target.ct_sym(name);
                        let idx = target.ct_add_call_payload(name_sym, argc);
                        let dst = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
                        target.ct_emit(Instruction::Call { dst, payload_idx: idx as u16, first_arg, arg_count: argc });
                        return dst;
                    }
                }
            }
            crate::compiler::expr::compile_complex::compile_expr_complex(target, expr)
                .expect("compile_complex failed")
        }
        Expr::Perform { action, .. } => {
            // Perform: extract Agent.action(args) → compile as action call
            if let Expr::Call { callee, args, .. } = action.as_ref() {
                if let Expr::Member { object, property, .. } = callee.as_ref() {
                    if let Expr::Identifier(agent_name, _) = object.as_ref() {
                        let qualified = format!("{}.{}", agent_name, property);
                        let argc = args.len() as u8;
                        let mut arg_regs = Vec::with_capacity(args.len());
                        for arg in args.iter() {
                            let r = compile_expr_to_reg(target, arg, regs);
                            arg_regs.push(r);
                        }
                        let call_ip = target.ct_current_ip();
                        let first_arg = regs.alloc_contiguous(argc, call_ip, call_ip + 1)
                            .expect("out of contiguous registers for perform");
                        for (i, r) in arg_regs.iter().enumerate() {
                            target.ct_emit(Instruction::Move { dst: first_arg + i as u8, src: *r });
                            regs.free_now(*r);
                        }
                        let name_sym = target.ct_sym(&qualified);
                        let idx = target.ct_add_call_payload(name_sym, argc);
                        let dst = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
                        target.ct_emit(Instruction::Call { dst, payload_idx: idx as u16, first_arg, arg_count: argc });
                        return dst;
                    }
                }
            }
            crate::compiler::expr::compile_complex::compile_expr_complex(target, expr)
                .expect("compile_complex failed")
        }
        Expr::Unary { op, expr: inner, .. } => {
            match op {
                UnaryOp::PostIncrement | UnaryOp::PostDecrement => {
                    let imm: i16 = if matches!(op, UnaryOp::PostIncrement) { 1 } else { -1 };
                    if let Expr::Identifier(name, _) = inner.as_ref() {
                        if let Some(reg) = target.ct_local_reg(name) {
                            let dst = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
                            target.ct_emit(Instruction::Move { dst, src: reg });
                            target.ct_emit(Instruction::IntAddI { dst: reg, src: reg, imm });
                            return dst;
                        }
                    }
                    // Not a local identifier — should be caught by parser/semantic analysis
                    unreachable!("Postfix ++/-- can only be applied to local variables");
                }
                _ => {}
            }
            let src = compile_expr_to_reg(target, inner, regs);
            let dst = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
            match op {
                UnaryOp::Neg => target.ct_emit(Instruction::Neg { dst, src }),
                UnaryOp::Not => target.ct_emit(Instruction::Not { dst, src }),
                UnaryOp::Plus => {
                    // Unary plus is a no-op — return src directly
                    regs.free_now(dst);
                    return src;
                }
                _ => {}
            }
            regs.free_now(src);
            dst
        }
        Expr::Index { object, index, .. } => {
            let obj_reg = compile_expr_to_reg(target, object, regs);
            let idx_reg = compile_expr_to_reg(target, index, regs);
            let dst = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
            target.ct_emit(Instruction::Index { dst, obj: obj_reg, idx: idx_reg });
            regs.free_now(obj_reg);
            regs.free_now(idx_reg);
            dst
        }
        Expr::Ternary { condition, true_expr, false_expr, .. } => {
            let cond_reg = compile_expr_to_reg(target, condition, regs);
            let dst = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
            let jump_to_else = target.ct_current_ip();
            target.ct_emit(Instruction::JumpIfFalse { src: cond_reg, offset: 0 });
            // True branch
            let true_reg = compile_expr_to_reg(target, true_expr, regs);
            target.ct_emit(Instruction::Move { dst, src: true_reg });
            regs.free_now(true_reg);
            // Jump to end
            let jump_to_end = target.ct_current_ip();
            target.ct_emit(Instruction::Jump(0));
            // Else branch
            let else_ip = target.ct_current_ip();
            target.ct_patch(jump_to_else, Instruction::JumpIfFalse {
                src: cond_reg, offset: jump_off(jump_to_else, else_ip) as i16,
            });
            regs.free_now(cond_reg);
            let false_reg = compile_expr_to_reg(target, false_expr, regs);
            target.ct_emit(Instruction::Move { dst, src: false_reg });
            regs.free_now(false_reg);
            // Patch end jump
            let end_ip = target.ct_current_ip();
            target.ct_patch(jump_to_end, Instruction::Jump(jump_off(jump_to_end, end_ip)));
            dst
        }
        Expr::OptionalMember { object, property, .. } => {
            let obj_reg = compile_expr_to_reg(target, object, regs);
            let null_idx = target.ct_emit_const(Value16::null());
            let null_reg = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
            target.ct_emit(Instruction::LoadConst { dst: null_reg, const_idx: null_idx as u16 });
            let cmp_reg = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
            target.ct_emit(Instruction::IntCmp { dst: cmp_reg, src1: obj_reg, src2: null_reg, op: 4 });
            let jump_null = target.ct_current_ip();
            target.ct_emit(Instruction::JumpIfTrue { src: cmp_reg, offset: 0 });
            let dst = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
            let prop_sym = target.ct_sym(property);
            target.ct_emit(Instruction::GetProperty { dst, obj: obj_reg, prop_sym: prop_sym.0 as u16 });
            let jump_end = target.ct_current_ip();
            target.ct_emit(Instruction::Jump(0));
            let null_case = target.ct_current_ip();
            target.ct_patch(
                jump_null,
                Instruction::JumpIfTrue { src: cmp_reg, offset: jump_off(jump_null, null_case) as i16 },
            );
            target.ct_emit(Instruction::LoadConst { dst, const_idx: null_idx as u16 });
            let end = target.ct_current_ip();
            target.ct_patch(jump_end, Instruction::Jump(jump_off(jump_end, end)));
            regs.free_now(obj_reg);
            regs.free_now(null_reg);
            regs.free_now(cmp_reg);
            dst
        }
        Expr::Binary { left, op, right, .. } => {
            // String concatenation uses register-based StrCat
            if matches!(op, BinaryOp::Add) {
                let resolve = |n: &str| target.ct_local_type(n);
                let l_ty = infer_type_with_locals(left, &resolve);
                let r_ty = infer_type_with_locals(right, &resolve);
                if l_ty == ExprType::Str || r_ty == ExprType::Str {
                    let l_reg = compile_expr_to_reg(target, left, regs);
                    let r_reg = compile_expr_to_reg(target, right, regs);
                    // SAFETY: dst == src1 in-place mutation is unsafe with Value16's
                    // shared raw-pointer strings — other copies of the same string
                    // (e.g., in local_slots) would see the mutation. TemplateString
                    // already uses a fresh dst; binary + must do the same.
                    let dst = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
                    target.ct_emit(Instruction::StrCat { dst, src1: l_reg, src2: r_reg });
                    regs.free_now(l_reg);
                    regs.free_now(r_reg);
                    return dst;
                }
            }
            // Fusion: local + imm → IntAddI / IntSubI (K1-1: locals are registers)
            if let (Expr::Identifier(name, _), Expr::Literal(Literal::Number(n, is_float), _)) = (left.as_ref(), right.as_ref()) {
                if !is_float && target.ct_local_type(name) != ExprType::Number {
                    if let Some(reg) = target.ct_local_reg(name) {
                        let imm = *n as i64;
                        if let Ok(imm_i16) = i16::try_from(imm) {
                            let dst = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
                            match op {
                                BinaryOp::Add => {
                                    target.ct_emit(Instruction::IntAddI { dst, src: reg, imm: imm_i16 });
                                    return dst;
                                }
                                BinaryOp::Sub => {
                                    target.ct_emit(Instruction::IntSubI { dst, src: reg, imm: imm_i16 });
                                    return dst;
                                }
                                BinaryOp::Mul => {
                                    target.ct_emit(Instruction::IntMulI { dst, src: reg, imm: imm_i16 });
                                    return dst;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            let l_reg = compile_expr_to_reg(target, left, regs);
            let dst = regs.alloc(target.ct_current_ip(), last_use).expect("out of registers");
            match op {
                BinaryOp::And => {
                    // SHORT-CIRCUIT: if left falsy → dst=left, skip right
                    let skip = target.ct_current_ip();
                    target.ct_emit(Instruction::JumpIfFalse { src: l_reg, offset: 0 });
                    let r_reg = compile_expr_to_reg(target, right, regs);
                    target.ct_emit(Instruction::Move { dst, src: r_reg });
                    let over = target.ct_current_ip();
                    target.ct_emit(Instruction::Jump(0));
                    let false_path = target.ct_current_ip();
                    target.ct_patch(skip, Instruction::JumpIfFalse { src: l_reg, offset: jump_off(skip, false_path) as i16 });
                    target.ct_emit(Instruction::Move { dst, src: l_reg });
                    let after = target.ct_current_ip();
                    target.ct_patch(over, Instruction::Jump(jump_off(over, after)));
                    regs.free_now(l_reg);
                    regs.free_now(r_reg);
                    return dst;
                }
                BinaryOp::Or => {
                    // SHORT-CIRCUIT: if left truthy → dst=left, skip right
                    let skip = target.ct_current_ip();
                    target.ct_emit(Instruction::JumpIfTrue { src: l_reg, offset: 0 });
                    let r_reg = compile_expr_to_reg(target, right, regs);
                    target.ct_emit(Instruction::Move { dst, src: r_reg });
                    let over = target.ct_current_ip();
                    target.ct_emit(Instruction::Jump(0));
                    let true_path = target.ct_current_ip();
                    target.ct_patch(skip, Instruction::JumpIfTrue { src: l_reg, offset: jump_off(skip, true_path) as i16 });
                    target.ct_emit(Instruction::Move { dst, src: l_reg });
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
                BinaryOp::Add => target.ct_emit(Instruction::IntAdd { dst, src1: l_reg, src2: r_reg }),
                BinaryOp::Sub => target.ct_emit(Instruction::IntSub { dst, src1: l_reg, src2: r_reg }),
                BinaryOp::Mul => target.ct_emit(Instruction::IntMul { dst, src1: l_reg, src2: r_reg }),
                BinaryOp::Div => {
                    let resolve = |n: &str| target.ct_local_type(n);
                    let l_ty = infer_type_with_locals(left, &resolve);
                    let r_ty = infer_type_with_locals(right, &resolve);
                    if l_ty == ExprType::Int && r_ty == ExprType::Int {
                        target.ct_emit(Instruction::IntDiv { dst, src1: l_reg, src2: r_reg })
                    } else {
                        target.ct_emit(Instruction::NumDiv { dst, src1: l_reg, src2: r_reg })
                    }
                }
                BinaryOp::Mod => {
                    let resolve = |n: &str| target.ct_local_type(n);
                    let l_ty = infer_type_with_locals(left, &resolve);
                    let r_ty = infer_type_with_locals(right, &resolve);
                    if l_ty == ExprType::Int && r_ty == ExprType::Int {
                        target.ct_emit(Instruction::IntMod { dst, src1: l_reg, src2: r_reg })
                    } else {
                        target.ct_emit(Instruction::NumMod { dst, src1: l_reg, src2: r_reg })
                    }
                },
                BinaryOp::Lt  => target.ct_emit(Instruction::IntCmp { dst, src1: l_reg, src2: r_reg, op: 0 }),
                BinaryOp::Le  => target.ct_emit(Instruction::IntCmp { dst, src1: l_reg, src2: r_reg, op: 1 }),
                BinaryOp::Gt  => target.ct_emit(Instruction::IntCmp { dst, src1: l_reg, src2: r_reg, op: 2 }),
                BinaryOp::Ge  => target.ct_emit(Instruction::IntCmp { dst, src1: l_reg, src2: r_reg, op: 3 }),
                BinaryOp::Eq  => target.ct_emit(Instruction::IntCmp { dst, src1: l_reg, src2: r_reg, op: 4 }),
                BinaryOp::Ne  => target.ct_emit(Instruction::IntCmp { dst, src1: l_reg, src2: r_reg, op: 5 }),
                BinaryOp::And => { /* handled above with short-circuit */ }
                BinaryOp::Or => { /* handled above with short-circuit */ }
                _ => { regs.free_now(l_reg); regs.free_now(r_reg); regs.free_now(dst); return dst; }
            }
            regs.free_now(l_reg);
            regs.free_now(r_reg);
            dst
        }
        _ => {
            crate::compiler::expr::compile_complex::compile_expr_complex(target, expr)
                .expect("compile_complex failed");
            let dst = regs.alloc(ip, last_use).expect("out of registers");
            target.ct_emit(Instruction::Move { dst, src: 255 });
            dst
        }
    }
}

fn literal_to_value(expr: &Expr) -> Value16 {
    match expr {
        Expr::Literal(Literal::String(s), _) => Value16::string(s.clone()),
        Expr::Literal(Literal::Boolean(b), _) => Value16::bool_(*b),
        Expr::Literal(Literal::Null, _) => Value16::null(),
        _ => Value16::null(),
    }
}
