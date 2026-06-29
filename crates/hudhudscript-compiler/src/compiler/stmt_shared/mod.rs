use super::*;

pub(super) fn compile_stmt_shared(
    target: &mut impl CompileTarget,
    stmt: &Stmt,
) -> CompileResult<()> {
    target.ct_mark_stmt_pos(&stmt.span());
    match stmt {
        Stmt::Let { .. }
        | Stmt::Const { .. }
        | Stmt::Assignment { .. }
        | Stmt::Expr(_)
        | Stmt::Return { .. }
        | Stmt::If { .. }
        | Stmt::While { .. }
        | Stmt::For { .. }
        | Stmt::ForCStyle { .. }
        | Stmt::ForRange { .. } => compile_stmt_part1(target, stmt)?,

        Stmt::Break { .. }
        | Stmt::Continue { .. }
        | Stmt::Switch { .. }
        | Stmt::Try { .. }
        | Stmt::Throw { .. }
        | Stmt::Block { .. }
        | Stmt::Function { .. }
        | Stmt::Class(_)
        | Stmt::EnumDecl { .. }
        | Stmt::Match { .. }
        | Stmt::Import { .. }
        | Stmt::Export { .. } => compile_stmt_part2(target, stmt)?,

        Stmt::VarDecl(_)
        | Stmt::McpServer(_)
        | Stmt::Spawn { .. }
        | Stmt::Despawn { .. }
        | Stmt::Send { .. }
        | Stmt::Receive { .. }
        | Stmt::Require { .. }
        | Stmt::Perform { .. }
        | Stmt::Remember { .. }
        | Stmt::Recall { .. }
        | Stmt::Forget { .. }
        | Stmt::Destructure { .. }
        | Stmt::Trait { .. }
        | Stmt::Decl(_) => compile_stmt_part3(target, stmt)?,
    }
    Ok(())
}

pub mod core;
use core::compile_stmt_part1;
pub mod declarations;
use declarations::compile_stmt_part2;
pub mod special;
use special::compile_stmt_part3;
pub mod loops;
pub mod assignment;
pub mod assignment_fusions;
