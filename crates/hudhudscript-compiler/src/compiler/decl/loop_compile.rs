use hudhudscript_ast::*;
use hudhudscript_bytecode::{Instruction, Value16, SymId};
use crate::compile_codes;
use crate::compiler::{Compiler, CompileTarget};
use crate::compiler::helpers::jump_off;
use crate::compiler::regalloc::RegAlloc;
use crate::CompileResult;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

fn jump_off_i16(site: usize, target: usize) -> CompileResult<i16> {
    i16::try_from(jump_off(site, target)).map_err(|_| {
        compile_codes::generic(format!("conditional jump out of range: site={site}, target={target}"))
    })
}

impl Compiler {
    pub(super) fn compile_decl_loop(
        &mut self, name: &str, items: &[LoopItemAst], mode: &RunModeAst, goal: Option<&GoalSpecAst>,
    ) -> CompileResult<()> {
        let chunk_name = format!("__loop::{}", name);
        if items.is_empty() { return Err(compile_codes::generic(format!("loop '{}' must have at least one item", name))); }
        let is_cyclic = matches!(mode, RunModeAst::Cyclic);
        let is_until_converged = matches!(mode, RunModeAst::UntilConverged);
        if is_cyclic || is_until_converged {
            // R1: validate at least one terminal target (done/fail) exists in any gate
            let mut has_terminal = false;
            for item in items {
                if let LoopItemAst::InlineStep(s) = item {
                    if let Decl::Step { gate: Some(g), .. } = s.as_ref() {
                        for t in g.branches.iter().map(|b| &b.target).chain(std::iter::once(&g.else_target)) {
                            if matches!(t, GateTargetAst::Done | GateTargetAst::Fail) { has_terminal = true; break; }
                        }
                    }
                }
            }
            if !has_terminal {
                let mode_name = if is_cyclic { "cyclic" } else { "until_converged" };
                return Err(compile_codes::generic(format!("{mode_name} loop: requires at least one done/fail terminal target")));
            }
        }
        let times_n: Option<u64> = match mode { RunModeAst::Times(n) => Some(*n), _ => None };
        let until_expr: Option<&Expr> = match mode { RunModeAst::Until(ref e) => Some(e), _ => None };

        let mut step_names: Vec<String> = Vec::new();
        let mut loop_refs: Vec<String> = Vec::new();
        { let mut seen: HashSet<&str> = HashSet::new();
        for item in items {
            if let LoopItemAst::InlineStep(s) = item {
                if let Decl::Step { name: sn, .. } = s.as_ref() {
                    if !seen.insert(sn.as_str()) { return Err(compile_codes::generic(format!("loop '{}': duplicate step '{}'", name, sn))); }
                    step_names.push(sn.clone());
                }
            }
        }}
        for item in items {
            if let LoopItemAst::InlineStep(s) = item { if let Decl::Step { gate: Some(g), .. } = s.as_ref() {
                for t in g.branches.iter().map(|b| &b.target).chain(std::iter::once(&g.else_target)) {
                    match t { GateTargetAst::Loop(ln) | GateTargetAst::LoopStep(ln, _) => { if !loop_refs.contains(ln) { loop_refs.push(ln.clone()); } } _ => {} }
                }
            }}
        }
        { let funcs = self.bytecode.functions.borrow();
          for ln in &loop_refs {
              let fn_name = format!("__loop::{}", ln);
              if !self.bytecode.has_function(&fn_name) { return Err(compile_codes::generic(format!("gate target: loop '{}' not found", ln))); }
          }
        }

        // FAZ G: collect AttachGate items before the callback
        let mut attach_gates: HashMap<String, Vec<(Vec<GateBranchAst>, GateTargetAst)>> = HashMap::new();
        for item in items {
            if let LoopItemAst::AttachGate { gate: gname, step: sname } = item {
                if let Some((branches, else_target)) = self.gate_registry.get(gname) {
                    attach_gates.entry(sname.clone()).or_default().push((branches.clone(), else_target.clone()));
                }
            }
        }

        // A3: resolve UseStep items into InlineStep equivalents before callback
        let mut resolved_items: Vec<LoopItemAst> = Vec::new();
        for item in items {
            match item {
                LoopItemAst::UseStep { name, alias, args: _ } => {
                    if let Some((params, body, gate)) = self.step_registry.get(name).cloned() {
                        let alias_name = alias.clone().unwrap_or_else(|| name.clone());
                        resolved_items.push(LoopItemAst::InlineStep(Box::new(Decl::Step {
                            name: alias_name, params, body, gate, span: hudhudscript_ast::Span::default()
                        })));
                    } else {
                        return Err(compile_codes::generic(format!("use step: unknown step '{}'", name)));
                    }
                }
                _ => resolved_items.push(item.clone()),
            }
        }

        // Rebuild step_names from resolved items (includes UseStep + already-injected AttachStep)
        step_names.clear();
        { let mut seen: HashSet<&str> = HashSet::new();
        for item in &resolved_items {
            if let LoopItemAst::InlineStep(s) = item {
                if let Decl::Step { name: sn, .. } = s.as_ref() {
                    if !seen.insert(sn.as_str()) { return Err(compile_codes::generic(format!("loop '{}': duplicate step '{}'", name, sn))); }
                    step_names.push(sn.clone());
                }
            }
        }}

        // Pass goal info for goal_error() builtin
        let local_goal = goal.map(|g| g.clone());

        let local_times_n = times_n;
        let local_is_cyclic = is_cyclic;
        let local_is_until_converged = is_until_converged;
        let local_until_expr = until_expr.map(|e| e.clone());
        let local_items: Vec<LoopItemAst> = resolved_items;
        let local_step_names = step_names.clone();
        let chunk = self.compile_function_chunk_with(vec!["__entry_selector".to_string()], Some(chunk_name.clone()), false, |compiler| {
            let sel_reg = 0u8; // r0 = __entry_selector param
            // Collect step name → temp label for all steps before emitting dispatch
            let dispatch_step_names = local_step_names.clone();
            let mut dispatch_fixups: Vec<(usize, String)> = Vec::new();
            for (idx, sn) in dispatch_step_names.iter().enumerate() {
                if idx == 0 { continue; } // first step is fall-through (no jump needed)
                let ci = compiler.ct_emit_int_const(idx as i64) as u16;
                let cmp_reg = compiler.next_local_reg; compiler.next_local_reg += 1;
                compiler.ct_emit(Instruction::IntCmpI { dst: cmp_reg, src: sel_reg, imm: idx as i16, op: 4 }); // op:4 = ==
                let jmp_ip = compiler.bytecode.instructions.len();
                compiler.ct_emit(Instruction::JumpIfFalse { src: cmp_reg, offset: 0 });
                // Jump to step sn (place holder, patched after step IPs known)
                compiler.ct_emit(Instruction::Jump(0));
                let fixup_ip = compiler.bytecode.instructions.len() - 1;
                dispatch_fixups.push((fixup_ip, sn.clone()));
                // Patch JumpIfFalse past the Jump
                let next_ip = compiler.bytecode.instructions.len();
                compiler.bytecode.instructions[jmp_ip] = Instruction::JumpIfFalse { src: cmp_reg, offset: jump_off_i16(jmp_ip, next_ip)? };
            }

            compiler.declare_local("result", false)?;
            let result_reg = compiler.ct_local_reg("result").ok_or_else(|| compile_codes::generic("result: local not found".to_string()))?;
            compiler.ct_emit(Instruction::MakeObject { dst: result_reg, count: 0 });
            // B: Initialize goal metadata if present
            if let Some(ref goal_spec) = local_goal {
                let metric_sym = compiler.ct_sym("__goal_metric");
                let metric_ci = compiler.ct_emit_const(Value16::string(goal_spec.metric.clone()));
                let t = compiler.next_local_reg; compiler.next_local_reg += 1;
                compiler.ct_emit(Instruction::LoadConst { dst: t, const_idx: metric_ci as u16 });
                compiler.ct_emit(Instruction::SetProperty { dst: result_reg, obj: result_reg, val: t, prop_sym: metric_sym.0 as u16 });
                let target_sym = compiler.ct_sym("__goal_target");
                let target_val = crate::compiler::expr::compile_reg::compile_expr_to_reg(compiler, &goal_spec.target, &mut RegAlloc::new_with_base(compiler.next_local_reg)?);
                compiler.ct_emit(Instruction::SetProperty { dst: result_reg, obj: result_reg, val: target_val, prop_sym: target_sym.0 as u16 });
            }
            // FP6: initialize __attempt = 0 for bounded retry
            let att_sym_init = compiler.ct_sym("__attempt");
            let zero_ci = compiler.ct_emit_int_const(0) as u16;
            let tmp_init = compiler.next_local_reg; compiler.next_local_reg += 1;
            compiler.ct_emit(Instruction::LoadIntConst { dst: tmp_init, const_idx: zero_ci });
            compiler.ct_emit(Instruction::SetProperty { dst: result_reg, obj: result_reg, val: tmp_init, prop_sym: att_sym_init.0 as u16 });
            let ss = compiler.ct_sym("success"); let st = compiler.ct_sym("status");

            // ── Times(N) header (KR-2: counter > 0, decrement-before-body) ─
            struct TimesState { ctr: u8, cmp_reg: u8, header_ip: usize, exit_fixup_ip: usize }
            let mut times_state: Option<TimesState> = None;
            if let Some(n) = local_times_n {
                let ctr = compiler.next_local_reg; compiler.next_local_reg += 1;
                let ci = compiler.ct_emit_int_const(n as i64) as u16;
                compiler.ct_emit(Instruction::LoadIntConst { dst: ctr, const_idx: ci });
                // header_ip = compare position (AFTER init)
                let header_ip = compiler.bytecode.instructions.len();
                let cmp_reg = compiler.next_local_reg; compiler.next_local_reg += 1;
                // KR-2: counter > 0 (op=2 in VM mapping)
                compiler.ct_emit(Instruction::IntCmpI { dst: cmp_reg, src: ctr, imm: 0, op: 2 });
                let exit_fixup_ip = compiler.bytecode.instructions.len();
                compiler.ct_emit(Instruction::JumpIfFalse { src: cmp_reg, offset: 0 });
                compiler.ct_emit(Instruction::IntSubI { dst: ctr, src: ctr, imm: 1 });
                times_state = Some(TimesState { ctr, cmp_reg, header_ip, exit_fixup_ip });
            }

            // ── Until(expr) header (KR-2: false→body, true→mode_done) ────
            struct UntilState { header_ip: usize, exit_jump_ip: usize }
            let mut until_state: Option<UntilState> = None;
            if let Some(ref expr) = local_until_expr {
                let header_ip = compiler.bytecode.instructions.len();
                let cr = { let mut ra = RegAlloc::new_with_base(compiler.next_local_reg)?; crate::compiler::expr::compile_reg::compile_expr_to_reg(compiler, expr, &mut ra) };
                // JumpIfFalse cr → body (continue when false)
                let body_jmp_ip = compiler.bytecode.instructions.len();
                compiler.ct_emit(Instruction::JumpIfFalse { src: cr, offset: 0 });
                // Jump → mode_done (exit when true)
                let exit_jump_ip = compiler.bytecode.instructions.len();
                compiler.ct_emit(Instruction::Jump(0));
                until_state = Some(UntilState { header_ip, exit_jump_ip });
                // Patch JumpIfFalse body target (KR-1: jump_off)
                let body_target = compiler.bytecode.instructions.len();
                compiler.bytecode.instructions[body_jmp_ip] = Instruction::JumpIfFalse { src: cr, offset: jump_off_i16(body_jmp_ip, body_target)? };
            }

            let mut loop_payloads: HashMap<String, u16> = HashMap::new();
            for ln in &loop_refs {
                let sym = compiler.ct_sym(&format!("__loop::{}", ln));
                loop_payloads.insert(ln.clone(), compiler.ct_add_call_payload(sym, 1) as u16); // LOOP_ENTRY: selector arg
            }

            // ── Compile steps (KR-9: no unreachable Jump(0) after true-target) ─
            let mut step_ips: HashMap<String, usize> = HashMap::new();
            let mut step_fixups: Vec<(usize, String)> = Vec::new();
            let num_steps = local_step_names.len();
            for (step_idx, item) in local_items.iter().enumerate() {
                if let LoopItemAst::InlineStep(s) = item { if let Decl::Step { name: sname, body, gate, .. } = s.as_ref() {
                    let has_next = step_idx + 1 < num_steps;
                    let next_name = if has_next { Some(&local_step_names[step_idx + 1]) } else { None };
                    compiler.begin_scope();
                    step_ips.insert(sname.clone(), compiler.bytecode.instructions.len());
                    for stmt in body { compiler.compile_stmt(stmt)?; }
                    // Compile gate: inline gate takes precedence, else check AttachGate items
                    let mut has_gate = false;
                    if let Some(g) = gate {
                        has_gate = true;
                        for branch in &g.branches {
                            let cr = { let mut ra = RegAlloc::new_with_base(compiler.next_local_reg)?; crate::compiler::expr::compile_reg::compile_expr_to_reg(compiler, &branch.cond, &mut ra) };
                            let jmp_ip = compiler.bytecode.instructions.len();
                            compiler.ct_emit(Instruction::JumpIfFalse { src: cr, offset: 0 });
                            emit_gate_target(compiler, &branch.target, result_reg, &step_ips, &loop_payloads, sname, &mut step_fixups, &ss, &st, next_name, local_times_n.is_some() || local_until_expr.is_some() || local_is_cyclic || local_is_until_converged)?;
                            let next_ip = compiler.bytecode.instructions.len();
                            compiler.bytecode.instructions[jmp_ip] = Instruction::JumpIfFalse { src: cr, offset: jump_off_i16(jmp_ip, next_ip)? };
                        }
                        emit_gate_target(compiler, &g.else_target, result_reg, &step_ips, &loop_payloads, sname, &mut step_fixups, &ss, &st, next_name, local_times_n.is_some() || local_until_expr.is_some() || local_is_cyclic || local_is_until_converged)?;
                    }

                    if let Some(attached) = attach_gates.remove(sname.as_str()) {
                        if has_gate {
                            return Err(compile_codes::generic(format!("step '{}' already has an inline gate; attach gate is not allowed", sname)));
                        }
                        if attached.len() > 1 {
                            return Err(compile_codes::generic(format!("step '{}' cannot have more than one attached gate", sname)));
                        }
                        let (branches, else_target) = &attached[0];
                        for branch in branches {
                            let cr = { let mut ra = RegAlloc::new_with_base(compiler.next_local_reg)?; crate::compiler::expr::compile_reg::compile_expr_to_reg(compiler, &branch.cond, &mut ra) };
                            let jmp_ip = compiler.bytecode.instructions.len();
                            compiler.ct_emit(Instruction::JumpIfFalse { src: cr, offset: 0 });
                            emit_gate_target(compiler, &branch.target, result_reg, &step_ips, &loop_payloads, sname, &mut step_fixups, &ss, &st, next_name, local_times_n.is_some() || local_until_expr.is_some() || local_is_cyclic || local_is_until_converged)?;
                            let next_ip = compiler.bytecode.instructions.len();
                            compiler.bytecode.instructions[jmp_ip] = Instruction::JumpIfFalse { src: cr, offset: jump_off_i16(jmp_ip, next_ip)? };
                        }
                        emit_gate_target(compiler, else_target, result_reg, &step_ips, &loop_payloads, sname, &mut step_fixups, &ss, &st, next_name, local_times_n.is_some() || local_until_expr.is_some() || local_is_cyclic || local_is_until_converged)?;
                    }
                    compiler.end_scope();
                }}
            }

            // ── Iteration footer (KR-1: jump_off for backedge) ──────────
            let iteration_footer_ip = compiler.bytecode.instructions.len();
            if let Some(ref ts) = times_state {
                let after_body = compiler.bytecode.instructions.len();
                compiler.ct_emit(Instruction::Jump(jump_off(after_body, ts.header_ip)));
                let mode_done_ip = compiler.bytecode.instructions.len();
                compiler.bytecode.instructions[ts.exit_fixup_ip] = Instruction::JumpIfFalse { src: ts.cmp_reg, offset: jump_off_i16(ts.exit_fixup_ip, mode_done_ip)? };
            }
            if let Some(ref us) = until_state {
                let after_body = compiler.bytecode.instructions.len();
                compiler.ct_emit(Instruction::Jump(jump_off(after_body, us.header_ip)));
                let mode_done_ip = compiler.bytecode.instructions.len();
                compiler.bytecode.instructions[us.exit_jump_ip] = Instruction::Jump(jump_off(us.exit_jump_ip, mode_done_ip));
            }
            // R1: cyclic/until_converged backedge to first step (or times/until header if present)
            if (local_is_cyclic || local_is_until_converged) && times_state.is_none() && until_state.is_none() {
                let first_step_ip = step_names.first().and_then(|sn| step_ips.get(sn)).copied().unwrap_or(compiler.bytecode.instructions.len());
                let after_body = compiler.bytecode.instructions.len();
                compiler.ct_emit(Instruction::Jump(jump_off(after_body, first_step_ip)));
            }

            // ── mode_done (KR-3: success/status metadata) ───────────────
            // This label covers both after-gate reachability AND mode exhaustion.
            let mode_done_ip = compiler.bytecode.instructions.len();
            emit_setprop(compiler, true, result_reg, &ss);
            emit_setprop_str(compiler, "done", result_reg, &st);
            compiler.ct_emit(Instruction::Return { src: result_reg });

            // Resolve fixups (KR-1: jump_off)
            // FAZ F: patch dispatch fixups (must be done after step_ips populated)
            for (fixup_ip, sn) in &dispatch_fixups {
                if let Some(&target_ip) = step_ips.get(sn) {
                    compiler.bytecode.instructions[*fixup_ip] = Instruction::Jump(jump_off(*fixup_ip, target_ip));
                } else { return Err(compile_codes::generic(format!("entry dispatch: step '{}' not found", sn))); }
            }
            for (fixup_ip, target_step) in &step_fixups {
                let target_ip = if target_step == "__iteration_done__" { iteration_footer_ip }
                    else { *step_ips.get(target_step).ok_or_else(|| compile_codes::generic(format!("gate target: step '{}' not found", target_step)))? };
                compiler.bytecode.instructions[*fixup_ip] = Instruction::Jump(jump_off(*fixup_ip, target_ip));
            }
            Ok(())
        })?;

        // FAZ F: register step names for cross-loop selector lookup
        self.loop_step_names.insert(name.to_string(), step_names);

        if self.bytecode.has_function(&chunk_name) { return Err(compile_codes::generic(format!("duplicate loop: '{}'", name))); }
        self.bytecode.add_function(chunk_name, Arc::new(chunk));
        Ok(())
    }

