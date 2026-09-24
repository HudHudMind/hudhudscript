//! Typed HIR — AŞAMA-0 skeleton (JIT_AOT_ARCHITECTURE.md §4.3).
//!
//! HIR = AST + resolved names + type annotations, in tree form (control
//! flow lowers to blocks in MIR). Purely additive: no existing consumer
//! changes; the native pipeline (`hudhudscript-mir`) builds on this.
//!
//! Source-level types come from this crate's [`Type`] lattice (the dynamic
//! language checker). Machine-level refinement (i64/f64 specialization)
//! happens at the HIR→MIR boundary, not here.

use std::collections::BTreeMap;
use std::fmt;

use crate::types::Type;

/// A whole compilation unit in typed high-level form.
#[derive(Debug, Clone, Default)]
pub struct HirModule {
    /// Top-level functions, keyed by name (deterministic iteration).
    pub functions: BTreeMap<String, HirFunction>,
}

/// A fully typed function.
#[derive(Debug, Clone, PartialEq)]
pub struct HirFunction {
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_type: Type,
    pub body: Vec<HirStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirParam {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirStmt {
    Let {
        name: String,
        ty: Type,
        value: HirExpr,
    },
    Assign {
        name: String,
        value: HirExpr,
    },
    Expr(HirExpr),
    Return(Option<HirExpr>),
    If {
        cond: HirExpr,
        then_branch: Vec<HirStmt>,
        else_branch: Vec<HirStmt>,
    },
    While {
        cond: HirExpr,
        body: Vec<HirStmt>,
    },
    /// arr[index] = value
    ArrayStore {
        array: HirExpr,
        index: HirExpr,
        value: HirExpr,
    },
    /// Döngü kırma: en içteki while'ın çıkışına dallanır
    Break,
    /// Döngü başına geri dön
    Continue,
    /// obj.name = value
    PropertySet {
        object: HirExpr,
        name: String,
        value: HirExpr,
    },
    /// Try-catch-finally statement
    Try {
        try_body: Vec<HirStmt>,
        catch_param: Option<String>,
        catch_body: Vec<HirStmt>,
        finally_body: Vec<HirStmt>,
    },
    /// Throw statement
    Throw(HirExpr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirBinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirUnOp {
    Neg,
    Not,
}

/// Typed expression. Every non-literal node carries its inferred/annotated
/// [`Type`] so the MIR lowering never re-infers.
#[derive(Debug, Clone, PartialEq)]
pub enum HirExpr {
    Local {
        name: String,
        ty: Type,
    },
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),
    NullLit,
    Binary {
        op: HirBinOp,
        lhs: Box<HirExpr>,
        rhs: Box<HirExpr>,
        ty: Type,
    },
    Unary {
        op: HirUnOp,
        operand: Box<HirExpr>,
        ty: Type,
    },
    Call {
        callee: String,
        args: Vec<HirExpr>,
        ty: Type,
    },
    /// Array literal: [1, 2, 3]
    ArrayLit {
        elements: Vec<HirExpr>,
        ty: Type,
    },
    /// Array indexing: arr[index]
    ArrayIndex {
        array: Box<HirExpr>,
        index: Box<HirExpr>,
        ty: Type,
    },
    /// Array element assignment target: arr[index] = value
    ArrayStore {
        array: Box<HirExpr>,
        index: Box<HirExpr>,
    },
    /// Array method call: arr.push(v), arr.pop(), arr.length
    ArrayMethod {
        array: Box<HirExpr>,
        method: String,
        args: Vec<HirExpr>,
        ty: Type,
    },
    /// Object literal: {x: 1, y: 2}
    ObjectLit {
        properties: Vec<(String, HirExpr)>,
        ty: Type,
    },
    /// Property access: obj.name
    PropertyGet {
        object: Box<HirExpr>,
        name: String,
        ty: Type,
    },
    /// Ternary: condition ? true_expr : false_expr
    Ternary {
        condition: Box<HirExpr>,
        true_expr: Box<HirExpr>,
        false_expr: Box<HirExpr>,
        ty: Type,
    },
}

impl HirExpr {
    /// Static source type of this expression.
    pub fn ty(&self) -> Type {
        match self {
            HirExpr::Local { ty, .. } => ty.clone(),
            HirExpr::IntLit(_) | HirExpr::FloatLit(_) => Type::Number,
            HirExpr::StringLit(_) => Type::String,
            HirExpr::BoolLit(_) => Type::Boolean,
            HirExpr::NullLit => Type::Null,
            HirExpr::Binary { ty, .. } => ty.clone(),
            HirExpr::Unary { ty, .. } => ty.clone(),
            HirExpr::Call { ty, .. } => ty.clone(),
            HirExpr::ArrayLit { ty, .. } => ty.clone(),
            HirExpr::ArrayIndex { ty, .. } => ty.clone(),
            HirExpr::ArrayStore { .. } => Type::Any,
            HirExpr::ArrayMethod { ty, .. } => ty.clone(),
            HirExpr::ObjectLit { ty, .. } => ty.clone(),
            HirExpr::PropertyGet { ty, .. } => ty.clone(),
            HirExpr::Ternary { ty, .. } => ty.clone(),
        }
    }
}

impl fmt::Display for HirBinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            HirBinOp::Add => "+",
            HirBinOp::Sub => "-",
            HirBinOp::Mul => "*",
            HirBinOp::Div => "/",
            HirBinOp::Rem => "%",
            HirBinOp::Eq => "==",
            HirBinOp::Ne => "!=",
            HirBinOp::Lt => "<",
            HirBinOp::Le => "<=",
            HirBinOp::Gt => ">",
            HirBinOp::Ge => ">=",
            HirBinOp::And => "&&",
            HirBinOp::Or => "||",
        };
        f.write_str(s)
    }
}

