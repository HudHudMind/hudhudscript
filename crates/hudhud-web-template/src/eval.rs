//! Template evaluator — walks AST and renders to string.
//!
//! Handles: variable output (auto-escaped), if/elif/else, for loops
//! with loop.index, template inheritance (extends/blocks), and includes.

use super::ast::{Expr, Node};
use super::filters;
use super::lexer;
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;
use std::path::PathBuf;

/// Context passed to the evaluator — maps template variable names to values.
pub type Context = hudhudscript_bytecode::ObjMap;

/// Template root directory for resolving extends/include paths.
#[derive(Clone)]
pub struct EvalConfig {
    pub template_root: PathBuf,
}

impl Default for EvalConfig {
    fn default() -> Self {
        EvalConfig {
            template_root: PathBuf::from("."),
        }
    }
}

/// Evaluate a parsed template with the given context.
pub fn eval(nodes: &[Node], ctx: &Context, config: &EvalConfig) -> Result<String, String> {
    // Convert to evaluable nodes (resolve extends/blocks)
    let resolved = resolve_inheritance(nodes.to_vec(), config)?;
    let mut out = String::new();
    eval_nodes(&resolved, ctx, config, &mut out, &mut LoopState::default())?;
    Ok(out)
}

#[derive(Default)]
struct LoopState {
    index: usize,
}

fn eval_nodes(
    nodes: &[Node],
    ctx: &Context,
    config: &EvalConfig,
    out: &mut String,
    loop_state: &mut LoopState,
) -> Result<(), String> {
    for node in nodes {
        eval_node(node, ctx, config, out, loop_state)?;
    }
    Ok(())
}

fn eval_node(
    node: &Node,
    ctx: &Context,
    config: &EvalConfig,
    out: &mut String,
    loop_state: &mut LoopState,
) -> Result<(), String> {
    match node {
        Node::Text(s) => out.push_str(s),
        Node::Variable(expr) => {
            let val = eval_expr(expr, ctx, loop_state);
            out.push_str(&filters::value_to_display(&val));
        }
        Node::If {
            conditions,
            else_body,
        } => {
            let mut matched = false;
            for (cond, body) in conditions {
                let val = eval_expr(cond, ctx, loop_state);
                if is_truthy(&val) {
                    eval_nodes(body, ctx, config, out, loop_state)?;
                    matched = true;
                    break;
                }
            }
            if !matched {
                eval_nodes(else_body, ctx, config, out, loop_state)?;
            }
        }
        Node::For {
            var,
            iter,
            body,
            else_body,
        } => {
            let iter_val = eval_expr(iter, ctx, loop_state);
            if let Some(arr) = iter_val.as_array() {
                if arr.is_empty() {
                    eval_nodes(else_body, ctx, config, out, loop_state)?;
                } else {
                    let mut child_state = LoopState::default();
                    for (idx, item) in arr.iter().enumerate() {
                        child_state.index = idx + 1; // 1-based
                        let mut local_ctx = ctx.clone();
                        local_ctx.insert(var.clone(), item.clone());
                        // loop variable
                        let mut loop_obj = hudhudscript_bytecode::ObjMap::default();
                        loop_obj.insert(
                            "index".to_string(),
                            Value16::number(child_state.index as f64),
                        );
                        local_ctx
                            .insert("loop".to_string(), Value16::object(loop_obj));
                        eval_nodes(body, &local_ctx, config, out, &mut child_state)?;
                    }
                }
            } else if let Some(s) = iter_val.as_str() {
                // iterate over characters
                if s.is_empty() {
                    eval_nodes(else_body, ctx, config, out, loop_state)?;
                } else {
                    let mut child_state = LoopState::default();
                    for (idx, ch) in s.chars().enumerate() {
                        child_state.index = idx + 1;
                        let mut local_ctx = ctx.clone();
                        local_ctx.insert(
                            var.clone(),
                            Value16::string(ch.to_string()),
                        );
                        let mut loop_obj = hudhudscript_bytecode::ObjMap::default();
                        loop_obj.insert(
                            "index".to_string(),
                            Value16::number(child_state.index as f64),
                        );
                        local_ctx
                            .insert("loop".to_string(), Value16::object(loop_obj));
                        eval_nodes(body, &local_ctx, config, out, &mut child_state)?;
                    }
                }
            } else {
                eval_nodes(else_body, ctx, config, out, loop_state)?;
            }
        }
        Node::Include(path) => {
            let full_path = config.template_root.join(path);
            let source = std::fs::read_to_string(&full_path).map_err(|e| {
                format!("include {}: {}", full_path.display(), e)
            })?;
            let tokens = lexer::lex(&source);
            let mut parser = super::parser::Parser::new(tokens);
            let included_nodes = parser.parse().map_err(|e| {
                format!("parse error in {}: {}", full_path.display(), e)
            })?;
            eval_nodes(
                &included_nodes,
                ctx,
                config,
                out,
                loop_state,
            )?;
        }
        Node::Extends(_) => {
            // Already handled by resolve_inheritance; skip.
        }
        Node::Block { name: _, body } => {
            // Block body rendered directly (inheritance already resolved).
            eval_nodes(body, ctx, config, out, loop_state)?;
        }
    }
    Ok(())
}

