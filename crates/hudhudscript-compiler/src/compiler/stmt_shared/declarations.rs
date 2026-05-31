use super::*;

pub(super) fn compile_stmt_part2(
    target: &mut impl CompileTarget,
    stmt: &Stmt,
) -> CompileResult<()> {
    match stmt {
        Stmt::Break { .. } => {
            target.ct_emit(Instruction::Break);
        }

        Stmt::Continue { .. } => {
            target.ct_emit(Instruction::Continue);
        }

        // Issue #244: switch
        Stmt::Switch {
            value,
            cases,
            default,
            ..
        } => {
            let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, value, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            target.ct_emit(Instruction::Move { dst: 255, src: r });
            let mut end_jumps: Vec<usize> = Vec::new();
            for case in cases {
                let case_reg = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, &case.value, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
                let cmp_reg = crate::compiler::regalloc::temp_reg();
                target.ct_emit(Instruction::IntCmp { dst: cmp_reg, src1: 255, src2: case_reg, op: 4 });
                let skip = target.ct_current_ip();
                target.ct_emit(Instruction::JumpIfFalse { src: cmp_reg, offset: 0 });
                for s in &case.body {
                    compile_stmt_shared(target, s)?;
                }
                let jmp = target.ct_current_ip();
                target.ct_emit(Instruction::Jump(0));
                end_jumps.push(jmp);
                let next = target.ct_current_ip();
                target.ct_patch(skip, Instruction::JumpIfFalse { src: cmp_reg, offset: jump_off(skip, next) as i16 });
            }
            // Default case
            if let Some(default_body) = default {
                for s in default_body {
                    compile_stmt_shared(target, s)?;
                }
            }
            let end = target.ct_current_ip();
            for jmp in end_jumps {
                target.ct_patch(jmp, Instruction::Jump(jump_off(jmp, end)));
            }
        }

        // Issue #244: try-catch-finally
        //
        // Layout WITHOUT finally (unchanged):
        //   TryBegin(catch_ip)
        //   <try body>
        //   TryEnd
        //   Jump(end_ip)
        //   catch_ip:
        //     StoreVar(e) | Pop
        //     <catch body>
        //   end_ip:
        //
        // Layout WITH finally (Gap 2 — interpreter parity):
        //   FinallyBegin(finally_ip)    # abrupt completion (Return/Throw) inside
        //                              # the try/catch routes through finally
        //   TryBegin(catch_ip)
        //   <try body>
        //   TryEnd
        //   Jump(finally_ip)           # normal try completion → run finally
        //   catch_ip:
        //     StoreVar(e) | Pop
        //     <catch body>
        //     Jump(finally_ip)         # normal catch completion → run finally
        //   finally_ip:
        //     FinallyEnd               # remove the finally frame before body
        //     <finally body>
        //     FinallyExit(end_ip)       # dispatch pending Return/Throw, else jump
        //   end_ip:
        Stmt::Try {
            try_block,
            catch_clause,
            finally_block,
            ..
        } => {
            let has_finally = finally_block.is_some();

            // Reserve FinallyBegin slot (patched once finally_ip is known).
            let finally_start_slot = if has_finally {
                let slot = target.ct_current_ip();
                target.ct_emit(Instruction::FinallyBegin(0));
                Some(slot)
            } else {
                None
            };

            let try_start = target.ct_current_ip();
            target.ct_emit(Instruction::TryBegin(0));
            compile_stmt_shared(target, try_block)?;
            target.ct_emit(Instruction::TryEnd);

            // Jump over the catch clause.  With finally, we jump to the finally
            // entry (patched later); without finally, we jump to the end.
            let jump_after_try = target.ct_current_ip();
            target.ct_emit(Instruction::Jump(0));

            // Catch block (or Pop if none).
            let catch_start = target.ct_current_ip();
            target.ct_patch(
                try_start,
                Instruction::TryBegin(jump_off(try_start, catch_start)),
            );
            if let Some(catch) = catch_clause {
                target.ct_declare_local(&catch.param, false)?;
                target.ct_emit_store_var(&catch.param);
                compile_stmt_shared(target, &catch.body)?;
            } else {
            }

            // End of catch — jump to finally entry (if any) or fall through.
            let jump_after_catch = if has_finally {
                let pos = target.ct_current_ip();
                target.ct_emit(Instruction::Jump(0));
                Some(pos)
            } else {
                None
            };

            if let Some(finally_stmt) = finally_block {
                let finally_ip = target.ct_current_ip();
                // Backpatch the FinallyBegin with the real finally entry IP.
                let fslot = finally_start_slot.expect("has_finally implies slot was reserved");
                target.ct_patch(fslot, Instruction::FinallyBegin(jump_off(fslot, finally_ip)));
                // Both the normal-try and normal-catch exits route here.
                target.ct_patch(
                    jump_after_try,
                    Instruction::Jump(jump_off(jump_after_try, finally_ip)),
                );
                let jac = jump_after_catch.expect("has_finally implies catch-jump was reserved");
                target.ct_patch(jac, Instruction::Jump(jump_off(jac, finally_ip)));
                // At the top of the finally body, pop the finally frame so
                // that nested try/finally inside the body get a clean stack.
                target.ct_emit(Instruction::FinallyEnd);
                compile_stmt_shared(target, finally_stmt)?;
                // Reserve the FinallyExit slot; patch with end_ip afterwards.
                let end_finally_slot = target.ct_current_ip();
                target.ct_emit(Instruction::FinallyExit(0));
                let end_ip = target.ct_current_ip();
                target.ct_patch(
                    end_finally_slot,
                    Instruction::FinallyExit(jump_off(end_finally_slot, end_ip)),
                );
            } else {
                let end_ip = target.ct_current_ip();
                target.ct_patch(
                    jump_after_try,
                    Instruction::Jump(jump_off(jump_after_try, end_ip)),
                );
            }
        }

        // Issue #244: throw
        Stmt::Throw { value, .. } => {
            let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, value, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            target.ct_emit(Instruction::Throw { src: r });
        }

        Stmt::Block { statements, .. } => {
            target.ct_begin_scope();
            for stmt in statements {
                compile_stmt_shared(target, stmt)?;
            }
            target.ct_end_scope();
        }

        Stmt::Function {
            name,
            params,
            body,
            is_async,
            is_generator,
            span,
            ..
        } => {
            target.ct_compile_function_decl(name, params, body, *is_async, *is_generator, *span)?;
        }

        // Issue #252, #345: real class compilation with static, inheritance, super
        Stmt::Class(class_decl) => {
            target.ct_compile_class(class_decl)?;
        }

        Stmt::EnumDecl { name, variants, .. } => {
            let name_sym = target.ct_sym(name);
            let variant_syms: Vec<SymId> =
                variants.iter().map(|v| target.ct_sym(&v.name)).collect();
            let idx = target.ct_add_enum_decl_payload(hudhudscript_bytecode::EnumDeclPayload {
                name: name_sym,
                variants: variant_syms,
            });
            target.ct_emit(Instruction::EnumDecl(idx));
        }

        Stmt::Match { value, arms, .. } => {
            {
            let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, value, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            target.ct_emit(Instruction::Move { dst: 255, src: r });
            target.ct_set_match_reg(255);
        }
            let mut end_jumps: Vec<usize> = Vec::new();
            let arm_count = arms.len();
            for (arm_idx, arm) in arms.iter().enumerate() {
                let is_last = arm_idx + 1 == arm_count;

                // Issue #1011: Duplicate the match value for non-last arms so that
                // if the pattern or guard fails, the value is still on the stack
                // for the next arm to consume.
                if !is_last {
                }

                let skip_positions = target.ct_compile_match_pattern(&arm.pattern)?;

                // Issue #748: compile guard expression if present
                let guard_skip = if let Some(guard_expr) = &arm.guard {
                    let gr = crate::compiler::regalloc::temp_reg();
                    {
            let r = crate::compiler::expr::compile_reg::compile_expr_to_reg(target, guard_expr, &mut RegAlloc::new_with_base(target.ct_next_local_reg())?);
            target.ct_emit(Instruction::Move { dst: gr, src: r });
        }
                    let pos = target.ct_current_ip();
                    target.ct_emit(Instruction::JumpIfFalse { src: gr, offset: 0 });
                    Some((pos, gr))
                } else {
                    None
                };

                // If this is not the last arm, pop the extra copy of the match
                // value that Dup left on the stack (the pattern consumed the top
                // copy; the duplicate underneath is only needed when we jump to
                // the next arm on failure).
                if !is_last {
                }

                for s in &arm.body {
                    compile_stmt_shared(target, s)?;
                }
                let jmp = target.ct_current_ip();
                target.ct_emit(Instruction::Jump(0));
                end_jumps.push(jmp);

                let next = target.ct_current_ip();
                for skip in skip_positions {
                    target.ct_patch_jump_offset(skip, jump_off(skip, next) as i16);
                }
                if let Some((gs, gr)) = guard_skip {
                    target.ct_patch(gs, Instruction::JumpIfFalse { src: gr, offset: jump_off(gs, next) as i16 });
                }
            }
            let end = target.ct_current_ip();
            for jmp in end_jumps {
                target.ct_patch(jmp, Instruction::Jump(jump_off(jmp, end)));
            }
        }

        // Issue #338: ES-module style import — emit LoadModule instruction
        Stmt::Import { path, .. } => {
            let alias = path
                .rsplit('/')
                .next()
                .unwrap_or(path)
                .trim_end_matches(".hud")
                .trim_end_matches(".js")
                .trim_end_matches(".ts")
                .to_string();
            let alias_sym = target.ct_sym(&alias);
            let idx = target.ct_add_load_module_payload(hudhudscript_bytecode::LoadModulePayload {
                path: path.clone(),
                alias: Some(alias_sym),
            });
            target.ct_emit(Instruction::LoadModule(idx));
        }

        // Issue #245: export (+ #1014: re-export from module)
        Stmt::Export { item, source, .. } => {
            if let Some(module_path) = source {
                // Re-export: first load the module, then compile the inner item
                let alias = module_path
                    .rsplit('/')
                    .next()
                    .unwrap_or(module_path)
                    .trim_end_matches(".hud")
                    .trim_end_matches(".js")
                    .trim_end_matches(".ts")
                    .to_string();
                let alias_sym = target.ct_sym(&alias);
                let idx =
                    target.ct_add_load_module_payload(hudhudscript_bytecode::LoadModulePayload {
                        path: module_path.clone(),
                        alias: Some(alias_sym),
                    });
                target.ct_emit(Instruction::LoadModule(idx));
            }
            compile_stmt_shared(target, item)?;
        }

        // Issue #246: legacy VarDecl
        // Issue #452: emit StoreTyped when a type annotation is present
        _ => unreachable!(),
    }
    Ok(())
}
