//! Loop Engineering DSL → sentetik fonksiyon AST lowering (M3, v0.9.36).
//!
//! `loop/step/gate`, `run loop`, `chain` bildirimleri saf-HudHudScript
//! sentetik fonksiyonlara dönüştürülür (AST→AST): adım gövdeleri, koşullar,
//! `let/if/while` vb. mevcut HIR/MIR lowering'ine aynen emanet edilir —
//! motor tarafında sıfır değişiklik. VM derleyicisinin (`loop_compile.rs`)
//! durum makinesi semantiği birebir denklenir: seçici-dispatch if zinciri,
//! gate koşulları `MirTerminator::CondBranch`'e iner, sonuç objesi
//! `result`, retry üst sınırı 3 (VM: `__attempt`+1 > 3 → escalate).
//!
//! Sözleşme: desteklenmeyen bir detay (UseStep args, until_converged
//! yakınsama ölçütü vb.) görülürse rewrite TAMAMEN iptal edilir ve
//! orijinal deyimler döner → bilinen dürüst VM-fallback yolu korunur;
//! asla yanlış kod üretilmez.

use hudhudscript_ast::stmt::{GateBranchAst, GateTargetAst, GoalSpecAst, LoopItemAst, RunModeAst, Stmt};
use hudhudscript_ast::{ChainLinkAst, Decl, Expr, Literal, Span, StepGateAst, UnaryOp};

use crate::hir_loop_build::*;

/// VM retry sınırı: `__attempt + 1 > 3` → escalate (loop_compile.rs denklemi).
const RETRY_CAP: i64 = 3;
/// Cyclic/until_converged: pratikte sınırsız iterasyon (çıkış hedeflerle).
const UNBOUNDED_ITERS: i64 = i64::MAX / 4;

struct StepDef {
    name: String,
    body: Vec<Stmt>,
    gate: Option<StepGateAst>,
}