impl HirFunction {
    /// Build `fn add(a: int64, b: int64) -> int64 { return a + b }` — the
    /// canonical AŞAMA-0 example used across the architecture document.
    pub fn example_add() -> Self {
        HirFunction {
            name: "add".to_string(),
            params: vec![
                HirParam { name: "a".into(), ty: Type::Number },
                HirParam { name: "b".into(), ty: Type::Number },
            ],
            return_type: Type::Number,
            body: vec![HirStmt::Return(Some(HirExpr::Binary {
                op: HirBinOp::Add,
                lhs: Box::new(HirExpr::Local { name: "a".into(), ty: Type::Number }),
                rhs: Box::new(HirExpr::Local { name: "b".into(), ty: Type::Number }),
                ty: Type::Number,
            }))],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_add_shape() {
        let f = HirFunction::example_add();
        assert_eq!(f.name, "add");
        assert_eq!(f.params.len(), 2);
        assert_eq!(f.params[0].name, "a");
        assert_eq!(f.params[0].ty, Type::Number);
        assert_eq!(f.return_type, Type::Number);
        assert_eq!(f.body.len(), 1);
        match &f.body[0] {
            HirStmt::Return(Some(HirExpr::Binary { op, lhs, rhs, ty })) => {
                assert_eq!(*op, HirBinOp::Add);
                assert!(matches!(lhs.as_ref(), HirExpr::Local { name, .. } if name == "a"));
                assert!(matches!(rhs.as_ref(), HirExpr::Local { name, .. } if name == "b"));
                assert_eq!(*ty, Type::Number);
            }
            other => panic!("expected return of binary add, got {other:?}"),
        }
    }

    #[test]
    fn expr_ty_reports_source_type() {
        assert_eq!(HirExpr::IntLit(3).ty(), Type::Number);
        assert_eq!(HirExpr::FloatLit(1.5).ty(), Type::Number);
        assert_eq!(HirExpr::BoolLit(true).ty(), Type::Boolean);
        assert_eq!(HirExpr::StringLit("x".into()).ty(), Type::String);
        assert_eq!(HirExpr::NullLit.ty(), Type::Null);
    }

    #[test]
    fn module_holds_functions_deterministically() {
        let mut m = HirModule::default();
        let add = HirFunction::example_add();
        m.functions.insert("add".into(), add.clone());
        m.functions.insert("main".into(), HirFunction {
            name: "main".into(),
            params: vec![],
            return_type: Type::Null,
            body: vec![HirStmt::Expr(HirExpr::Call {
                callee: "print".into(),
                args: vec![HirExpr::IntLit(5)],
                ty: Type::Null,
            })],
        });
        let names: Vec<&str> = m.functions.keys().map(|s| s.as_str()).collect();
        assert_eq!(names, ["add", "main"]); // BTreeMap → alfabetik
        assert!(m.functions.contains_key("add"));
        assert_eq!(m.functions["main"].body.len(), 1);
    }

    #[test]
    fn bin_op_display() {
        assert_eq!(HirBinOp::Add.to_string(), "+");
        assert_eq!(HirBinOp::Le.to_string(), "<=");
        assert_eq!(HirBinOp::And.to_string(), "&&");
    }
}
