//! Loop engineering declaration parsing

use hudhudscript_ast::{Decl, Expr, GateBranchAst, GateTargetAst, LoopItemAst, RunModeAst, StepGateAst, Stmt, ChainLinkAst, ChainTargetAst, AttachStepTarget, GoalSpecAst};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_expression};
use crate::parser::statement::declarations::parse_block;
use crate::pest_parser::Rule;

pub fn parse_gate_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
.next().ok_or_else(|| parse_codes::invalid_syntax("Expected gate name", span))?
        .as_str()
        .to_string();

    let mut branches = Vec::new();
    let mut else_target: Option<GateTargetAst> = None;

    for item in inner {
        match item.as_rule() {
            Rule::when_branch => {
                let mut parts = item.into_inner();
                let cond = parts
                    .next()
                    .ok_or_else(|| parse_codes::invalid_syntax("Expected condition", span))?;
                let expr = parse_expression(cond)?;
                let target_pair = parts
                    .next()
                    .ok_or_else(|| parse_codes::invalid_syntax("Expected gate target", span))?;
                let target = parse_gate_target(target_pair);
                branches.push(GateBranchAst { cond: expr, target });
            }
            Rule::gate_target => {
                else_target = Some(parse_gate_target(item));
            }
            _ => {
                // else keyword consumed implicitly; the gate_target follows
                if item.as_rule() == Rule::gate_target {
                    else_target = Some(parse_gate_target(item));
                }
            }
        }
    }

    let else_target = else_target
        .ok_or_else(|| parse_codes::invalid_syntax("Gate requires else target", span))?;

    Ok(Stmt::Decl(Decl::Gate {
        name,
        branches,
        else_target,
        span,
    }))
}

fn parse_gate_target(pair: Pair<Rule>) -> GateTargetAst {
    let s = pair.as_str();
    match s {
        "done" => return GateTargetAst::Done,
        "fail" => return GateTargetAst::Fail,
        "retry" => return GateTargetAst::Retry,
        "continue" => return GateTargetAst::Continue,
        "pause" => return GateTargetAst::Pause,
        "approval" => return GateTargetAst::Approval,
        "escalate" => return GateTargetAst::Escalate,
        s if s.starts_with("loop ") => {
            let rest = s.trim_start_matches("loop ");
            return GateTargetAst::Loop(rest.to_string());
        }
        s if s.contains('.') => {
            let parts: Vec<&str> = s.splitn(2, '.').collect();
            return GateTargetAst::LoopStep(parts[0].to_string(), parts[1].to_string());
        }
        _ => return GateTargetAst::Step(s.to_string()),
    }
}

