//! AST to Bytecode compiler utilities

use crate::bytecode::SymId;
use hudhudscript_ast::{BinaryOp, Expr, Literal};

/// Intern a string into the global interner and return a compact `SymId`.
/// Convenience wrapper used by code that cannot go through `CompileTarget::ct_sym`.
#[inline]
pub(super) fn sym(s: &str) -> SymId {
    SymId(hudhudscript_bytecode::interner::intern(s).0)
}

/// Compute signed relative offset from a jump instruction's IP to an
/// absolute target IP.  Used everywhere we emit/patch Jump/JumpIfFalse/
/// JumpIfTrue/IterNext/TryBegin/FinallyBegin/FinallyExit which now carry an
/// `i32` relative offset (Audit v3 Finding 4.2, BYTECODE_VERSION 6).
///
/// Backward jumps produce a negative offset; forward jumps positive.
#[inline]
pub(super) fn jump_off(jump_site: usize, target: usize) -> i32 {
    (target as i64 - jump_site as i64) as i32
}

/// A local variable tracked during compilation.
#[inline]
pub(super) fn ends_with_return(stmt: &hudhudscript_ast::Stmt) -> bool {
    use hudhudscript_ast::Stmt;
    match stmt {
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Throw { .. } => true,
        Stmt::Block { statements, .. } => {
            statements.last().map(|s| ends_with_return(s)).unwrap_or(false)
        }
        Stmt::If { then_branch, else_branch, .. } => {
            let then_ends = ends_with_return(then_branch);
            else_branch.as_ref().map(|e| then_ends && ends_with_return(e)).unwrap_or(false)
        }
        _ => false,
    }
}

/// A local variable tracked during compilation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(super) struct Local {
    pub(super) name: String,
    pub(super) depth: usize,
    pub(super) is_captured: bool,
    /// S2.2c: register index assigned to this local variable.
    /// `FunctionChunk::local_names` keeps the debug name for the same index.
    pub(super) slot: Option<u32>,
    /// K1-1: register index assigned to this local variable.
    /// `None` for globals / pre-register locals.
    pub(super) reg: Option<u8>,
    /// K1-3: compile-time const flag — true for `const` locals.
    pub(super) is_const: bool,
    /// ISSUE-1: compile-time known type for loop-counter inference.
    pub(super) known_type: crate::compiler::expr::ExprType,
}

/// Walk a member/this expression chain to find the root variable name.
/// Identical logic previously duplicated in both `Compiler` and `FunctionCompiler`.
pub(super) fn root_var_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Identifier(name, _) => Some(name.clone()),
        Expr::This(_) => Some("this".to_string()),
        Expr::Member { object, .. } => root_var_name(object),
        Expr::Call { callee, .. } => root_var_name(callee),
        _ => None,
    }
}