    // ── Chain (R3: on_done/on_fail branching) ───────────────────────
    pub(super) fn compile_decl_chain(
        &mut self, name: &str, links: &[ChainLinkAst], _mode: &RunModeAst,
    ) -> CompileResult<()> {
        let chunk_name = format!("__chain::{}", name);
        if links.is_empty() { return Err(compile_codes::generic(format!("chain '{}' must have at least one link", name))); }
        for link in links { if let Some(inline) = &link.inline_loop { if let Decl::Loop { name: ln, items, mode: lm, .. } = inline.as_ref() { self.compile_decl_loop(ln, items, lm, None)?; } } }

        let local_links: Vec<ChainLinkAst> = links.iter().map(|l| l.clone()).collect();
        let chunk = self.compile_function_chunk_with(vec![], Some(chunk_name.clone()), false, |compiler| {
            let ss = compiler.ct_sym("success");
            let st = compiler.ct_sym("status");
            let link_count = local_links.len();
            for (i, link) in local_links.iter().enumerate() {
                let is_last = i + 1 == link_count;
                let sym = compiler.ct_sym(&format!("__loop::{}", link.loop_name));
                let pi = compiler.ct_add_call_payload(sym, 1) as u16; // LOOP_ENTRY: selector arg
                let call_dst = compiler.next_local_reg; compiler.next_local_reg += 1;
                // P0: emit selector=0 (first step) as the entry argument
                let sel_reg = compiler.next_local_reg; compiler.next_local_reg += 1;
                let const_idx = compiler.ct_emit_int_const(0) as u16;
                compiler.ct_emit(Instruction::LoadIntConst { dst: sel_reg, const_idx });
                compiler.ct_emit(Instruction::Call { dst: call_dst, payload_idx: pi, first_arg: sel_reg, arg_count: 1 });
                if is_last {
                    // Last link: return child result (on_done/on_fail customize metadata)
                    compiler.ct_emit(Instruction::Return { src: call_dst });
                } else {
                    let prop_dst = compiler.next_local_reg; compiler.next_local_reg += 1;
                    compiler.ct_emit(Instruction::GetProperty { dst: prop_dst, obj: call_dst, prop_sym: ss.0 as u16 });
                    let jmp_ip = compiler.bytecode.instructions.len();
                    // success=true → on_done; false → on_fail (return child)
                    compiler.ct_emit(Instruction::JumpIfTrue { src: prop_dst, offset: 0 });
                    // Fail path: emit on_fail target
                    emit_chain_target(compiler, &link.on_fail, call_dst, &ss, &st)?;
                    // Success path: emit on_done target, patch JumpIfTrue to here
                    let next_ip = compiler.bytecode.instructions.len();
                    compiler.bytecode.instructions[jmp_ip] = Instruction::JumpIfTrue { src: prop_dst, offset: jump_off_i16(jmp_ip, next_ip)? };
                    emit_chain_target(compiler, &link.on_done, call_dst, &ss, &st)?;
                }
            }
            Ok(())
        })?;

        if self.bytecode.has_function(&chunk_name) { return Err(compile_codes::generic(format!("duplicate chain: '{}'", name))); }
        self.bytecode.add_function(chunk_name, Arc::new(chunk));
        Ok(())
    }

