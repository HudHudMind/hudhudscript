//! Call, Perform and Recall expression compilation to registers (Constitution H.5).

use super::*;
use crate::compiler::regalloc;

pub(crate) fn compile_perform(
    target: &mut impl CompileTarget,
    action: &Expr,
    regs: &mut regalloc::RegAlloc,
    ip: usize,
    last_use: usize,
) -> u8 {
    if let Expr::Call { callee, args, .. } = action {
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
    let complex_reg = crate::compiler::expr::compile_complex::compile_expr_complex(
        target,
        &Expr::Perform {
            action: Box::new(action.clone()),
            span: action.span(),
        },
    )
    .expect("compile_complex failed");
    let dst = regs.alloc(ip, last_use).expect("out of registers");
    target.emit_move(dst, complex_reg);
    dst
}

pub(crate) fn compile_recall(
    target: &mut impl CompileTarget,
    query: &Expr,
    store_name: &Option<String>,
    regs: &mut regalloc::RegAlloc,
    last_use: usize,
) -> u8 {
    let r = compile_expr_to_reg(target, query, regs);
    let store_sym = store_name.as_ref().map(|s| target.ct_sym(s));
    let idx = target.ct_add_opt_sym_payload(store_sym);
    let dst = regs
        .alloc(target.ct_current_ip(), last_use)
        .expect("out of registers");
    target.ct_emit(Instruction::Recall {
        store_idx: idx as u16,
        src: r,
        dst,
    });
    regs.free_now(r);
    dst
}

pub(crate) fn compile_call(
    target: &mut impl CompileTarget,
    callee: &Expr,
    args: &[Expr],
    regs: &mut regalloc::RegAlloc,
    ip: usize,
    last_use: usize,
    full_expr: &Expr,
) -> u8 {
    let has_spread = args.iter().any(|a| matches!(a, Expr::Spread { .. }));
    if !has_spread {
        if let Expr::Identifier(name, _) = callee {
            if name != "super" && !target.ct_is_known_generator(name) {
                // P3b: try compiler-side inlining BEFORE emitting Call
                if let Some(chunk) = target.ct_get_function_chunk(name) {
                    let argc = args.len() as u8;
                    let mut arg_regs = Vec::with_capacity(args.len());
                    for arg in args.iter() {
                        let r = compile_expr_to_reg(target, arg, regs);
                        arg_regs.push(r);
                    }
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
                    let first_arg = opt_first_arg.unwrap_or(dst);
                    if crate::optimizer::inline_compile::try_inline_call(
                        target, &chunk, first_arg, argc, dst,
                    ) {
                        if argc == 1 {
                            regs.free_now(first_arg);
                        }
                        return dst;
                    }
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
                let argc = args.len() as u8;
                let mut arg_regs = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    let r = compile_expr_to_reg(target, arg, regs);
                    arg_regs.push(r);
                }
                let call_ip = target.ct_current_ip();
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
    let complex_reg = crate::compiler::expr::compile_complex::compile_expr_complex(target, full_expr)
        .expect("compile_complex failed");
    let dst = regs.alloc(ip, last_use).expect("out of registers");
    target.emit_move(dst, complex_reg);
    dst
}

pub(crate) fn compile_array(
    target: &mut impl CompileTarget,
    elements: &[Expr],
    regs: &mut regalloc::RegAlloc,
    last_use: usize,
) -> u8 {
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

pub(crate) fn compile_object(
    target: &mut impl CompileTarget,
    properties: &[(String, Expr)],
    regs: &mut regalloc::RegAlloc,
    last_use: usize,
) -> u8 {
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
