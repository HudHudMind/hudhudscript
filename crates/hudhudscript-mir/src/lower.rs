//! HIR → MIR lowering for the AŞAMA-0 subset
//! (JIT_AOT_ARCHITECTURE.md §5.3 example: `fn main() { print(2 + 3) }`).
//!
//! Machine refinement rules at this boundary (documented, then widened):
//! - `IntLit`    → `i64` (ConstInt)
//! - `FloatLit`  → `f64` (ConstFloat)
//! - `Bool/Null` → bool / generic lanes
//! - binary on two i64 literals/locals → i64 checked lane
//! - `Call` to `print` → CallNative(Print)
//! Anything else is a clean `LowerError::Unsupported` — the lowering
//! NEVER guesses semantics (protocol: reject, don't approximate).

use std::fmt;

use hudhudscript_types::{HirBinOp, HirExpr, HirFunction, HirStmt};

use crate::builder::{BinOp, MirFunctionBuilder};
use crate::mir::{CmpOp, MirType, RuntimeHelperId, TrapKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerError {
    Unsupported { function: String, reason: String },
}

impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LowerError::Unsupported { function, reason } => {
                write!(f, "cannot lower `{function}`: {reason}")
            }
        }
    }
}

impl std::error::Error for LowerError {}

/// Lower a typed HIR function to verified MIR.
pub fn lower_function(hir: &HirFunction) -> Result<crate::mir::MirFunction, LowerError> {
    let mut b = MirFunctionBuilder::new(
        &hir.name,
        vec![],
        machine_return_ty(&hir.return_type)?,
    );
    let entry = b.entry();
    let return_ty = machine_return_ty(&hir.return_type)?;
    let terminator_value = lower_stmts(hir, &mut b, entry, &hir.body)?;
    if return_ty == MirType::Unit {
        b.ret_void(entry);
    } else {
        b.ret(entry, terminator_value);
    }
    let f = b.finish();
    crate::verify::verify_function(&f)
        .map_err(|e| LowerError::Unsupported {
            function: hir.name.clone(),
            reason: format!("lowering produced invalid MIR: {e}"),
        })?;
    Ok(f)
}

fn lower_stmts(
    hir: &HirFunction,
    b: &mut MirFunctionBuilder,
    block: crate::mir::BlockId,
    stmts: &[HirStmt],
) -> Result<crate::mir::ValueId, LowerError> {
    let mut last: Option<crate::mir::ValueId> = None;
    for stmt in stmts {
        match stmt {
            HirStmt::Return(Some(expr)) => {
                let v = lower_expr(hir, b, block, expr)?;
                return Ok(v);
            }
            HirStmt::Return(None) => {
                return Ok(b.const_null(block));
            }
            HirStmt::Expr(expr) => {
                last = Some(lower_expr(hir, b, block, expr)?);
            }
            HirStmt::Let { .. } | HirStmt::Assign { .. } => {
                return Err(LowerError::Unsupported {
                    function: hir.name.clone(),
                    reason: "let/assign locals (AŞAMA-0: literal-expression subset)".into(),
                });
            }
        HirStmt::ArrayStore { .. } | HirStmt::PropertySet { .. } => {
                return Err(LowerError::Unsupported {
                    function: hir.name.clone(),
                    reason: "array/property store (typed lane only)".into(),
                });
            }
            HirStmt::Continue => {
                return Err(LowerError::Unsupported {
                    function: hir.name.clone(),
                    reason: "continue (typed lane only)".into(),
                });
            }
            HirStmt::Break => {
                return Err(LowerError::Unsupported {
                    function: hir.name.clone(),
                    reason: "break (typed lane only)".into(),
                });
            }
            HirStmt::If { .. } | HirStmt::While { .. } | HirStmt::Try { .. } | HirStmt::Throw(_) => {
                return Err(LowerError::Unsupported {
                    function: hir.name.clone(),
                    reason: "control flow (arrives with full-CFG lowering)".into(),
                });
            }
        }
    }
    Ok(last.unwrap_or_else(|| b.const_null(block)))
}