/// Resolve extends/blocks: if the template extends a base, load the base
/// and substitute its blocks with child block definitions.
fn resolve_inheritance(
    nodes: Vec<Node>,
    config: &EvalConfig,
) -> Result<Vec<Node>, String> {
    // Find extends and blocks
    let mut extends_path: Option<String> = None;
    let mut child_blocks: HashMap<String, Vec<Node>> = HashMap::new();
    let mut own_content: Vec<Node> = Vec::new();

    for node in nodes {
        match node {
            Node::Extends(path) => extends_path = Some(path),
            Node::Block { name, body } => {
                child_blocks.insert(name, body);
            }
            _ => own_content.push(node),
        }
    }

    match extends_path {
        Some(path) => {
            // Load base template
            let full_path = config.template_root.join(&path);
            let source = std::fs::read_to_string(&full_path).map_err(|e| {
                format!("extends {}: {}", full_path.display(), e)
            })?;
            let tokens = lexer::lex(&source);
            let mut parser = super::parser::Parser::new(tokens);
            let base_nodes = parser.parse().map_err(|e| {
                format!("parse error in {}: {}", full_path.display(), e)
            })?;

            // Substitute blocks: traverse base, replace blocks with child definitions
            Ok(substitute_blocks(base_nodes, &child_blocks, config)?)
        }
        None => {
            // No inheritance — merge everything
            let mut merged = own_content;
            for (name, body) in child_blocks {
                eprintln!("warning: block '{}' defined without extends", name);
                merged.push(Node::Block { name, body });
            }
            Ok(merged)
        }
    }
}

fn substitute_blocks(
    nodes: Vec<Node>,
    blocks: &HashMap<String, Vec<Node>>,
    config: &EvalConfig,
) -> Result<Vec<Node>, String> {
    let mut result = Vec::new();
    for node in nodes {
        match node {
            Node::Block { name, body } => {
                if let Some(replacement) = blocks.get(&name) {
                    result.extend(replacement.clone());
                } else {
                    result.push(Node::Block { name, body });
                }
            }
            Node::Extends(path) => {
                // Nested extends: load grandparent
                let full_path = config.template_root.join(&path);
                let source = std::fs::read_to_string(&full_path).map_err(|e| {
                    format!("extends {}: {}", full_path.display(), e)
                })?;
                let tokens = lexer::lex(&source);
                let mut parser = super::parser::Parser::new(tokens);
                let grandparent = parser.parse().map_err(|e| {
                    format!("parse error in {}: {}", full_path.display(), e)
                })?;
                let substituted =
                    substitute_blocks(grandparent, blocks, config)?;
                result.extend(substituted);
            }
            _ => result.push(node),
        }
    }
    Ok(result)
}

