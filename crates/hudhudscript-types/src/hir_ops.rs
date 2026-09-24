//! Literal and binary operator conversions for HIR lowering.

use hudhudscript_ast::{BinaryOp, Literal};
use crate::hir::{HirBinOp, HirExpr};
use crate::hir_lower::{reject, HirLowerError};

pub(crate) fn lower_literal(lit: &Literal) -> Result<HirExpr, HirLowerError> {
    Ok(match lit {
        Literal::Int(v) => HirExpr::IntLit(*v),
        Literal::Number(v, _) => HirExpr::FloatLit(*v),
        Literal::Boolean(v) => HirExpr::BoolLit(*v),
        Literal::Null => HirExpr::NullLit,
        Literal::String(s) => HirExpr::StringLit(s.clone()),
        Literal::BigInt(text) => {
            return Err(reject(
                "BigInt literal",
                &format!("`{text}` exceeds i64 (BigInt lane arrives with the runtime ABI)"),
            ));
        }
    })
}

pub(crate) fn bin_op(op: BinaryOp) -> Result<HirBinOp, HirLowerError> {
    Ok(match op {
        BinaryOp::Add => HirBinOp::Add,
        BinaryOp::Sub => HirBinOp::Sub,
        BinaryOp::Mul => HirBinOp::Mul,
        BinaryOp::Div => HirBinOp::Div,
        BinaryOp::Mod => HirBinOp::Rem,
        BinaryOp::And => HirBinOp::And,
        BinaryOp::Or => HirBinOp::Or,
        BinaryOp::Eq => HirBinOp::Eq,
        BinaryOp::Ne => HirBinOp::Ne,
        BinaryOp::Lt => HirBinOp::Lt,
        BinaryOp::Le => HirBinOp::Le,
        BinaryOp::Gt => HirBinOp::Gt,
        BinaryOp::Ge => HirBinOp::Ge,
        other => return Err(reject("binary op", &format!("{other:?} (logical ops arrive with the bool lane)"))),
    })
}

pub(crate) fn has_return_value(stmts: &[hudhudscript_ast::Stmt]) -> bool {
    stmts.iter().any(|s| match s {
        hudhudscript_ast::Stmt::Return { value: Some(_), .. } => true,
        hudhudscript_ast::Stmt::If { then_branch, else_branch, .. } => {
            let t = has_return_value(&lower_block_as_vec(then_branch));
            let e = else_branch.as_ref()
                .map(|b| has_return_value(&lower_block_as_vec(b)))
                .unwrap_or(false);
            t || e
        }
        hudhudscript_ast::Stmt::While { body, .. } => has_return_value(&lower_block_as_vec(body)),
        hudhudscript_ast::Stmt::Block { statements, .. } => has_return_value(statements),
        hudhudscript_ast::Stmt::Try { try_block, catch_clause, finally_block, .. } => {
            let t = has_return_value(&lower_block_as_vec(try_block));
            let c = catch_clause.as_ref()
                .map(|cl| has_return_value(&lower_block_as_vec(&cl.body)))
                .unwrap_or(false);
            let f = finally_block.as_ref()
                .map(|fb| has_return_value(&lower_block_as_vec(fb)))
                .unwrap_or(false);
            t || c || f
        }
        _ => false,
    })
}

pub(crate) fn lower_block_as_vec(stmt: &hudhudscript_ast::Stmt) -> Vec<hudhudscript_ast::Stmt> {
    match stmt {
        hudhudscript_ast::Stmt::Block { statements, .. } => statements.clone(),
        other => vec![other.clone()],
    }
}

pub(crate) fn unsupported_stmt(stmt: &hudhudscript_ast::Stmt) -> HirLowerError {
    match stmt {
        hudhudscript_ast::Stmt::Switch { .. } => reject("switch statement", "switch statement is not lowered to typed HIR"),
        hudhudscript_ast::Stmt::Match { .. } => reject("match statement", "match statement is not lowered to typed HIR"),
        hudhudscript_ast::Stmt::Destructure { .. } => reject("destructure statement", "destructuring is not lowered to typed HIR"),
        hudhudscript_ast::Stmt::ForRange { .. } => reject("for range", "range for loops arrive with the for-in lane"),
        hudhudscript_ast::Stmt::Import { .. } => reject("import statement", "module imports are resolved by VM"),
        hudhudscript_ast::Stmt::Export { .. } => reject("export statement", "module exports are resolved by VM"),
        hudhudscript_ast::Stmt::Class(..) => reject("class declaration", "class declarations are lowered by ClassTable"),
        hudhudscript_ast::Stmt::Trait { .. } => reject("trait declaration", "traits are checked by type checker"),
        hudhudscript_ast::Stmt::Decl(decl) => reject("declaration", &format!("{decl:?}")),
        other => reject("statement", &format!("{:?}", std::mem::discriminant(other))),
    }
}

pub(crate) fn unsupported_expr(expr: &hudhudscript_ast::Expr) -> HirLowerError {
    match expr {
        hudhudscript_ast::Expr::Await { .. } => reject("await expression", "async/await requires async runtime"),
        hudhudscript_ast::Expr::Yield { .. } => reject("yield expression", "generators require generator runtime"),
        hudhudscript_ast::Expr::Perform { .. } => reject("perform expression", "agent actions are handled by VM"),
        hudhudscript_ast::Expr::Recall { .. } => reject("recall expression", "vector memory is handled by VM"),
        hudhudscript_ast::Expr::Spawn { .. } => reject("spawn expression", "actor spawn is handled by VM"),
        hudhudscript_ast::Expr::OptionalMember { .. } => reject("optional member", "optional chaining is handled by VM"),
        other => reject("expression", &format!("{:?}", std::mem::discriminant(other))),
    }
}