/// Görünür loop-artefaktları yeniden yazar; desteklenmeyen detayda None.
pub(crate) fn rewrite_loops(stmts: &[Stmt]) -> Option<Vec<Stmt>> {
    let mut loops: Vec<(String, RunModeAst, Vec<LoopItemAst>, Option<GoalSpecAst>)> = Vec::new();
    let mut chains: Vec<(String, Vec<ChainLinkAst>)> = Vec::new();
    let mut steps: Vec<StepDef> = Vec::new();
    let mut gates: Vec<(String, Vec<GateBranchAst>, GateTargetAst)> = Vec::new();
    let mut out: Vec<Stmt> = Vec::new();
    let mut has_any = false;

    for stmt in stmts {
        match stmt {
            Stmt::Decl(Decl::Loop { name, mode, items, goal, .. }) => {
                has_any = true;
                loops.push((name.clone(), mode.clone(), items.clone(), goal.clone()));
            }
            Stmt::Decl(Decl::Chain { name, links, .. }) => {
                has_any = true;
                chains.push((name.clone(), links.clone()));
            }
            Stmt::Decl(Decl::Step { name, body, gate, .. }) => {
                has_any = true;
                steps.push(StepDef {
                    name: name.clone(),
                    body: body.clone(),
                    gate: gate.clone().or_else(|| lookup_gate(&gates, name)),
                });
            }
            Stmt::Decl(Decl::Gate { name, branches, else_target, .. }) => {
                has_any = true;
                gates.push((name.clone(), branches.clone(), else_target.clone()));
            }
            Stmt::Decl(Decl::RunLoop { name, .. }) => {
                has_any = true;
                out.push(call_stmt(&format!("__loop__{name}")));
            }
            Stmt::Decl(Decl::RunChain { name, .. }) => {
                has_any = true;
                out.push(call_stmt(&format!("__chain__{name}")));
            }
            other => out.push(other.clone()),
        }
    }
    if !has_any {
        // M6: SOP bildirimleri — Role/Relation/Council/Compose/Event atlanır
        // (metadata + stderr uyarıları); Effect → __event__ fn; kapsamsız
        // Ability (on attack) → fn. Kalıp: eylem adları küresel.
        let mut sop: Vec<Stmt> = Vec::new();
        let mut rest: Vec<Stmt> = Vec::new();
        let mut event_names: Vec<String> = Vec::new();
        for stmt in out.into_iter() {
            match stmt {
                Stmt::Decl(Decl::Effect { event_name, params, body, .. }) => {
                    event_names.push(event_name.clone());
                    let fname = format!("__event__{event_name}");
                    let f = Stmt::Function {
                        name: fname, params, body,
                        is_async: false, is_generator: false, type_params: vec![],
                        span: Span::default(),
                    };
                    rest.push(f);
                }
                Stmt::Decl(Decl::Ability { subject_type: None, name, params, body, .. }) => {
                    // Kapsamsız yetenek: her subject için metot tablosuna
                    // kaydedilir (ClassTable); fn gövdesi burada sentezlenir.
                    let f = Stmt::Function {
                        name: name.clone(), params, body,
                        is_async: false, is_generator: false, type_params: vec![],
                        span: Span::default(),
                    };
                    rest.push(f);
                }
                Stmt::Decl(Decl::Ability { subject_type: Some(_), .. }) => {
                    // Kapsamlı yetenek: hir_class kolu ele alır; burada tüket
                }
                Stmt::Decl(
                    Decl::Role { .. }
                    | Decl::Relation { .. }
                    | Decl::Council { .. }
                    | Decl::Compose { .. }
                    | Decl::Event { .. },
                ) => {}
                // View-subject (subject X of Y) + yetenek = derin kompozisyon
                // semantiği (view auto-bind, self üzerinden cross-subject
                // mutasyon). Native denklemi yok — DÜRÜST fallback: tüm
                // dosya VM'de koşar (yanlış native üretimi yasak).
                Stmt::Decl(Decl::Subject { of_subject: Some(_), ability_defs, .. })
                    if !ability_defs.is_empty() =>
                {
                    return None;
                }
                other => rest.push(other),
            }
        }
        let _ = (&sop,);
        sop.extend(rest);
        crate::hir_sop::set_event_names(event_names);
        return Some(sop);
    }

    // Zincir linklerindeki INLINE loop'ları da kayda al (07_chain deseni):
    // link.loop_name ile sentetik __loop__ sembolü eşleşsin.
    for (_, links) in &chains {
        for link in links {
            if let Some(d) = &link.inline_loop {
                if let Decl::Loop { name, mode, items, goal, .. } = &**d {
                    loops.push((name.clone(), mode.clone(), items.clone(), goal.clone()));
                }
            }
        }
    }

    // Loop fonksiyonları: ad gövdeleri (InlineStep/UseStep/AttachGate).
    let mut synth: Vec<Stmt> = Vec::new();
    for (name, mode, items, goal) in &loops {
        let mut lsteps: Vec<StepDef> = Vec::new();
        for it in items {
            match it {
                LoopItemAst::InlineStep(d) => {
                    if let Decl::Step { name, body, gate, .. } = &**d {
                        lsteps.push(StepDef {
                            name: name.clone(),
                            body: body.clone(),
                            gate: gate.clone().or_else(|| lookup_gate(&gates, name)),
                        });
                    }
                }
                LoopItemAst::UseStep { name, alias, args } => {
                    if !args.is_empty() {
                        return None; // UseStep arg bağlama henüz yok → dürüst fallback
                    }
                    let base = steps.iter().find(|s| s.name == *name)?;
                    lsteps.push(StepDef {
                        name: alias.clone().unwrap_or_else(|| name.clone()),
                        body: base.body.clone(),
                        gate: base.gate.clone().or_else(|| lookup_gate(&gates, name)),
                    });
                }
                LoopItemAst::AttachGate { .. } => {
                    return None; // nadir yol → fallback
                }
            }
        }
        synth.push(synth_loop_fn(name, mode, &lsteps, goal, &loops)?);
    }
    for (name, links) in &chains {
        synth.push(synth_chain_fn(name, links, &loops)?);
    }
    let mut result = synth;
    result.extend(out);
    Some(result)
}