/// Evaluate an expression → Value16.
fn eval_expr(
    expr: &Expr,
    ctx: &Context,
    loop_state: &LoopState,
) -> Value16 {
    match expr {
        Expr::Ident(name) => ctx.get(name).cloned().unwrap_or(Value16::null()),
        Expr::String(s) => Value16::string(s.clone()),
        Expr::Number(n) => Value16::number(*n),
        Expr::Dot(base, field) => {
            let val = eval_expr(base, ctx, loop_state);
            if let Some(obj) = val.as_object() {
                obj.get(field).cloned().unwrap_or(Value16::null())
            } else {
                Value16::null()
            }
        }
        Expr::Bracket(base, idx) => {
            let val = eval_expr(base, ctx, loop_state);
            let idx_val = eval_expr(idx, ctx, loop_state);
            match (val.as_array(), val.as_object()) {
                (Some(arr), _) => {
                    let i = idx_val.as_number().unwrap_or(0.0) as usize;
                    arr.get(i).cloned().unwrap_or(Value16::null())
                }
                (_, Some(obj)) => {
                    let key = idx_val.as_str().unwrap_or("");
                    obj.get(key).cloned().unwrap_or(Value16::null())
                }
                _ => Value16::null(),
            }
        }
        Expr::Filter(base, name, args) => {
            let val = eval_expr(base, ctx, loop_state);
            let str_args: Vec<String> = args
                .iter()
                .map(|a| {
                    let v = eval_expr(a, ctx, loop_state);
                    v.as_str().unwrap_or("").to_string()
                })
                .collect();
            filters::apply_filter(name, &val, &str_args)
        }
        Expr::Not(inner) => {
            let v = eval_expr(inner, ctx, loop_state);
            Value16::bool_(!is_truthy(&v))
        }
        Expr::Eq(a, b) => {
            let va = eval_expr(a, ctx, loop_state);
            let vb = eval_expr(b, ctx, loop_state);
            Value16::bool_(values_eq(&va, &vb))
        }
        Expr::Neq(a, b) => {
            let va = eval_expr(a, ctx, loop_state);
            let vb = eval_expr(b, ctx, loop_state);
            Value16::bool_(!values_eq(&va, &vb))
        }
        Expr::Lt(a, b) => cmp_values(expr, a, b, ctx, loop_state, |x, y| x < y),
        Expr::Gt(a, b) => cmp_values(expr, a, b, ctx, loop_state, |x, y| x > y),
        Expr::Le(a, b) => cmp_values(expr, a, b, ctx, loop_state, |x, y| x <= y),
        Expr::Ge(a, b) => cmp_values(expr, a, b, ctx, loop_state, |x, y| x >= y),
        Expr::And(a, b) => {
            let va = eval_expr(a, ctx, loop_state);
            if !is_truthy(&va) {
                Value16::bool_(false)
            } else {
                let vb = eval_expr(b, ctx, loop_state);
                Value16::bool_(is_truthy(&vb))
            }
        }
        Expr::Or(a, b) => {
            let va = eval_expr(a, ctx, loop_state);
            if is_truthy(&va) {
                Value16::bool_(true)
            } else {
                let vb = eval_expr(b, ctx, loop_state);
                Value16::bool_(is_truthy(&vb))
            }
        }
    }
}

fn is_truthy(v: &Value16) -> bool {
    if v.is_null() {
        return false;
    }
    if let Some(b) = v.as_bool() {
        return b;
    }
    if let Some(n) = v.as_number() {
        return n != 0.0;
    }
    if let Some(s) = v.as_str() {
        return !s.is_empty();
    }
    if let Some(arr) = v.as_array() {
        return !arr.is_empty();
    }
    true
}

fn values_eq(a: &Value16, b: &Value16) -> bool {
    match (a.as_str(), b.as_str()) {
        (Some(sa), Some(sb)) => sa == sb,
        _ => match (a.as_number(), b.as_number()) {
            (Some(na), Some(nb)) => na == nb,
            _ => match (a.as_bool(), b.as_bool()) {
                (Some(ba), Some(bb)) => ba == bb,
                _ => a.is_null() && b.is_null(),
            },
        },
    }
}

fn cmp_values(
    _expr: &Expr,
    a: &Expr,
    b: &Expr,
    ctx: &Context,
    loop_state: &LoopState,
    op: fn(f64, f64) -> bool,
) -> Value16 {
    let va = eval_expr(a, ctx, loop_state);
    let vb = eval_expr(b, ctx, loop_state);
    let na = va.as_number().unwrap_or(0.0);
    let nb = vb.as_number().unwrap_or(0.0);
    Value16::bool_(op(na, nb))
}