    pub(super) fn compile_run_loop(&mut self, name: &str) -> CompileResult<()> {
        let const_idx = self.ct_emit_int_const(0) as u16;
        let fn_name = format!("__loop::{}", name);
        { if !self.bytecode.has_function(&fn_name) { return Err(compile_codes::generic(format!("run loop '{}': loop not found", name))); } }
        let sym = self.ct_sym(&fn_name); let payload_idx = self.ct_add_call_payload(sym, 1);
        let sel_reg = self.next_local_reg; self.next_local_reg += 1;
        self.ct_emit(Instruction::LoadIntConst { dst: sel_reg, const_idx: const_idx });
        let dst = self.next_local_reg; self.next_local_reg += 1;
        self.ct_emit(Instruction::Call { dst, payload_idx: payload_idx as u16, first_arg: sel_reg, arg_count: 1 });
        Ok(())
    }

    pub(super) fn compile_run_chain(&mut self, name: &str) -> CompileResult<()> {
        let fn_name = format!("__chain::{}", name);
        { if !self.bytecode.has_function(&fn_name) { return Err(compile_codes::generic(format!("run chain '{}': chain not found", name))); } }
        let sym = self.ct_sym(&fn_name); let payload_idx = self.ct_add_call_payload(sym, 0);
        let dst = self.next_local_reg; self.next_local_reg += 1;
        self.ct_emit(Instruction::Call { dst, payload_idx: payload_idx as u16, first_arg: 0, arg_count: 0 });
        Ok(())
    }
}