fn lower_expr(
    hir: &HirFunction,
    b: &mut MirFunctionBuilder,
    block: crate::mir::BlockId,
    expr: &HirExpr,
) -> Result<crate::mir::ValueId, LowerError> {
    let unsupported = |reason: &str| LowerError::Unsupported {
        function: hir.name.clone(),
        reason: reason.to_string(),
    };
    match expr {
        HirExpr::IntLit(v) => Ok(b.const_i64(block, *v)),
        HirExpr::FloatLit(v) => Ok(b.const_f64(block, *v)),
        HirExpr::BoolLit(v) => Ok(b.const_bool(block, *v)),
        HirExpr::NullLit => Ok(b.const_null(block)),
        HirExpr::StringLit(_) => Err(unsupported("string literals (needs string ABI lane)")),
        HirExpr::ArrayLit { .. } => Err(unsupported("array literals (typed lane only)")),
        HirExpr::ObjectLit { .. } => Err(unsupported("object literals (typed lane only)")),
        HirExpr::PropertyGet { .. } => Err(unsupported("property access (typed lane only)")),
        HirExpr::ArrayIndex { .. } => Err(unsupported("array indexing (typed lane only)")),
        HirExpr::ArrayStore { .. } => Err(unsupported("array store (typed lane only)")),
        HirExpr::ArrayMethod { .. } => Err(unsupported("array methods (typed lane only)")),
        HirExpr::Ternary { .. } => Err(unsupported("ternary operator (typed lane only)")),
        HirExpr::Local { .. } => Err(unsupported("locals/params (needs block-param support)")),
        HirExpr::Unary { .. } => Err(unsupported("unary operators")),
        HirExpr::Binary { op, lhs, rhs, .. } => {
            let l = lower_expr(hir, b, block, lhs)?;
            let r = lower_expr(hir, b, block, rhs)?;
            // Şu an sadece i64 lane: operand tipleri lowering sırasında
            // bilinir (literal kökenli). f64 lane bir sonraki adımda.
            match op {
                HirBinOp::Add => Ok(b.bin(block, BinOp::Add, MirType::I64, l, r)),
                HirBinOp::Sub => Ok(b.bin(block, BinOp::Sub, MirType::I64, l, r)),
                HirBinOp::Mul => Ok(b.bin(block, BinOp::Mul, MirType::I64, l, r)),
                HirBinOp::Div => Ok(b.bin(block, BinOp::Div, MirType::I64, l, r)),
                HirBinOp::Rem => Ok(b.bin(block, BinOp::Rem, MirType::I64, l, r)),
                HirBinOp::Eq => Ok(b.cmp(block, CmpOp::Eq, MirType::I64, l, r)),
                HirBinOp::Ne => Ok(b.cmp(block, CmpOp::Ne, MirType::I64, l, r)),
                HirBinOp::Lt => Ok(b.cmp(block, CmpOp::Lt, MirType::I64, l, r)),
                HirBinOp::Le => Ok(b.cmp(block, CmpOp::Le, MirType::I64, l, r)),
                HirBinOp::Gt => Ok(b.cmp(block, CmpOp::Gt, MirType::I64, l, r)),
                HirBinOp::Ge => Ok(b.cmp(block, CmpOp::Ge, MirType::I64, l, r)),
                HirBinOp::And | HirBinOp::Or => unreachable!(),
            }
        }
        HirExpr::Call { callee, args, .. } => {
            if callee != "print" {
                return Err(unsupported("non-print calls (CallStatic arrives with M2)"));
            }
            if args.len() != 1 {
                return Err(unsupported("print takes exactly one argument"));
            }
            let v = lower_expr(hir, b, block, &args[0])?;
            let out = b.call_native(block, MirType::Unit, RuntimeHelperId::Print, vec![v]);
            b.gc_safepoint(block);
            // print'in kendisi değer üretmez; ifade değeri olarak null döneriz
            // (çağıran Return(None)/Expr zinciri için).
            let _ = out;
            Ok(b.const_null(block))
        }
    }
}

pub(crate) fn machine_return_ty(ty: &hudhudscript_types::Type) -> Result<MirType, LowerError> {
    use hudhudscript_types::Type;
    match ty {
        Type::Number => Ok(MirType::I64), // AŞAMA-0: sayı dönüşler i64 lane
        Type::Null | Type::Any => Ok(MirType::Unit),
        Type::Boolean => Ok(MirType::Bool),
        other => Err(LowerError::Unsupported {
            function: "<signature>".into(),
            reason: format!("return type {other:?}"),
        }),
    }
}

// TrapKind bu aşamada lowering'de kullanılmıyor ama ön uç sözleşmede
// yerini koruyor (overflow lane M2'de bağlanır).
#[allow(dead_code)]
fn _overflow_trap_kind() -> TrapKind {
    TrapKind::IntegerOverflow
}

#[cfg(test)]
mod tests {
    use super::*;
    use hudhudscript_types::{HirBinOp, HirExpr, HirFunction, HirParam, HirStmt};
    use hudhudscript_types::Type;

    fn main_print_2_3() -> HirFunction {
        HirFunction {
            name: "main".into(),
            params: vec![],
            return_type: Type::Null,
            body: vec![HirStmt::Expr(HirExpr::Call {
                callee: "print".into(),
                args: vec![HirExpr::Binary {
                    op: HirBinOp::Add,
                    lhs: Box::new(HirExpr::IntLit(2)),
                    rhs: Box::new(HirExpr::IntLit(3)),
                    ty: Type::Number,
                }],
                ty: Type::Null,
            })],
        }
    }

    #[test]
    fn lowers_print_2_plus_3() {
        let mir = lower_function(&main_print_2_3()).expect("must lower");
        let text = crate::print::render_function(&mir);
        assert!(text.contains("v0 = const.i64 2"), "{text}");
        assert!(text.contains("v1 = const.i64 3"), "{text}");
        assert!(text.contains("v2 = i64.add v0, v1"), "{text}");
        assert!(text.contains("native hudhud_print(v2) -> unit"), "{text}");
        assert!(text.contains("gc.safepoint"), "{text}");
    }

    #[test]
    fn rejects_unsupported_cleanly() {
        let hir = HirFunction {
            name: "f".into(),
            params: vec![HirParam { name: "a".into(), ty: Type::Number }],
            return_type: Type::Number,
            body: vec![HirStmt::Return(Some(HirExpr::Local {
                name: "a".into(),
                ty: Type::Number,
            }))],
        };
        match lower_function(&hir) {
            Err(LowerError::Unsupported { function, reason }) => {
                assert_eq!(function, "f");
                assert!(reason.contains("locals"), "{reason}");
            }
            Ok(_) => panic!("locals must be rejected in AŞAMA-0"),
        }
    }
}
