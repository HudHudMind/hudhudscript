use super::*;

pub(super) fn compile_stmt_part3(
    target: &mut impl CompileTarget,
    stmt: &Stmt,
) -> CompileResult<()> {
    match stmt {
        Stmt::VarDecl(var_decl) => {
            let reg = if let Some(init) = &var_decl.initializer {
                crate::compiler::expr::compile_reg::compile_expr_to_reg(
                    target, init, &mut crate::compiler::regalloc::RegAlloc::new_with_base(target.ct_next_local_reg()).expect("out of register zones"),
                )
            } else {
                let idx = target.ct_emit_const(Value16::null());
                let tr = crate::compiler::regalloc::temp_reg();
                target.ct_emit(Instruction::LoadConst { dst: tr, const_idx: idx as u16 });
                tr
            };
            target.ct_declare_local(&var_decl.name, var_decl.is_const)?;
            if var_decl.is_const {
                if let Some(local_reg) = target.ct_local_reg(&var_decl.name) {
                    target.ct_emit(Instruction::Move { dst: local_reg, src: reg });
                }
                let sym = target.ct_intern(&var_decl.name);
                target.ct_emit(Instruction::StoreConst { src: reg, sym: sym as u16 });
            } else if let Some(local_reg) = target.ct_local_reg(&var_decl.name) {
                target.ct_emit(Instruction::Move { dst: local_reg, src: reg });
                if target.ct_is_top_level() && target.ct_is_shared_top_level(&var_decl.name) {
                    let sym = target.ct_intern(&var_decl.name);
                    target.ct_emit(Instruction::DeclGlobal { src: reg, sym: sym as u16 });
                }
            } else {
                let sym = target.ct_intern(&var_decl.name);
                target.ct_emit(Instruction::StoreGlobal { src: reg, sym: sym as u16 });
            }
        }

        // Issue #246 / #339: MCP server — store all config fields
        Stmt::McpServer(mcp_decl) => {
            target.ct_compile_mcp_server(mcp_decl)?;
        }

        // Issue #250: SOP statements
        Stmt::Spawn {
            subject_name, args, ..
        } => {
            let name_sym = target.ct_sym(subject_name);
            let argc = args.len() as u8;
            let first_arg = crate::compiler::regalloc::temp_reg();
            for (i, arg) in args.iter().enumerate() {
                let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, arg, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                target.ct_emit(Instruction::Move { dst: first_arg + i as u8, src: r });
            }
            let idx = target.ct_add_call_payload(name_sym, argc);
            target.ct_emit(Instruction::Spawn { payload_idx: idx as u16, first_arg, arg_count: argc });
            target.ct_emit_store_var(subject_name);
        }

        Stmt::Despawn { name, .. } => {
            use hudhudscript_ast::Expr;
            let ident_expr = Expr::Identifier(name.clone(), hudhudscript_ast::Span::default());
            let reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, &ident_expr, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            target.ct_emit(Instruction::Despawn { reg });
        }

        Stmt::Send {
            message,
            target: send_target,
            ..
        } => {
            let msg_r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, message, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            let tgt_r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, send_target, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            target.ct_emit(Instruction::Send { message: msg_r, target: tgt_r });
        }

        Stmt::Receive {
            variable, source, ..
        } => {
            let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, source, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            let var_sym = target.ct_sym(variable);
            target.ct_emit(Instruction::Receive { var_sym_idx: var_sym.0 as u16, src: r });
        }

        Stmt::Require { condition, .. } => {
            let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, condition, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            target.ct_emit(Instruction::Require { src: r });
        }

        Stmt::Perform { action, .. } => {
            // compile_perform_to_reg emits Instruction::Call directly;
            // no separate Perform instruction needed
            let _r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, action, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
        }

        // Issue #251: RAG statements
        Stmt::Remember {
            content,
            store_name,
            ..
        } => {
            let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, content, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            let store_sym = store_name.as_ref().map(|s| target.ct_sym(s));
            let idx = target.ct_add_opt_sym_payload(store_sym);
            target.ct_emit(Instruction::Remember { store_idx: idx as u16, src: r });
        }

        Stmt::Recall {
            query, store_name, ..
        } => {
            let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, query, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            let store_sym = store_name.as_ref().map(|s| target.ct_sym(s));
            let idx = target.ct_add_opt_sym_payload(store_sym);
            target.ct_emit(Instruction::Recall { store_idx: idx as u16, src: r, dst: r });
        }

        Stmt::Forget {
            target: forget_target,
            store_name,
            ..
        } => {
            let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, forget_target, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            let store_sym = store_name.as_ref().map(|s| target.ct_sym(s));
            let idx = target.ct_add_opt_sym_payload(store_sym);
            target.ct_emit(Instruction::Forget { store_idx: idx as u16, src: r });
        }

        // Destructuring declaration — Issue #668
        Stmt::Destructure {
            pattern,
            value,
            is_const,
            ..
        } => {
            {
            let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, value, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            target.ct_emit(Instruction::Move { dst: 255, src: r });
        }
            target.ct_compile_destructure_pattern(pattern, *is_const)?;
        }

        // Trait declarations — Issue #982: record method signatures for SOP enforcement
        Stmt::Trait { name, methods, .. } => {
            target.ct_register_trait(name, methods);
        }

        // All Decl variants
        Stmt::Decl(decl) => {
            target.ct_compile_decl(decl)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}