/// Issue #474 / Kural 2: serialize a constitution/law rule `Expr` back to
/// a source-shaped string that `hudhudscript_builtins::governance_ops::parse_rule_to_condition`
/// can re-parse at runtime.  Most scripts already write rules as string
/// literals (`"data_size < 1000"`) — for those we pass the string through
/// unchanged.  Structured comparison/logical exprs fall back to a
/// best-effort source reconstruction covering the cases supported by the
/// runtime condition parser.
pub(super) fn expr_to_rule_string(expr: &Expr) -> String {
    match expr {
        Expr::Literal(Literal::String(s), _) => s.clone(),
        Expr::Literal(Literal::Number(n, _), _) => {
            if n.fract() == 0.0 && n.abs() < 1e16 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        Expr::Literal(Literal::Int(i), _) => format!("{}", i),
        Expr::Literal(Literal::BigInt(s), _) => s.clone(),
        Expr::Literal(Literal::Boolean(b), _) => b.to_string(),
        Expr::Literal(Literal::Null, _) => "null".to_string(),
        Expr::Identifier(name, _) => name.clone(),
        Expr::Binary {
            op, left, right, ..
        } => {
            let lhs = expr_to_rule_string(left);
            let rhs = expr_to_rule_string(right);
            let op_str = match op {
                BinaryOp::Eq => "==",
                BinaryOp::Ne => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                BinaryOp::And => "AND",
                BinaryOp::Or => "OR",
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Mod => "%",
                BinaryOp::NullCoalesce => "??",
                BinaryOp::InstanceOf => "instanceof",
            };
            format!("{} {} {}", lhs, op_str, rhs)
        }
        Expr::Array { elements, .. } => {
            let parts: Vec<String> = elements.iter().map(expr_to_rule_string).collect();
            format!("[{}]", parts.join(", "))
        }
        // Anything else (calls, member access, etc.) — fall back to
        // Debug format.  Runtime `parse_rule_to_condition` will
        // fail-close on these, matching the existing safety default.
        other => format!("{:?}", other),
    }
}

/// If `expr` is a chain of `+` operations whose left-most leaf is
/// `Identifier(name, _)`, returns the list of right-hand expressions
/// in evaluation order (left-to-right, depth-first).
///
/// Example: `name + a + b` → `Some(vec![a, b])`
/// Non-matching: `a + name` → `None`
/// Non-matching: `name` (no Add) → `None`
pub(super) fn decompose_add_chain<'a>(expr: &'a Expr, name: &str) -> Option<Vec<&'a Expr>> {
    fn walk<'a>(expr: &'a Expr, name: &str, out: &mut Vec<&'a Expr>) -> bool {
        match expr {
            Expr::Binary { left, op: BinaryOp::Add, right, .. } => {
                if walk(left, name, out) {
                    out.push(right);
                    true
                } else {
                    false
                }
            }
            Expr::Identifier(n, _) if n == name => true,
            _ => false,
        }
    }
    let mut out = Vec::new();
    if walk(expr, name, &mut out) && !out.is_empty() {
        Some(out)
    } else {
        None
    }
}

/// C3 helper: returns true if the statement tree contains a `break` or
/// `continue` that targets the immediately enclosing loop.  Nested loops
/// are a boundary, but switch is NOT a boundary for `continue` — a
/// `continue` inside a switch still targets the enclosing loop.
/// A plain `break` inside a switch targets the switch, so it is ignored.
pub(crate) fn body_contains_loop_exit(body: &hudhudscript_ast::Stmt) -> bool {
    body_contains_loop_exit_impl(body, false)
}

fn body_contains_loop_exit_impl(body: &hudhudscript_ast::Stmt, in_switch: bool) -> bool {
    use hudhudscript_ast::Stmt;
    match body {
        Stmt::Break { .. } => !in_switch,
        Stmt::Continue { .. } => true,
        Stmt::Block { statements, .. } => {
            statements.iter().any(|s| body_contains_loop_exit_impl(s, in_switch))
        }
        Stmt::If { then_branch, else_branch, .. } => {
            body_contains_loop_exit_impl(then_branch, in_switch)
                || else_branch.as_ref().map_or(false, |e| body_contains_loop_exit_impl(e, in_switch))
        }
        Stmt::While { .. }
        | Stmt::For { .. }
        | Stmt::ForCStyle { .. }
        | Stmt::ForRange { .. } => {
            // Inner loop forms its own break/continue-target boundary.
            false
        }
        Stmt::Switch { cases, default, .. } => {
            // Inside a switch, `break` exits the switch — not the loop.
            // But `continue` inside a switch still targets the loop.
            let case_has_continue = cases.iter().any(|c| {
                c.body.iter().any(|s| body_contains_loop_exit_impl(s, true))
            });
            let default_has_continue = default
                .as_ref()
                .map_or(false, |stmts| stmts.iter().any(|s| body_contains_loop_exit_impl(s, true)));
            case_has_continue || default_has_continue
        }
        Stmt::Try { try_block, catch_clause, finally_block, .. } => {
            body_contains_loop_exit_impl(try_block, in_switch)
                || catch_clause.as_ref().map_or(false, |c| body_contains_loop_exit_impl(&c.body, in_switch))
                || finally_block.as_ref().map_or(false, |b| body_contains_loop_exit_impl(b, in_switch))
        }
        _ => false,
    }
}
