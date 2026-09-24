//! Fast AST pre-check (<0.05ms) for early VM fallback.
//!
//! Scans AST to detect unsupported features and statically guaranteed
//! integer overflows before spending CPU time on HIR/MIR/Cranelift pipeline.

use hudhudscript_ast::{ClassMember, Expr, Literal, Stmt};

/// Result of pre-checking an AST for JIT execution capability.
pub fn quick_precheck(stmts: &[Stmt]) -> Result<(), String> {
    let trace = std::env::var("HUDHUD_JIT_TRACE").is_ok();
    for stmt in stmts {
        if let Err((line, reason)) = check_stmt(stmt) {
            if trace {
                eprintln!("[jit:trace] Fallback to VM at line {line}: {reason}");
            }
            return Err(format!("line {line}: {reason}"));
        }
    }
    Ok(())
}

fn check_stmt(stmt: &Stmt) -> Result<(), (usize, String)> {
    match stmt {
        Stmt::Import { span, .. } => Err((span.start.line, "module imports require VM module resolver".into())),
        Stmt::Export { span, .. } => Err((span.start.line, "module exports require VM module resolver".into())),
        Stmt::Decl(hudhudscript_ast::Decl::Import { span, .. }) => {
            Err((span.start.line, "import declarations require VM module resolver".into()))
        }
        Stmt::Function { body, span, .. } => {
            for s in body {
                check_stmt(s)?;
            }
            // Check for guaranteed recursive overflow (e.g. fib(94+), fact(21+))
            check_function_overflow(body, span.start.line)?;
            Ok(())
        }
        Stmt::Block { statements, .. } => {
            for s in statements {
                check_stmt(s)?;
            }
            Ok(())
        }
        Stmt::If { condition, then_branch, else_branch, .. } => {
            check_expr(condition)?;
            check_stmt(then_branch)?;
            if let Some(e) = else_branch {
                check_stmt(e)?;
            }
            Ok(())
        }
        Stmt::While { condition, body, span } => {
            check_expr(condition)?;
            check_stmt(body)?;
            check_loop_overflow(condition, body, span.start.line)?;
            Ok(())
        }
        Stmt::Let { value, .. } => check_expr(value),
        Stmt::VarDecl(v) => {
            if let Some(ref init) = v.initializer {
                check_expr(init)?;
            }
            Ok(())
        }
        Stmt::Assignment { target, value, .. } => {
            check_expr(target)?;
            check_expr(value)
        }
        Stmt::Expr(e) => check_expr(e),
        Stmt::Return { value: Some(e), .. } => check_expr(e),
        Stmt::Return { value: None, .. } => Ok(()),
        Stmt::Break { .. } | Stmt::Continue { .. } => Ok(()),
        Stmt::Try { try_block, catch_clause, finally_block, .. } => {
            check_stmt(try_block)?;
            if let Some(c) = catch_clause {
                check_stmt(&c.body)?;
            }
            if let Some(f) = finally_block {
                check_stmt(f)?;
            }
            Ok(())
        }
        Stmt::Throw { value, .. } => check_expr(value),
        Stmt::Class(c) => {
            for m in &c.members {
                if let ClassMember::Method { body, .. } = m {
                    for s in body {
                        check_stmt(s)?;
                    }
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn check_expr(expr: &Expr) -> Result<(), (usize, String)> {
    match expr {
        Expr::Literal(Literal::BigInt(text), span) => {
            Err((span.start.line, format!("BigInt literal `{text}` exceeds i64 range (requires VM)")))
        }
        Expr::Await { span, .. } => Err((span.start.line, "await expression requires async runtime (VM)".into())),
        Expr::Yield { span, .. } => Err((span.start.line, "yield expression requires generator runtime (VM)".into())),
        Expr::Perform { span, .. } => Err((span.start.line, "perform expression requires agentic runtime (VM)".into())),
        Expr::Spawn { args, .. } => {
            for a in args {
                check_expr(a)?;
            }
            Ok(())
        }
        Expr::Binary { left, right, .. } => {
            check_expr(left)?;
            check_expr(right)
        }
        Expr::Unary { expr: inner, .. } => check_expr(inner),
        Expr::Call { callee, args, span } => {
            check_expr(callee)?;
            for a in args {
                check_expr(a)?;
            }
            Ok(())
        }
        Expr::Array { elements, .. } => {
            for e in elements {
                check_expr(e)?;
            }
            Ok(())
        }
        Expr::Index { object, index, .. } => {
            check_expr(object)?;
            check_expr(index)
        }
        Expr::Member { object, .. } => check_expr(object),
        Expr::Object { properties, .. } => {
            for (_, v) in properties {
                check_expr(v)?;
            }
            Ok(())
        }
        Expr::Ternary { condition, true_expr, false_expr, .. } => {
            check_expr(condition)?;
            check_expr(true_expr)?;
            check_expr(false_expr)
        }
        _ => Ok(()),
    }
}

/// Loop overflow check: JIT has native BigInt promotion, so loops no longer need to be rejected to VM.
fn check_loop_overflow(_cond: &Expr, _body: &Stmt, _line: usize) -> Result<(), (usize, String)> {
    Ok(())
}

fn check_function_overflow(body: &[Stmt], line: usize) -> Result<(), (usize, String)> {
    for s in body {
        if let Stmt::While { condition, body: loop_body, .. } = s {
            check_loop_overflow(condition, loop_body, line)?;
        }
    }
    Ok(())
}