fn lookup_gate(
    gates: &[(String, Vec<GateBranchAst>, GateTargetAst)],
    step: &str,
) -> Option<StepGateAst> {
    // Adlandırılmış gate'i adım adına eşle: `gate {name}` ya da `{step}`.
    gates.iter().find(|(n, _, _)| n == step || n == &format!("{step}_gate")).map(|(_, b, e)| StepGateAst {
        name: String::new(),
        branches: b.clone(),
        else_target: e.clone(),
    })
}

// ── sentetik fonksiyon üretimi ────────────────────────────────────────

fn synth_loop_fn(
    name: &str,
    mode: &RunModeAst,
    steps: &[StepDef],
    goal: &Option<GoalSpecAst>,
    all_loops: &[(String, RunModeAst, Vec<LoopItemAst>, Option<GoalSpecAst>)],
) -> Option<Stmt> {
    if steps.is_empty() {
        return None;
    }
    let idx_of = |s: &str| steps.iter().position(|st| st.name == s);
    let loop_exists =
        |s: &str| all_loops.iter().any(|(n, ..)| n == s) || name == s;

    let mut body: Vec<Stmt> = Vec::new();
    body.push(let_int("__ll_state", 0)); // VM: seçici ilk adımdan başlar
    body.push(let_obj("result", goal));
    body.push(let_int("__ll_attempt", 0));
    body.push(let_bool("__ll_cont", false));
    let (iters, until) = match mode {
        RunModeAst::Times(n) => (*n as i64, None),
        RunModeAst::Once => (1, None),
        RunModeAst::Until(e) => (UNBOUNDED_ITERS, Some(e.clone())),
        RunModeAst::Cyclic | RunModeAst::UntilConverged => (UNBOUNDED_ITERS, None),
    };
    body.push(let_int("__ll_iters", iters));

    let mut pass: Vec<Stmt> = vec![assign_bool("__ll_cont", false)];
    for (i, st) in steps.iter().enumerate() {
        let mut blk: Vec<Stmt> = Vec::new();
        for s in &st.body {
            blk.push(s.clone());
        }
        blk.push(gate_stmt(&st.gate, &idx_of, &loop_exists)?);
        pass.push(Stmt::If {
            condition: and(not_cont(), eq(
                Expr::Identifier("__ll_state".into(), Span::default()),
                Expr::Literal(Literal::Int(i as i64), Span::default()),
            )),
            then_branch: Box::new(Stmt::Block { statements: blk, span: Span::default() }),
            else_branch: None,
            span: Span::default(),
        });
    }
    pass.push(assign_int_expr(
        "__ll_iters",
        sub(ident("__ll_iters"), int(1)),
    ));
    let cond = match &until {
        Some(e) => and3(
            ge(ident("__ll_state"), int(0)),
            gt(ident("__ll_iters"), int(0)),
            Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(e.clone()),
                span: Span::default(),
            },
        ),
        None => and(
            ge(ident("__ll_state"), int(0)),
            gt(ident("__ll_iters"), int(0)),
        ),
    };
    body.push(Stmt::While {
        condition: cond,
        body: Box::new(Stmt::Block { statements: pass, span: Span::default() }),
        span: Span::default(),
    });
    body.push(Stmt::Return {
        value: Some(Expr::Identifier("result".into(), Span::default())),
        span: Span::default(),
    });
    Some(fn_decl(&format!("__loop__{name}"), body))
}