pub fn parse_step_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
.next().ok_or_else(|| parse_codes::invalid_syntax("Expected step name", span))?
        .as_str()
        .to_string();

    let mut params = Vec::new();
    let mut body = Vec::new();
    let mut gate: Option<StepGateAst> = None;

    for item in inner {
        match item.as_rule() {
            Rule::identifier => {
                params.push(item.as_str().to_string());
            }
            Rule::block => {
                let block_stmt = parse_block(item.clone())?;
                let block_span = pair_to_span(&item);
                if let Stmt::Block { statements: stmts, .. } = block_stmt {
                    for s in stmts {
                        match s {
                            Stmt::Decl(Decl::Gate { name: gname, branches: gbranches, else_target: gelse, .. }) => {
                                if gate.is_some() {
                                    return Err(parse_codes::invalid_syntax(&format!("step '{}' cannot have more than one gate", name), block_span));
                                }
                                gate = Some(StepGateAst { name: gname, branches: gbranches, else_target: gelse });
                            }
                            other => body.push(other),
                        }
                    }
                }
            }
            Rule::gate_decl => {
                // Parse inline gate
                let item_span = pair_to_span(&item);
                let stmt = parse_gate_decl(item)?;
                if let Stmt::Decl(Decl::Gate {
                    name: gname,
                    branches: gbranches,
                    else_target: gelse,
                    ..
                }) = stmt
                {
                    if gate.is_some() {
                        return Err(parse_codes::invalid_syntax(&format!("step '{}' cannot have more than one gate", name), item_span));
                    }
                    gate = Some(StepGateAst {
                        name: gname,
                        branches: gbranches,
                        else_target: gelse,
                    });
                }
            }
            Rule::statement => {
                // Statement inside step body — parse and add to body (gates extracted separately)
                let stmt_span = pair_to_span(&item);
                if let Ok(Some(stmt)) = super::super::statement::parse_statement(item) {
                    if let Stmt::Decl(Decl::Gate { name: gname, branches: gbranches, else_target: gelse, .. }) = &stmt {
                        if gate.is_some() {
                            return Err(parse_codes::invalid_syntax(&format!("step '{}' cannot have more than one gate", name), stmt_span));
                        }
                        gate = Some(StepGateAst { name: gname.clone(), branches: gbranches.clone(), else_target: gelse.clone() });
                    } else {
                        body.push(stmt);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::Step {
        name,
        params,
        body,
        gate,
        span,
    }))
}

pub fn parse_loop_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
.next().ok_or_else(|| parse_codes::invalid_syntax("Expected loop name", span))?
        .as_str()
        .to_string();

    let mut mode = RunModeAst::Once;
    let mut goal: Option<GoalSpecAst> = None;
    let mut items = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::run_mode => {
                mode = parse_run_mode(item, span)?;
            }
            Rule::goal_spec => {
                let mut metric_name = String::new();
                let mut target: Option<Expr> = None;
                for gp in item.into_inner() {
                    if gp.as_rule() == Rule::goal_pair {
                        let mut parts = gp.into_inner();
                        let key = parts.next().map(|p| p.as_str().to_string()).unwrap_or_default();
                        let val_pair = parts.next();
                        if key == "metric" {
                            metric_name = val_pair.map(|p| p.as_str().to_string()).unwrap_or_default();
                        } else if key == "target" {
                            target = val_pair.and_then(|p| parse_expression(p).ok());
                        }
                    }
                }
                if !metric_name.is_empty() {
                    goal = Some(GoalSpecAst { metric: metric_name, target: target.unwrap_or(Expr::Literal(hudhudscript_ast::Literal::Int(0), span)) });
                }
            }
            _ => {
                let rule = item.as_rule();
                let unwrapped = if rule == Rule::loop_item {
                    item.into_inner().next()
                } else {
                    Some(item)
                };
                let item = match unwrapped {
                    Some(i) => i,
                    None => continue,
                };
                match item.as_rule() {
                    Rule::step_decl => {
                        let step_stmt = parse_step_decl(item)?;
                        if let Stmt::Decl(step_decl) = step_stmt {
                            items.push(LoopItemAst::InlineStep(Box::new(step_decl)));
                        }
                    }
                    Rule::use_step => {
                        let mut parts = item.into_inner();
                        let name = parts
                            .next()
                            .ok_or_else(|| parse_codes::invalid_syntax("Expected step name", span))?
                            .as_str()
                            .to_string();
                        let mut alias = None;
                        let mut args = Vec::new();
                        for p in parts {
                            match p.as_rule() {
                                Rule::identifier if alias.is_none() => {
                                    alias = Some(p.as_str().to_string());
                                }
                                Rule::expression => {
                                    if let Ok(expr) = parse_expression(p.clone()) {
                                        args.push(expr);
                                    }
                                }
                                _ => {}
                            }
                        }
                        items.push(LoopItemAst::UseStep { name, alias, args });
                    }
                    Rule::attach_gate => {
                        let mut parts = item.into_inner();
                        let gate = parts
                            .next()
                            .ok_or_else(|| parse_codes::invalid_syntax("Expected gate name", span))?
                            .as_str()
                            .to_string();
                        let step = parts
                            .next()
                            .ok_or_else(|| parse_codes::invalid_syntax("Expected step name", span))?
                            .as_str()
                            .to_string();
                        items.push(LoopItemAst::AttachGate { gate, step });
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(Stmt::Decl(Decl::Loop {
        name,
        mode,
        items,
        goal,
        span,
    }))
}

fn parse_run_mode(pair: Pair<Rule>, span: hudhudscript_ast::Span) -> ParseResult<RunModeAst> {
    let s = pair.as_str();
    // Single-keyword modes (keywords are silent, matched via as_str)
    match s {
        "once" => return Ok(RunModeAst::Once),
        "cyclic" => return Ok(RunModeAst::Cyclic),
        "until_converged" => return Ok(RunModeAst::UntilConverged),
        _ => {}
    }
    // Compound modes: times_mode / until_mode have non-silent inner rules
    let inner = pair.into_inner().next()
        .ok_or_else(|| parse_codes::invalid_syntax(format!("Unknown run mode: {}", s), span))?;
    match inner.as_rule() {
        Rule::times_mode => {
            // times_kw is silent; only non-silent child is times_int
            let num_pair = inner.into_inner().next()
                .ok_or_else(|| parse_codes::invalid_syntax("Expected integer in times(N)", span))?;
            let n: u64 = num_pair.as_str().parse().map_err(|_|
                parse_codes::invalid_syntax(format!("Invalid integer in times({}): expected non-negative integer", num_pair.as_str()), span))?;
            Ok(RunModeAst::Times(n))
        }
        Rule::until_mode => {
            // until_kw is silent; only non-silent child is expression
            let expr_pair = inner.into_inner().next()
                .ok_or_else(|| parse_codes::invalid_syntax("Expected expression in until(expr)", span))?;
            let expr = parse_expression(expr_pair)?;
            Ok(RunModeAst::Until(expr))
        }
        _ => Err(parse_codes::invalid_syntax(format!("Unknown run mode: {}", s), span)),
    }
}

pub fn parse_chain_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
.next().ok_or_else(|| parse_codes::invalid_syntax("Expected chain name", span))?
        .as_str()
        .to_string();

    let mut mode = RunModeAst::Once;
    let mut links = Vec::new();
    for item in inner {
        match item.as_rule() {
            Rule::run_mode => {
                mode = parse_run_mode(item, span)?;
            }
            Rule::chain_link => {
                let mut parts = item.into_inner();
                let loop_pair = parts.next()
                    .ok_or_else(|| parse_codes::invalid_syntax("Expected loop in chain link", span))?;
                let loop_stmt = parse_loop_decl(loop_pair)?;
                let mut on_done = ChainTargetAst::Next;
                let mut on_fail = ChainTargetAst::ChainFail;
                for p in parts {
                    match p.as_rule() {
                        Rule::chain_on_done => {
                            let target_pair = p.into_inner().next()
                                .ok_or_else(|| parse_codes::invalid_syntax("Expected on_done target", span))?;
                            on_done = parse_chain_target(&target_pair);
                        }
                        Rule::chain_on_fail => {
                            let target_pair = p.into_inner().next()
                                .ok_or_else(|| parse_codes::invalid_syntax("Expected on_fail target", span))?;
                            on_fail = parse_chain_target(&target_pair);
                        }
                        _ => {}
                    }
                }
                if let Stmt::Decl(loop_decl) = loop_stmt {
                    let loop_name = if let Decl::Loop { name, .. } = &loop_decl { name.clone() } else { String::new() };
                    links.push(ChainLinkAst {
                        loop_name: loop_name.clone(),
                        inline_loop: Some(Box::new(loop_decl)),
                        on_done,
                        on_fail,
                    });
                }
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::Chain {
        name,
        mode,
        links,
        span,
    }))
}

pub fn parse_attach_step_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let mut targets = Vec::new();
    let mut loop_name = String::new();

    for item in inner {
        match item.as_rule() {
            Rule::step_with_gate => {
                let mut parts = item.into_inner();
                let step = parts.next()
                    .ok_or_else(|| parse_codes::invalid_syntax("Expected step name", span))?
                    .as_str().to_string();
                let mut gate = None;
                for p in parts {
                    if p.as_rule() == Rule::gate_kw {
                        // next should be gate identifier
                    } else {
                        gate = Some(p.as_str().to_string());
                    }
                }
                targets.push(AttachStepTarget { step, gate });
            }
            Rule::identifier => {
                loop_name = item.as_str().to_string();
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::AttachStep { targets, loop_name, span }))
}

fn parse_chain_target(pair: &Pair<Rule>) -> ChainTargetAst {
    let s = pair.as_str();
    match s {
        "done" => ChainTargetAst::ChainDone,
        "fail" => ChainTargetAst::ChainFail,
        s if s.starts_with("loop ") => {
            let rest = s.trim_start_matches("loop ");
            ChainTargetAst::Loop(rest.to_string())
        }
        _ => ChainTargetAst::Next,
    }
}

pub fn parse_run_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let name = inner
.next().ok_or_else(|| parse_codes::invalid_syntax("Expected loop name", span))?
        .as_str()
        .to_string();
    Ok(Stmt::Decl(Decl::RunLoop { name, span }))
}

pub fn parse_run_chain_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let name = inner.next().ok_or_else(|| parse_codes::invalid_syntax("Expected chain name", span))?.as_str().to_string();
    Ok(Stmt::Decl(Decl::RunChain { name, span }))
}