// ── Gate target emission ─────────────────────────────────────────────
fn emit_gate_target(
    compiler: &mut Compiler, target: &GateTargetAst, result_reg: u8,
    step_ips: &HashMap<String, usize>, loop_payloads: &HashMap<String, u16>,
    current_step: &str, step_fixups: &mut Vec<(usize, String)>,
    ss: &SymId, st: &SymId,
    next_step: Option<&String>, has_loop_mode: bool,
) -> CompileResult<()> {
    match target {
        GateTargetAst::Done => { emit_setprop(compiler, true, result_reg, ss); emit_setprop_str(compiler, "done", result_reg, st); compiler.ct_emit(Instruction::Return { src: result_reg }); }
        GateTargetAst::Fail => { emit_setprop(compiler, false, result_reg, ss); emit_setprop_str(compiler, "failed", result_reg, st); compiler.ct_emit(Instruction::Return { src: result_reg }); }
        GateTargetAst::Step(sn) => {
            let ip = compiler.bytecode.instructions.len();
            if let Some(&target_ip) = step_ips.get(sn) { compiler.ct_emit(Instruction::Jump(jump_off(ip, target_ip))); }
            else { compiler.ct_emit(Instruction::Jump(0)); step_fixups.push((ip, sn.clone())); }
        }
        GateTargetAst::Retry => {
            // FP6: bounded retry — __attempt initialized to 0 at loop start, max 3
            let att_sym = compiler.ct_sym("__attempt");
            let att_reg = compiler.next_local_reg; compiler.next_local_reg += 1;
            compiler.ct_emit(Instruction::GetProperty { dst: att_reg, obj: result_reg, prop_sym: att_sym.0 as u16 });
            compiler.ct_emit(Instruction::IntAddI { dst: att_reg, src: att_reg, imm: 1 });
            compiler.ct_emit(Instruction::SetProperty { dst: result_reg, obj: result_reg, val: att_reg, prop_sym: att_sym.0 as u16 });
            let bound_reg = compiler.next_local_reg; compiler.next_local_reg += 1;
            compiler.ct_emit(Instruction::IntCmpI { dst: bound_reg, src: att_reg, imm: 3, op: 3 });
            let exceeded_ip = compiler.bytecode.instructions.len();
            compiler.ct_emit(Instruction::JumpIfTrue { src: bound_reg, offset: 0 });
            // Not exceeded: jump to step
            if let Some(&ip) = step_ips.get(current_step) {
                let site_ip = compiler.bytecode.instructions.len();
                compiler.ct_emit(Instruction::Jump(jump_off(site_ip, ip)));
            }
            // Exceeded: escalate
            let esc_ip = compiler.bytecode.instructions.len();
            compiler.bytecode.instructions[exceeded_ip] = Instruction::JumpIfTrue { src: bound_reg, offset: jump_off_i16(exceeded_ip, esc_ip)? };
            emit_setprop_str(compiler, "escalated", result_reg, st);
            compiler.ct_emit(Instruction::Return { src: result_reg });
        }
        GateTargetAst::Loop(ln) => {
            if let Some(&pi) = loop_payloads.get(ln) {
                // P0: emit selector=0 for first step entry
                let const_idx = compiler.ct_emit_int_const(0) as u16;
                let sel_reg = compiler.next_local_reg; compiler.next_local_reg += 1;
                compiler.ct_emit(Instruction::LoadIntConst { dst: sel_reg, const_idx });
                compiler.ct_emit(Instruction::Call { dst: result_reg, payload_idx: pi, first_arg: sel_reg, arg_count: 1 });
                compiler.ct_emit(Instruction::Return { src: result_reg });
            }
            else { return Err(compile_codes::generic(format!("gate target: loop '{}' not found", ln))); }
        }
        GateTargetAst::LoopStep(ln, sn) => {
            // FAZ F: pass step index as entry selector
            if let Some(&pi) = loop_payloads.get(ln) {
                // Look up step index from the target loop's step names
                let step_idx = match compiler.loop_step_names.get(ln).and_then(|names| names.iter().position(|n| n == sn)) {
                    Some(idx) => idx as u8,
                    None => return Err(compile_codes::generic(format!("gate target: loop '{}' has no step '{}'", ln, sn))),
                };
                // P0: always emit selector, even for step 0
                let idx_reg = compiler.next_local_reg; compiler.next_local_reg += 1;
                let ci = compiler.ct_emit_int_const(step_idx as i64) as u16;
                compiler.ct_emit(Instruction::LoadIntConst { dst: idx_reg, const_idx: ci });
                compiler.ct_emit(Instruction::Call { dst: result_reg, payload_idx: pi, first_arg: idx_reg, arg_count: 1 });
                compiler.ct_emit(Instruction::Return { src: result_reg });
            } else {
                return Err(compile_codes::generic(format!("gate target: loop '{}' not found for step '{}'", ln, sn)));
            }
        }
        GateTargetAst::Continue => {
            let ip = compiler.bytecode.instructions.len();
            if let Some(ns) = next_step {
                if let Some(&target_ip) = step_ips.get(ns.as_str()) { compiler.ct_emit(Instruction::Jump(jump_off(ip, target_ip))); }
                else { compiler.ct_emit(Instruction::Jump(0)); step_fixups.push((ip, ns.clone())); }
            } else if has_loop_mode {
                compiler.ct_emit(Instruction::Jump(0)); step_fixups.push((ip, "__iteration_done__".to_string()));
            } else {
                compiler.ct_emit(Instruction::Return { src: result_reg });
            }
        }
        GateTargetAst::Pause => { emit_setprop_str(compiler, "paused", result_reg, st); compiler.ct_emit(Instruction::Return { src: result_reg }); }
        GateTargetAst::Approval => { emit_setprop_str(compiler, "awaiting_approval", result_reg, st); compiler.ct_emit(Instruction::Return { src: result_reg }); }
        GateTargetAst::Escalate => { emit_setprop_str(compiler, "escalated", result_reg, st); compiler.ct_emit(Instruction::Return { src: result_reg }); }
    }
    Ok(())
}
fn emit_setprop(compiler: &mut Compiler, val: bool, obj: u8, psym: &SymId) { let ci = compiler.ct_emit_const(Value16::bool_(val)); let t = compiler.next_local_reg; compiler.next_local_reg += 1; compiler.ct_emit(Instruction::LoadConst { dst: t, const_idx: ci as u16 }); compiler.ct_emit(Instruction::SetProperty { dst: obj, obj, val: t, prop_sym: psym.0 as u16 }); }
fn emit_setprop_str(compiler: &mut Compiler, val: &str, obj: u8, psym: &SymId) { let ci = compiler.ct_emit_const(Value16::string(val.to_string())); let t = compiler.next_local_reg; compiler.next_local_reg += 1; compiler.ct_emit(Instruction::LoadConst { dst: t, const_idx: ci as u16 }); compiler.ct_emit(Instruction::SetProperty { dst: obj, obj, val: t, prop_sym: psym.0 as u16 }); }