fn synth_chain_fn(
    name: &str,
    links: &[hudhudscript_ast::ChainLinkAst],
    all_loops: &[(String, RunModeAst, Vec<LoopItemAst>, Option<GoalSpecAst>)],
) -> Option<Stmt> {
    if links.is_empty() {
        return None;
    }
    // VM denklemi: link sırayla; her linkin ardında success kontrolü,
    // başarısızlıkta kısa devre (Break while gövdesinde — geçerli).
    let mut seq: Vec<Stmt> = Vec::new();
    for link in links {
        // INLINE chain-loop: ad Decl içinden (rewrite_loops kayda aldı);
        // harici link: loop_name. on_done/on_fail Next kabul edilir.
        let lname = match &link.inline_loop {
            Some(d) => match &**d {
                Decl::Loop { name, .. } => name.clone(),
                _ => return None,
            },
            None => link.loop_name.clone(),
        };
        if !all_loops.iter().any(|(n, ..)| *n == lname) {
            return None;
        }
        seq.push(assign_expr("__ll_r", call(&format!("__loop__{lname}"))));
        seq.push(Stmt::If {
            condition: eq(success_of("__ll_r"), bool_lit(true)),
            then_branch: Box::new(wrap_block(vec![])),
            else_branch: Some(Box::new(wrap_block(vec![
                Stmt::Break { span: Span::default() },
            ]))),
            span: Span::default(),
        });
    }
    seq.push(Stmt::Break { span: Span::default() }); // tüm linkler bitti
    let mut body: Vec<Stmt> = vec![let_int("__ll_r", 0)];
    body.push(Stmt::While {
        condition: bool_lit(true),
        body: Box::new(Stmt::Block { statements: seq, span: Span::default() }),
        span: Span::default(),
    });
    Some(fn_decl(&format!("__chain__{name}"), body))
}

// ── gate → if zinciri ─────────────────────────────────────────────────

fn gate_stmt(
    gate: &Option<StepGateAst>,
    idx_of: &dyn Fn(&str) -> Option<usize>,
    loop_exists: &dyn Fn(&str) -> bool,
) -> Option<Stmt> {
    let g = gate.as_ref()?;
    let mut chain: Option<Stmt> = None; // sondan başa örün
    let else_arm = target_stmt(&g.else_target, idx_of, loop_exists)?;
    chain = Some(wrap_block(else_arm));
    for b in g.branches.iter().rev() {
        let arm = target_stmt(&b.target, idx_of, loop_exists)?;
        chain = Some(Stmt::If {
            condition: b.cond.clone(),
            then_branch: Box::new(wrap_block(arm)),
            else_branch: Some(Box::new(chain?)),
            span: Span::default(),
        });
    }
    chain
}

fn target_stmt(
    t: &GateTargetAst,
    idx_of: &dyn Fn(&str) -> Option<usize>,
    loop_exists: &dyn Fn(&str) -> bool,
) -> Option<Vec<Stmt>> {
    let mut v = Vec::new();
    match t {
        GateTargetAst::Done => {
            v.push(result_set("success", bool_lit(true)));
            v.push(result_set("done", bool_lit(true)));
            v.push(assign_int("__ll_state", -1));
        }
        GateTargetAst::Fail => {
            v.push(result_set("failed", bool_lit(true)));
            v.push(assign_int("__ll_state", -1));
        }
        GateTargetAst::Escalate | GateTargetAst::Pause | GateTargetAst::Approval => {
            v.push(result_set("escalated", bool_lit(true)));
            v.push(assign_int("__ll_state", -1));
        }
        GateTargetAst::Continue => v.push(assign_bool("__ll_cont", true)),
        GateTargetAst::Retry => {
            // VM: __attempt+1 > 3 → escalate, değilse aynı adım yeniden
            v.push(assign_expr(
                "__ll_attempt",
                add(ident("__ll_attempt"), int(1)),
            ));
            v.push(Stmt::If {
                condition: gt(ident("__ll_attempt"), int(RETRY_CAP)),
                then_branch: Box::new(wrap_block(vec![
                    result_set("escalated", bool_lit(true)),
                    assign_int("__ll_state", -1),
                ])),
                else_branch: None,
                span: Span::default(),
            });
        }
        GateTargetAst::Step(s) => v.push(assign_int("__ll_state", idx_of(s)? as i64)),
        GateTargetAst::Loop(l) | GateTargetAst::LoopStep(l, _) => {
            if !loop_exists(l) {
                return None;
            }
            v.push(assign_expr("result", call(&format!("__loop__{l}"))));
            v.push(assign_int("__ll_state", -1));
        }
    }
    Some(v)
}
