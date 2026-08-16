use super::*;
use crate::compiler::regalloc;
pub(crate) fn compile_expr_to_reg(
    target: &mut impl CompileTarget,
    expr: &Expr,
    regs: &mut regalloc::RegAlloc,
) -> u8 {
    let ip = target.ct_current_ip();
    let last_use = ip + 255;
    match expr {
        Expr::Literal(Literal::Number(n, is_float), _) => {
            if *is_float {
                let idx = target.ct_emit_num_const(*n);
                let dst = regs.alloc(ip, last_use).expect("out of registers");
                target.ct_emit(Instruction::LoadNumConst {
                    dst,
                    const_idx: idx as u16,
                });
                dst
            } else {
                // Back-compat: manually-built AST may still use Number(false)
                // for small integers; treat it as Int.
                let i = *n as i64;
                let idx = target.ct_emit_int_const(i);
                let dst = regs.alloc(ip, last_use).expect("out of registers");
                target.ct_emit(Instruction::LoadIntConst {
                    dst,
                    const_idx: idx as u16,
                });
                dst
            }
        }
        Expr::Literal(Literal::Int(i), _) => {
            let idx = target.ct_emit_int_const(*i);
            let dst = regs.alloc(ip, last_use).expect("out of registers");
            target.ct_emit(Instruction::LoadIntConst {
                dst,
                const_idx: idx as u16,
            });
            dst
        }
        Expr::Literal(Literal::BigInt(s), _) => {
            let big = s
                .parse::<num_bigint::BigInt>()
                .expect("BigInt literal must be a valid decimal integer");
            let v = hudhudscript_bytecode::Value16::bigint(big);
            let idx = target.ct_emit_const(v);
            let dst = regs.alloc(ip, last_use).expect("out of registers");
            target.ct_emit(Instruction::LoadConst {
                dst,
                const_idx: idx as u16,
            });
            dst
        }
        Expr::Literal(_, _) => {
            let v = literal_to_value(expr);
            let idx = target.ct_emit_const(v);
            let dst = regs.alloc(ip, last_use).expect("out of registers");
            target.ct_emit(Instruction::LoadConst {
                dst,
                const_idx: idx as u16,
            });
            dst
        }
        Expr::This(_) => {
            if let Some(reg) = target
                .ct_local_reg("self")
                .or_else(|| target.ct_local_reg("this"))
            {
                let dst = regs.alloc(ip, last_use).expect("out of registers");
                target.emit_move(dst, reg);
                dst
            } else {
                let sym = target.ct_intern("this");
                let dst = regs.alloc(ip, last_use).expect("out of registers");
                target.ct_emit(Instruction::LoadGlobal {
                    dst,
                    sym: sym as u16,
                });
                dst
            }
        }
        Expr::Identifier(name, _) => {
            target.ct_track_reference(name);
            // B5: return local register directly, avoid unnecessary Move.
            // ISSUE-2e-optimize: shared top-level symbols live in globals (2e-E),
            // so the local register may be stale after a function call mutates
            // the global.  Emit LoadGlobal to read the canonical value.
            if let Some(reg) = target.ct_local_reg(name) {
                if target.ct_is_top_level() && target.ct_is_shared_top_level(name) {
                    let dst = regs.alloc(ip, last_use).expect("out of registers");
                    let sym = target.ct_intern(name);
                    target.ct_emit(Instruction::LoadGlobal {
                        dst,
                        sym: sym as u16,
                    });
                    dst
                } else {
                    reg
                }
            } else {
                let dst = regs.alloc(ip, last_use).expect("out of registers");
                let sym = target.ct_intern(name);
                target.ct_emit(Instruction::LoadGlobal {
                    dst,
                    sym: sym as u16,
                });
                dst
            }
        }
        Expr::Member {
            object, property, ..
        } => {
            // P2: fast path for .length on typed locals
            if property == "length" {
                if let Expr::Identifier(name, _) = &**object {
                    let obj_type = target.ct_local_type(name);
                    if obj_type == ExprType::Array {
                        let obj_reg = compile_expr_to_reg(target, object, regs);
                        let dst = regs
                            .alloc(target.ct_current_ip(), last_use)
                            .expect("out of registers");
                        target.ct_emit(Instruction::ArrayLen { dst, obj: obj_reg });
                        regs.free_now(obj_reg);
                        return dst;
                    }
                    if obj_type == ExprType::Str {
                        let obj_reg = compile_expr_to_reg(target, object, regs);
                        let dst = regs
                            .alloc(target.ct_current_ip(), last_use)
                            .expect("out of registers");
                        target.ct_emit(Instruction::StringLen { dst, obj: obj_reg });
                        regs.free_now(obj_reg);
                        return dst;
                    }
                }
            }
            let obj_reg = compile_expr_to_reg(target, object, regs);
            let dst = regs
                .alloc(target.ct_current_ip(), last_use)
                .expect("out of registers");
            let prop_sym = target.ct_sym(property);
            target.ct_emit(Instruction::GetProperty {
                dst,
                obj: obj_reg,
                prop_sym: prop_sym.0 as u16,
            });
            regs.free_now(obj_reg);
            dst
        }
        Expr::Array { elements, .. } => {
            // P8: 2-element array without spread → MakeArray2
            if elements.len() == 2 && !elements.iter().any(|e| matches!(e, Expr::Spread { .. })) {
                let a = compile_expr_to_reg(target, &elements[0], regs);
                let b = compile_expr_to_reg(target, &elements[1], regs);
                let dst = regs
                    .alloc(target.ct_current_ip(), last_use)
                    .expect("out of registers");
                target.ct_emit(Instruction::MakeArray2 { dst, a, b });
                regs.free_now(a);
                regs.free_now(b);
                return dst;
            }
            let has_spread = elements.iter().any(|e| matches!(e, Expr::Spread { .. }));
            let dst = regs
                .alloc(target.ct_current_ip(), last_use)
                .expect("out of registers");
            if has_spread {
                target.ct_emit(Instruction::MakeArray { dst, count: 0 });
                for elem in elements {
                    if let Expr::Spread { expr, .. } = elem {
                        let src = compile_expr_to_reg(
                            target,
                            expr,
                            &mut RegAlloc::new_with_base(target.ct_next_local_reg())
                                .expect("out of register zones"),
                        );
                        target.ct_emit(Instruction::SpreadIntoArray { dst, src });
                    } else {
                        let val = compile_expr_to_reg(
                            target,
                            elem,
                            &mut RegAlloc::new_with_base(target.ct_next_local_reg())
                                .expect("out of register zones"),
                        );
                        target.ct_emit(Instruction::ArrayPush { dst, arr: dst, val });
                    }
                }
            } else {
                // ISSUE-9b: pre-allocate array capacity when literal size is known.
                let count = elements.len().min(255) as u16;
                target.ct_emit(Instruction::MakeArray { dst, count });
                for elem in elements {
                    let r = compile_expr_to_reg(target, elem, regs);
                    target.ct_emit(Instruction::ArrayPush {
                        dst,
                        arr: dst,
                        val: r,
                    });
                    regs.free_now(r);
                }
            }
            dst
        }
        Expr::Object { properties, .. } => {
            let has_spread = properties
                .iter()
                .any(|(_, v)| matches!(v, Expr::Spread { .. }));
            let dst = regs
                .alloc(target.ct_current_ip(), last_use)
                .expect("out of registers");
            if has_spread {
                target.ct_emit(Instruction::MakeObject { dst, count: 0 });
                for (key, value) in properties {
                    if let Expr::Spread { expr, .. } = value {
                        let src = compile_expr_to_reg(
                            target,
                            expr,
                            &mut RegAlloc::new_with_base(target.ct_next_local_reg())
                                .expect("out of register zones"),
                        );
                        target.ct_emit(Instruction::SpreadIntoObject { dst, src });
                    } else {
                        let val_reg = compile_expr_to_reg(
                            target,
                            value,
                            &mut RegAlloc::new_with_base(target.ct_next_local_reg())
                                .expect("out of register zones"),
                        );
                        let key_sym = target.ct_sym(key);
                        target.ct_emit(Instruction::SetProperty {
                            dst,
                            obj: dst,
                            val: val_reg,
                            prop_sym: key_sym.0 as u16,
                        });
                    }
                }
            } else {
                target.ct_emit(Instruction::MakeObject {
                    dst,
                    count: properties.len() as u16,
                });
                for (key, value) in properties {
                    let val_reg = compile_expr_to_reg(target, value, regs);
                    let key_sym = target.ct_sym(key);
                    target.ct_emit(Instruction::ObjLitSet {
                        obj: dst,
                        val: val_reg,
                        prop_sym: key_sym.0 as u16,
                    });
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
                        // P3b: try compiler-side inlining BEFORE emitting Call
                        if let Some(chunk) = target.ct_get_function_chunk(name) {
                            let argc = args.len() as u8;
                            let mut arg_regs = Vec::with_capacity(args.len());
                            for arg in args.iter() {
                                let r = compile_expr_to_reg(target, arg, regs);
                                arg_regs.push(r);
                            }
                            // Allocate arg area BEFORE dst (register order = v0.8.157 compat)
                            let opt_first_arg = if argc == 0 {
                                None
                            } else if argc == 1 {
                                Some(arg_regs[0])
                            } else {
                                let first = regs
                                    .alloc_contiguous(
                                        argc,
                                        target.ct_current_ip(),
                                        target.ct_current_ip() + 1,
                                    )
                                    .expect("out of contiguous registers");
                                for (i, &r) in arg_regs.iter().enumerate() {
                                    target.emit_move(first + i as u8, r);
                                    regs.free_now(r);
                                }
                                Some(first)
                            };
                            let dst = regs
                                .alloc(target.ct_current_ip(), last_use)
                                .expect("out of registers");
                            // L1: for 0-arg calls, use dst as first_arg to avoid clobber
                            let first_arg = opt_first_arg.unwrap_or(dst);
                            if crate::optimizer::inline_compile::try_inline_call(
                                target, &chunk, first_arg, argc, dst,
                            ) {
                                if argc == 1 {
                                    regs.free_now(first_arg);
                                }
                                return dst;
                            }
                            // Inline failed: emit normal Call
                            let name_sym = target.ct_sym(name);
                            let idx = target.ct_add_call_payload(name_sym, argc);
                            target.ct_emit(Instruction::Call {
                                dst,
                                payload_idx: idx as u16,
                                first_arg,
                                arg_count: argc,
                            });
                            if argc == 1 {
                                regs.free_now(first_arg);
                            }
                            return dst;
                        }
                        // Original direct Call path (no inline attempt)
                        let argc = args.len() as u8;
                        let mut arg_regs = Vec::with_capacity(args.len());
                        for arg in args.iter() {
                            let r = compile_expr_to_reg(target, arg, regs);
                            arg_regs.push(r);
                        }
                        let call_ip = target.ct_current_ip();
                        // P3-A2: argc==1 uses arg register directly as first_arg, no Move
                        let first_arg = if argc == 1 {
                            arg_regs[0]
                        } else {
                            let first_arg = regs
                                .alloc_contiguous(argc, call_ip, call_ip + 1)
                                .expect("out of contiguous registers");
                            for (i, r) in arg_regs.iter().enumerate() {
                                target.emit_move(first_arg + i as u8, *r);
                                regs.free_now(*r);
                            }
                            first_arg
                        };
                        let name_sym = target.ct_sym(name);
                        let idx = target.ct_add_call_payload(name_sym, argc);
                        let dst = regs
                            .alloc(target.ct_current_ip(), last_use)
                            .expect("out of registers");
                        target.ct_emit(Instruction::Call {
                            dst,
                            payload_idx: idx as u16,
                            first_arg,
                            arg_count: argc,
                        });
                        if argc == 1 {
                            regs.free_now(first_arg);
                        }
                        return dst;
                    }
                }
            }
            // B5: always move compile_expr_complex result to caller's RegAlloc
            // (complex_reg is from a different pool — must be tracked here)
            let complex_reg =
                crate::compiler::expr::compile_complex::compile_expr_complex(target, expr)
                    .expect("compile_complex failed");
            let dst = regs.alloc(ip, last_use).expect("out of registers");
            target.emit_move(dst, complex_reg);
            dst
        }
        Expr::Perform { action, .. } => {
            if let Expr::Call { callee, args, .. } = action.as_ref() {
                if let Expr::Member {
                    object, property, ..
                } = callee.as_ref()
                {
                    if let Expr::Identifier(agent_name, _) = object.as_ref() {
                        let qualified = format!("{}.{}", agent_name, property);
                        let argc = args.len() as u8;
                        let mut arg_regs = Vec::with_capacity(args.len());
                        for arg in args.iter() {
                            let r = compile_expr_to_reg(target, arg, regs);
                            arg_regs.push(r);
                        }
                        let call_ip = target.ct_current_ip();
                        let first_arg = regs
                            .alloc_contiguous(argc, call_ip, call_ip + 1)
                            .expect("out of contiguous registers for perform");
                        for (i, r) in arg_regs.iter().enumerate() {
                            target.emit_move(first_arg + i as u8, *r);
                            regs.free_now(*r);
                        }
                        let name_sym = target.ct_sym(&qualified);
                        let idx = target.ct_add_call_payload(name_sym, argc);
                        let dst = regs
                            .alloc(target.ct_current_ip(), last_use)
                            .expect("out of registers");
                        target.ct_emit(Instruction::Call {
                            dst,
                            payload_idx: idx as u16,
                            first_arg,
                            arg_count: argc,
                        });
                        return dst;
                    }
                }
            }
            let complex_reg =
                crate::compiler::expr::compile_complex::compile_expr_complex(target, expr)
                    .expect("compile_complex failed");
            let dst = regs.alloc(ip, last_use).expect("out of registers");
            target.emit_move(dst, complex_reg);
            dst
        }
        Expr::Unary {
            op, expr: inner, ..
        } => {
            match op {
                UnaryOp::PostIncrement | UnaryOp::PostDecrement => {
                    let imm: i16 = if matches!(op, UnaryOp::PostIncrement) {
                        1
                    } else {
                        -1
                    };
                    if let Expr::Identifier(name, _) = inner.as_ref() {
                        if let Some(reg) = target.ct_local_reg(name) {
                            let dst = regs
                                .alloc(target.ct_current_ip(), last_use)
                                .expect("out of registers");
                            target.emit_move(dst, reg);
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
                            return dst;
                        }
                    }
                    unreachable!("Postfix ++/-- can only be applied to local variables");
                }
                _ => {}
            }
            let src = compile_expr_to_reg(target, inner, regs);
            let dst = regs
                .alloc(target.ct_current_ip(), last_use)
                .expect("out of registers");
            match op {
                UnaryOp::Neg => target.ct_emit(Instruction::Neg { dst, src }),
                UnaryOp::Not => target.ct_emit(Instruction::Not { dst, src }),
                UnaryOp::Plus => {
                    regs.free_now(dst);
                    return src;
                }
                _ => {}
            }
            regs.free_now(src);
            dst
        }
        Expr::Index { object, index, .. } => {
            // P1: specialized index read opcodes based on local variable type.
            // Checks `ct_local_type` before compiling — array gets IndexArray,
            // string gets IndexStringAscii.
            if let Expr::Identifier(name, _) = &**object {
                let obj_type = target.ct_local_type(name);
                if obj_type == ExprType::Array {
                    let obj_reg = compile_expr_to_reg(target, object, regs);
                    let idx_reg = compile_expr_to_reg(target, index, regs);
                    let dst = regs
                        .alloc(target.ct_current_ip(), last_use)
                        .expect("out of registers");
                    target.ct_emit(Instruction::IndexArray {
                        dst,
                        obj: obj_reg,
                        idx: idx_reg,
                    });
                    regs.free_now(obj_reg);
                    regs.free_now(idx_reg);
                    return dst;
                }
                if obj_type == ExprType::Str {
                    let obj_reg = compile_expr_to_reg(target, object, regs);
                    let idx_reg = compile_expr_to_reg(target, index, regs);
                    let dst = regs
                        .alloc(target.ct_current_ip(), last_use)
                        .expect("out of registers");
                    target.ct_emit(Instruction::IndexStringAscii {
                        dst,
                        obj: obj_reg,
                        idx: idx_reg,
                    });
                    regs.free_now(obj_reg);
                    regs.free_now(idx_reg);
                    return dst;
                }
            }
            // Fallback: generic Index
            let obj_reg = compile_expr_to_reg(target, object, regs);
            let idx_reg = compile_expr_to_reg(target, index, regs);
            let dst = regs
                .alloc(target.ct_current_ip(), last_use)
                .expect("out of registers");
            target.ct_emit(Instruction::Index {
                dst,
                obj: obj_reg,
                idx: idx_reg,
            });
            regs.free_now(obj_reg);
            regs.free_now(idx_reg);
            dst
        }
        Expr::Ternary {
            condition,
            true_expr,
            false_expr,
            ..
        } => {
            let cond_reg = compile_expr_to_reg(target, condition, regs);
            let dst = regs
                .alloc(target.ct_current_ip(), last_use)
                .expect("out of registers");
            let jump_to_else = target.ct_current_ip();
            target.ct_emit(Instruction::JumpIfFalse {
                src: cond_reg,
                offset: 0,
            });
            let true_reg = compile_expr_to_reg(target, true_expr, regs);
            target.emit_move(dst, true_reg);
            regs.free_now(true_reg);
            let jump_to_end = target.ct_current_ip();
            target.ct_emit(Instruction::Jump(0));
            let else_ip = target.ct_current_ip();
            target.ct_patch(
                jump_to_else,
                Instruction::JumpIfFalse {
                    src: cond_reg,
                    offset: jump_off(jump_to_else, else_ip) as i16,
                },
            );
            regs.free_now(cond_reg);
            let false_reg = compile_expr_to_reg(target, false_expr, regs);
            target.emit_move(dst, false_reg);
            regs.free_now(false_reg);
            let end_ip = target.ct_current_ip();
            target.ct_patch(
                jump_to_end,
                Instruction::Jump(jump_off(jump_to_end, end_ip)),
            );
            dst
        }
        Expr::OptionalMember {
            object, property, ..
        } => crate::compiler::expr::compile_reg_binary::compile_optional_member(
            target, object, property, regs, last_use,
        ),
        Expr::Binary {
            left, op, right, ..
        } => crate::compiler::expr::compile_reg_binary::compile_binary(
            target, left, op, right, regs, last_use,
        ),
        _ => {
            crate::compiler::expr::compile_complex::compile_expr_complex(target, expr)
                .expect("compile_complex failed");
            let dst = regs.alloc(ip, last_use).expect("out of registers");
            target.emit_move(dst, 255);
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