// R3: chain target emission
fn emit_chain_target(compiler: &mut Compiler, target: &ChainTargetAst, child_reg: u8, ss: &SymId, st: &SymId) -> CompileResult<()> {
    match target {
        ChainTargetAst::Next => {
            // No-op: caller continues to next link after this returns
        }
        ChainTargetAst::ChainFail => {
            compiler.ct_emit(Instruction::Return { src: child_reg });
        }
        ChainTargetAst::ChainDone => {
            emit_setprop(compiler, true, child_reg, ss);
            emit_setprop_str(compiler, "done", child_reg, st);
            compiler.ct_emit(Instruction::Return { src: child_reg });
        }
        ChainTargetAst::Loop(ln) => {
            let fn_name = format!("__loop::{}", ln);
            let sym = compiler.ct_sym(&fn_name);
            let pi = compiler.ct_add_call_payload(sym, 1) as u16; // LOOP_ENTRY: selector arg
            // P0: emit selector=0 for first step
            let sel_reg = compiler.next_local_reg; compiler.next_local_reg += 1;
            let const_idx = compiler.ct_emit_int_const(0) as u16;
            compiler.ct_emit(Instruction::LoadIntConst { dst: sel_reg, const_idx });
            compiler.ct_emit(Instruction::Call { dst: child_reg, payload_idx: pi, first_arg: sel_reg, arg_count: 1 });
            compiler.ct_emit(Instruction::Return { src: child_reg });
        }
    }
    Ok(())
}
