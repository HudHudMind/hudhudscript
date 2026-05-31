use super::*;
use super::compile_complex_extra::compile_expr_complex_inner_extra;

pub(crate) fn compile_expr_complex(
    target: &mut impl CompileTarget,
    expr: &Expr,
) -> CompileResult<u8> {
    let ip = target.ct_current_ip();
    let mut regs = RegAlloc::new_with_base(target.ct_next_local_reg())?;
    compile_expr_complex_inner(target, expr, &mut regs, ip)
}

fn compile_expr_complex_inner(
    target: &mut impl CompileTarget,
    expr: &Expr,
    regs: &mut RegAlloc,
    base_ip: usize,
) -> CompileResult<u8> {
    #[allow(unreachable_patterns)]
    match expr {
        Expr::Array { elements, .. } => {
            let has_spread = elements.iter().any(|e| matches!(e, Expr::Spread { .. }));
            let dst = regs.alloc(base_ip, base_ip + 255).expect("out of registers");
            if has_spread {
                // Build array incrementally to handle spread elements (#945)
                { let tr = crate::compiler::regalloc::temp_reg(); target.ct_emit(Instruction::MakeArray { dst: tr, count: 0 }); target.ct_emit(Instruction::Move { dst: 255, src: tr }); }
                for elem in elements {
                    if let Expr::Spread { expr, .. } = elem {
                        let r = compile_expr_to_reg(target, expr, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                        target.ct_emit(Instruction::SpreadIntoArray { dst: 255, src: r });
                    } else {
                        let mut regs = RegAlloc::new_with_base(target.ct_next_local_reg())?;
                        let ip = target.ct_current_ip();
                        // Pop current array from stack, then push back after mutation
                        let arr_reg = crate::compiler::regalloc::temp_reg();
                        target.ct_emit(Instruction::Move { dst: arr_reg, src: 255 });
                        let val_reg = compile_expr_to_reg(target, elem, &mut regs);
                        let dst_reg = regs.alloc(ip, ip + 255).expect("out of registers");
                        target.ct_emit(Instruction::ArrayPush { dst: dst_reg, arr: arr_reg, val: val_reg });
                        target.ct_emit(Instruction::Move { dst: 255, src: dst_reg });
                    }
                }
            } else {
                // Register-based: build array incrementally with ArrayPush
                { let tr = crate::compiler::regalloc::temp_reg(); target.ct_emit(Instruction::MakeArray { dst: tr, count: 0 }); target.ct_emit(Instruction::Move { dst: 255, src: tr }); }
                for elem in elements {
                    let r = compile_expr_to_reg(target, elem, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                    let arr_reg = crate::compiler::regalloc::temp_reg();
                    target.ct_emit(Instruction::Move { dst: arr_reg, src: 255 });
                    let dst_reg = crate::compiler::regalloc::temp_reg();
                    target.ct_emit(Instruction::ArrayPush { dst: dst_reg, arr: arr_reg, val: r });
                    target.ct_emit(Instruction::Move { dst: 255, src: dst_reg });
                }
            }
        }

        Expr::Object { properties, .. } => {
            // Gap 1 — the parser emits object-literal spread entries as
            // `("__spread__N", Expr::Spread { expr })`.  When any spread
            // is present, build the object incrementally using
            // MakeObject(0) + SetProperty / SpreadIntoObject so the
            // unpacked source fields land as real keys (and the
            // "__spread__N" placeholder never leaks into the live
            // object).  No-spread path preserves the fast MakeObject(n)
            // encoding.
            let has_spread = properties
                .iter()
                .any(|(_, v)| matches!(v, Expr::Spread { .. }));
            if has_spread {
                { let tr = crate::compiler::regalloc::temp_reg(); target.ct_emit(Instruction::MakeObject { dst: tr, count: 0 }); target.ct_emit(Instruction::Move { dst: 255, src: tr }); }
                for (key, value) in properties {
                    if let Expr::Spread { expr, .. } = value {
                        let r = compile_expr_to_reg(target, expr, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                        target.ct_emit(Instruction::SpreadIntoObject { dst: 255, src: r });
                    } else {
                        let mut regs = RegAlloc::new_with_base(target.ct_next_local_reg())?;
                        let ip = target.ct_current_ip();
                        // Pop current object from stack, then push back after mutation
                        let obj_reg = crate::compiler::regalloc::temp_reg();
                        target.ct_emit(Instruction::Move { dst: obj_reg, src: 255 });
                        let val_reg = compile_expr_to_reg(target, value, &mut regs);
                        let dst_reg = regs.alloc(ip, ip + 255).expect("out of registers");
                        let key_sym = target.ct_sym(key);
                        target.ct_emit(Instruction::SetProperty { dst: dst_reg, obj: obj_reg, val: val_reg, prop_sym: key_sym.0 as u16 });
                        target.ct_emit(Instruction::Move { dst: 255, src: dst_reg });
                    }
                }
            } else {
                for (key, value) in properties {
                    let key_idx = target.ct_emit_const(Value16::string(key.clone()));
                    { let tr = crate::compiler::regalloc::temp_reg(); target.ct_emit(Instruction::LoadConst { dst: tr, const_idx: key_idx as u16 }); target.ct_emit(Instruction::Move { dst: 255, src: tr }); }
                    {
                        let r = compile_expr_to_reg(target, value, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                        target.ct_emit(Instruction::Move { dst: 255, src: r });
                    }
                }
                { let tr = crate::compiler::regalloc::temp_reg(); target.ct_emit(Instruction::MakeObject { dst: tr, count: properties.len() as u16 }); target.ct_emit(Instruction::Move { dst: 255, src: tr }); }
            }
        }

        Expr::Index { object, index, .. } => {
            // ISSUE-2: IndexFast for slot-mapped local array + local index.
            let fast_regs = if let (Expr::Identifier(arr_name, _), Expr::Identifier(idx_name, _)) =
                (object.as_ref(), index.as_ref())
            {
                match (target.ct_local_reg(arr_name), target.ct_local_reg(idx_name)) {
                    (Some(a), Some(i)) => Some((a, i)),
                    _ => None,
                }
            } else {
                None
            };
            if let Some((arr_reg, idx_reg)) = fast_regs {
                // K1-1: Register-based fast path — locals are already in registers.
                let tr_dst = crate::compiler::regalloc::temp_reg();
                target.ct_emit(Instruction::Index { dst: tr_dst, obj: arr_reg, idx: idx_reg });
                target.ct_emit(Instruction::Move { dst: 255, src: tr_dst });
            } else {
                {
                        let r = compile_expr_to_reg(target, object, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                        target.ct_emit(Instruction::Move { dst: 255, src: r });
                    }
                {
                        let r = compile_expr_to_reg(target, index, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                        target.ct_emit(Instruction::Move { dst: 255, src: r });
                    }
                let tr_idx = crate::compiler::regalloc::temp_reg();
                let tr_obj = crate::compiler::regalloc::temp_reg();
                let tr_dst = crate::compiler::regalloc::temp_reg();
                target.ct_emit(Instruction::Move { dst: tr_idx, src: 255 });
                target.ct_emit(Instruction::Move { dst: tr_obj, src: 255 });
                target.ct_emit(Instruction::Index { dst: tr_dst, obj: tr_obj, idx: tr_idx });
                target.ct_emit(Instruction::Move { dst: 255, src: tr_dst });
            }
        }

        Expr::Member {
            object, property, ..
        } => {
            // Issue #345: ClassName.staticMember → GetStatic
            if let Expr::Identifier(class_name, _) = &**object {
                if target.ct_is_known_class(class_name) {
                    let cls_sym = target.ct_sym(class_name);
                    let prop_sym = target.ct_sym(property);
                    let idx = target.ct_add_two_sym_payload(cls_sym.0, prop_sym.0);
                    target.ct_emit(Instruction::GetStatic(idx));
                } else {
                    let mut regs = RegAlloc::new_with_base(target.ct_next_local_reg())?;
                    let ip = target.ct_current_ip();
                    let obj_reg = compile_expr_to_reg(target, object, &mut regs);
                    let dst_reg = regs.alloc(ip, ip + 255).expect("out of registers");
                    let prop_sym = target.ct_sym(property);
                    target.ct_emit(Instruction::GetProperty { dst: dst_reg, obj: obj_reg, prop_sym: prop_sym.0 as u16 });
                    target.ct_emit(Instruction::Move { dst: 255, src: dst_reg });
                }
            } else {
                let mut regs = RegAlloc::new_with_base(target.ct_next_local_reg())?;
                let ip = target.ct_current_ip();
                let obj_reg = compile_expr_to_reg(target, object, &mut regs);
                let dst_reg = regs.alloc(ip, ip + 255).expect("out of registers");
                let prop_sym = target.ct_sym(property);
                target.ct_emit(Instruction::GetProperty { dst: dst_reg, obj: obj_reg, prop_sym: prop_sym.0 as u16 });
                target.ct_emit(Instruction::Move { dst: 255, src: dst_reg });
            }
        }

        Expr::Call { callee, args, span } => {
            // Gap 1 — if any argument is a spread expression (`f(...arr)`),
            // the argument count is only known at runtime.  Build the
            // argument list as an Array on the stack and dispatch via the
            // `CallSpread` / `MethodCallSpread` instructions which pop the
            // array and use its length as the arg count.
            let has_spread = args.iter().any(|a| matches!(a, Expr::Spread { .. }));

            if has_spread {
                // Materialise the arguments as a single Array at runtime.
                { let tr = crate::compiler::regalloc::temp_reg(); target.ct_emit(Instruction::MakeArray { dst: tr, count: 0 }); target.ct_emit(Instruction::Move { dst: 255, src: tr }); }
                for arg in args {
                    if let Expr::Spread { expr, .. } = arg {
                        let r = compile_expr_to_reg(target, expr, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                        target.ct_emit(Instruction::SpreadIntoArray { dst: 255, src: r });
                    } else {
                        let mut regs = RegAlloc::new_with_base(target.ct_next_local_reg())?;
                        let ip = target.ct_current_ip();
                        // arr_reg: re-read from stack (previously pushed MakeArray result)
                        let arr_reg = crate::compiler::regalloc::temp_reg();
                        target.ct_emit(Instruction::Move { dst: arr_reg, src: 255 });
                        let val_reg = compile_expr_to_reg(target, arg, &mut regs);
                        let dst_reg = regs.alloc(ip, ip + 255).expect("out of registers");
                        target.ct_emit(Instruction::ArrayPush { dst: dst_reg, arr: arr_reg, val: val_reg });
                        target.ct_emit(Instruction::Move { dst: 255, src: dst_reg });
                    }
                }
                match &**callee {
                    Expr::Identifier(name, _) => {
                        let name_sym = target.ct_sym(name);
                        target.ct_emit(Instruction::CallSpread(name_sym));
                    }
                    Expr::Member {
                        object, property, ..
                    } => {
                        if matches!(&**object, Expr::Identifier(n, _) if n == "super") {
                            return Err(compile_codes::unsupported_feature_at(
                                "spread arguments are not supported in super-method calls"
                                    .to_string(),
                                span_pos(span),
                            ));
                        }
                        // Receiver goes on top of the argument array so
                        // MethodCallSpread can pop it first.
                        // FIX: stash receiver in a safe register so nested
                        // MethodCall arguments cannot clobber reg255.
                        let mut outer_regs = RegAlloc::new_with_base(target.ct_next_local_reg())?;
                        let receiver_reg = compile_expr_to_reg(target, object, &mut outer_regs);
                        let prop_sym = target.ct_sym(property);
                        target.ct_emit(Instruction::Move { dst: 255, src: receiver_reg });
                        target.ct_emit(Instruction::MethodCallSpread(prop_sym));
                        if let Some(root_name) = root_var_name(object) {
                            let root_sym = target.ct_sym(&root_name);
                            target.ct_emit(Instruction::WriteBackReceiver(root_sym));
                        }
                    }
                    _ => {
                        return Err(compile_codes::unsupported_feature_at(
                            "Only simple and member function calls supported".to_string(),
                            span_pos(span),
                        ));
                    }
                }
            } else {
                for arg in args {
                    let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, arg, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                    target.ct_emit(Instruction::Move { dst: 255, src: r });
                }
                match &**callee {
                    Expr::Identifier(name, _) => {
                        if name == "super" {
                            let argc = args.len() as u8;
                            let first_arg = crate::compiler::regalloc::temp_reg();
                            for (i, arg) in args.iter().enumerate() {
                                let r = compile_expr_to_reg(target, arg, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                                target.ct_emit(Instruction::Move { dst: first_arg + i as u8, src: r });
                            }
                            let ctor_sym = target.ct_sym("constructor");
                            let idx = target.ct_add_call_payload(ctor_sym, argc);
                            target.ct_emit(Instruction::SuperCall { dst: 255, payload_idx: idx as u16, first_arg, arg_count: argc });
                        } else if target.ct_is_known_generator(name) {
                            // Issue #667: Emit MakeGenerator for generator function calls
                            let name_sym = target.ct_sym(name);
                            let argc = args.len() as u8;
                            let first_arg = crate::compiler::regalloc::temp_reg();
                            for (i, arg) in args.iter().enumerate() {
                                let r = compile_expr_to_reg(target, arg, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                                target.ct_emit(Instruction::Move { dst: first_arg + i as u8, src: r });
                            }
                            let idx = target.ct_add_call_payload(name_sym, argc);
                            target.ct_emit(Instruction::MakeGenerator { payload_idx: idx as u16, first_arg, arg_count: argc });
                        } else {
                            let name_sym = target.ct_sym(name);
                            let idx = target.ct_add_call_payload(name_sym, args.len() as u8);
                            let tr = crate::compiler::regalloc::temp_reg();
                            target.ct_emit(Instruction::Call { dst: tr, payload_idx: idx as u16, first_arg: 0, arg_count: args.len() as u8 });
                            target.ct_emit(Instruction::Move { dst: 255, src: tr });
                        }
                    }
                    Expr::Member {
                        object, property, ..
                    } => {
                        // Issue #345: super.method(args) → SuperCall
                        if matches!(&**object, Expr::Identifier(n, _) if n == "super") {
                            let argc = args.len() as u8;
                            let first_arg = crate::compiler::regalloc::temp_reg();
                            for (i, arg) in args.iter().enumerate() {
                                let r = compile_expr_to_reg(target, arg, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                                target.ct_emit(Instruction::Move { dst: first_arg + i as u8, src: r });
                            }
                            let prop_sym = target.ct_sym(property);
                            let idx = target.ct_add_call_payload(prop_sym, argc);
                            target.ct_emit(Instruction::SuperCall { dst: 255, payload_idx: idx as u16, first_arg, arg_count: argc });
                        } else {
                            // FIX: stash receiver in a safe register so nested
                            // MethodCall arguments cannot clobber reg255.
                            let mut outer_regs = RegAlloc::new_with_base(target.ct_next_local_reg())?;
                            let receiver_reg = compile_expr_to_reg(target, object, &mut outer_regs);
                            let stash = crate::compiler::regalloc::temp_reg();
                            target.ct_emit(Instruction::Move { dst: stash, src: receiver_reg });
                            let argc = args.len() as u8;
                            let first_arg = crate::compiler::regalloc::temp_reg();
                            for (i, arg) in args.iter().enumerate() {
                                let r = compile_expr_to_reg(target, arg, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                                target.ct_emit(Instruction::Move { dst: first_arg + i as u8, src: r });
                            }
                            // OPT: array.push(x) → ArrayPush; string.indexOf/contains → static opcodes
                            if property == "push" && argc == 1 {
                                target.ct_emit(Instruction::ArrayPush { dst: stash, arr: stash, val: first_arg });
                                target.ct_emit(Instruction::Move { dst: 255, src: stash });
                            } else if property == "indexOf" && argc == 1 {
                                target.ct_emit(Instruction::StringIndexOf { dst: stash, haystack: stash, needle: first_arg });
                                target.ct_emit(Instruction::Move { dst: 255, src: stash });
                            } else if property == "contains" && argc == 1 {
                                target.ct_emit(Instruction::StringContains { dst: stash, haystack: stash, needle: first_arg });
                                target.ct_emit(Instruction::Move { dst: 255, src: stash });
                            } else {
                                let prop_sym = target.ct_sym(property);
                                let idx = target.ct_add_call_payload(prop_sym, argc);
                                target.ct_emit(Instruction::Move { dst: 255, src: stash });
                                target.ct_emit(Instruction::MethodCall { dst: 255, obj: 255, payload_idx: idx as u16, first_arg, arg_count: argc });
                            }
                            // Bug 4: this.call() implicitly references 'provider'
                            if property == "call" && root_var_name(object).as_deref() == Some("this") {
                                target.ct_track_reference("provider");
                            }
                            if let Some(root_name) = root_var_name(object) {
                                // Mutation writeback for BOTH:
                                //   (a) array mutating methods (push/pop/shift/…)
                                //       — VM's call_array_method stashes the modified
                                //       array in last_instance_mutation.
                                //   (b) user class methods that mutate `this` —
                                //       VM captures post-call `this` the same way.
                                //
                                // In both cases WriteBackReceiver flushes the slot
                                // back to the receiver variable without disturbing
                                // the method return value on the stack. No-op when
                                // the slot is empty (non-mutating receiver). Parity
                                // with the interpreter (#P1-5 + Kural 2).
                                let root_sym = target.ct_sym(&root_name);
                                target.ct_emit(Instruction::WriteBackReceiver(root_sym));
                            }
                        }
                    }
                    Expr::Index { object, index, .. } => {
                        // funcs[i]() — closure stored in array/object
                        let argc = args.len() as u8;
                        let first_arg = crate::compiler::regalloc::temp_reg();
                        for (i, arg) in args.iter().enumerate() {
                            let r = compile_expr_to_reg(target, arg, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                            target.ct_emit(Instruction::Move { dst: first_arg + i as u8, src: r });
                        }
                        let mut ri = RegAlloc::new_with_base(target.ct_next_local_reg())?;
                        let obj_reg = compile_expr_to_reg(target, object, &mut ri);
                        let idx_reg = compile_expr_to_reg(target, index, &mut ri);
                        target.ct_emit(Instruction::Index { dst: 255, obj: obj_reg, idx: idx_reg });
                        let temp_name = "__call_indexed";
                        let temp_sym = target.ct_intern(temp_name);
                        target.ct_emit(Instruction::StoreGlobal { src: 255, sym: temp_sym as u16 });
                        let call_idx = target.ct_add_call_payload(hudhudscript_bytecode::SymId(temp_sym), argc);
                        let tr = crate::compiler::regalloc::temp_reg();
                        target.ct_emit(Instruction::Call { dst: tr, payload_idx: call_idx as u16, first_arg, arg_count: argc });
                        target.ct_emit(Instruction::Move { dst: 255, src: tr });
                    }
                    _ => {
                        return Err(compile_codes::unsupported_feature_at(
                            "Only simple and member function calls supported".to_string(),
                            span_pos(span),
                        ));
                    }
                }
            }
        }

        Expr::ArrowFunction {
            params,
            body: arrow_body,
            is_async,
            ..
        } => {
            target.ct_compile_arrow(params, arrow_body, *is_async)?;
        }

        Expr::New {
            class_name, args, ..
        } => {
            let argc = args.len() as u8;
            let first_arg = crate::compiler::regalloc::temp_reg();
            for (i, arg) in args.iter().enumerate() {
                let r = compile_expr_to_reg(target, arg, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                target.ct_emit(Instruction::Move { dst: first_arg + i as u8, src: r });
            }
            let cls_sym = target.ct_sym(class_name);
            let idx = target.ct_add_call_payload(cls_sym, argc);
            target.ct_emit(Instruction::NewInstance { payload_idx: idx as u16, first_arg, arg_count: argc });
        }

        Expr::This(_) => {
            target.ct_emit_load_var("this");
        }

        Expr::TemplateString { parts, .. } => {
            let mut result_reg: Option<u8> = None;
            for part in parts {
                let part_reg = match part {
                    hudhudscript_ast::TemplateStringPart::Text(s) => {
                        let idx = target.ct_emit_const(Value16::string(s.clone()));
                        let tr = crate::compiler::regalloc::temp_reg();
                        target.ct_emit(Instruction::LoadConst { dst: tr, const_idx: idx as u16 });
                        tr
                    }
                    hudhudscript_ast::TemplateStringPart::Interpolation(expr) => {
                        compile_expr_to_reg(target, expr, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?)
                    }
                };
                match result_reg {
                    None => result_reg = Some(part_reg),
                    Some(r) => {
                        let dst = crate::compiler::regalloc::temp_reg();
                        target.ct_emit(Instruction::StrCat { dst, src1: r, src2: part_reg });
                        result_reg = Some(dst);
                    }
                }
            }
            let final_reg = result_reg.unwrap_or_else(|| {
                let idx = target.ct_emit_const(Value16::string(String::new()));
                let tr = crate::compiler::regalloc::temp_reg();
                target.ct_emit(Instruction::LoadConst { dst: tr, const_idx: idx as u16 });
                tr
            });
            target.ct_emit(Instruction::Move { dst: 255, src: final_reg });
        }

        _ => { compile_expr_complex_inner_extra(target, expr, regs, base_ip)?; }
    }
    Ok(255)
}
