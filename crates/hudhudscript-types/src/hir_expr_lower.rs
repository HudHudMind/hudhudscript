//! Expression lowering for typed HIR (AST Expr → HirExpr).

use hudhudscript_ast::{Expr, UnaryOp};

use crate::hir::{HirBinOp, HirExpr, HirUnOp};
use crate::hir_lower::{reject, HirLowerError};
use crate::hir_ops::{bin_op, lower_literal};
use crate::types::Type;

pub(crate) fn lower_expr(expr: &Expr) -> Result<HirExpr, HirLowerError> {
    match expr {
        Expr::Literal(lit, _) => lower_literal(lit),
        Expr::Identifier(name, _) => Ok(HirExpr::Local { name: name.clone(), ty: Type::Any }),
        Expr::Binary { left, op, right, .. } => {
            let l = lower_expr(left)?;
            let r = lower_expr(right)?;
            let hir_op = bin_op(*op)?;
            let ty = if matches!(hir_op, HirBinOp::Add | HirBinOp::Sub | HirBinOp::Mul | HirBinOp::Div | HirBinOp::Rem)
                && l.ty() == Type::Number
                && r.ty() == Type::Number
            {
                Type::Number
            } else if matches!(hir_op, HirBinOp::Eq | HirBinOp::Ne | HirBinOp::Lt | HirBinOp::Le | HirBinOp::Gt | HirBinOp::Ge) {
                Type::Boolean
            } else {
                Type::Any
            };
            Ok(HirExpr::Binary { op: hir_op, lhs: Box::new(l), rhs: Box::new(r), ty })
        }
        Expr::Unary { op, expr: inner, .. } => {
            let operand = lower_expr(inner)?;
            let ty = operand.ty();
            let hir_op = match op {
                UnaryOp::Neg => HirUnOp::Neg,
                UnaryOp::Not => HirUnOp::Not,
                other => return Err(reject("unary op", &format!("{other:?}"))),
            };
            Ok(HirExpr::Unary { op: hir_op, operand: Box::new(operand), ty })
        }
        Expr::Call { callee, args, .. } => {
            let lowered_args = args.iter().map(lower_expr).collect::<Result<Vec<_>, _>>()?;
            match callee.as_ref() {
                Expr::Identifier(name, _) => {
                    if !crate::hir_closure::is_known_function(name) {
                        if let Some(dispatch) = crate::hir_closure::dispatch_closure_call(name, lowered_args.clone()) {
                            return Ok(dispatch);
                        }
                    }
                    Ok(HirExpr::Call { callee: name.clone(), args: lowered_args, ty: Type::Any })
                }
                Expr::Member { object, property, .. } => {
                    if let Expr::Identifier(ns, _) = object.as_ref() {
                        match (ns.as_str(), property.as_str()) {
                            ("Date", "to_millis" | "now" | "millis") => {
                                return Ok(HirExpr::Call { callee: "Date.to_millis".into(), args: lowered_args, ty: Type::Number });
                            }
                            ("Array", "fill" | "filled") => {
                                return Ok(HirExpr::Call { callee: "Array.fill".into(), args: lowered_args, ty: Type::Any });
                            }
                            ("Math", m @ ("sin" | "sqrt" | "cos" | "floor" | "abs" | "pow" | "min" | "max")) => {
                                return Ok(HirExpr::Call { callee: format!("Math.{m}").into(), args: lowered_args, ty: Type::Any });
                            }
                            _ => {}
                        }
                    }
                    let arr = lower_expr(object)?;
                    if matches!(
                        property.as_str(),
                        "push" | "pop" | "length" | "len" | "join" | "substring" | "split" | "indexOf" | "fill"
                    ) {
                        let ty = match property.as_str() {
                            "length" | "len" | "indexOf" => Type::Number,
                            _ => Type::Any,
                        };
                        Ok(HirExpr::ArrayMethod { array: Box::new(arr), method: property.clone(), args: lowered_args, ty })
                    } else if let Some(implementors) = crate::hir_class::get_method_implementors(property) {
                        Ok(crate::hir_class::dispatch_method_call(arr, property, lowered_args, &implementors))
                    } else {
                        return Err(reject("array method", &format!(".{property} not supported on arrays in this lane")));
                    }
                }
                other => return Err(reject("call", &format!("non-identifier callee {:?}", std::mem::discriminant(other)))),
            }
        }
        Expr::Array { elements, .. } => {
            let lowered = elements.iter().map(lower_expr).collect::<Result<Vec<_>, _>>()?;
            Ok(HirExpr::ArrayLit { elements: lowered, ty: Type::Any })
        }
        Expr::Index { object, index, .. } => {
            let arr = lower_expr(object)?;
            let idx = lower_expr(index)?;
            Ok(HirExpr::ArrayIndex { array: Box::new(arr), index: Box::new(idx), ty: Type::Number })
        }
        Expr::Member { object, property, .. } => {
            let obj = lower_expr(object)?;
            Ok(HirExpr::PropertyGet { object: Box::new(obj), name: property.clone(), ty: Type::Any })
        }
        Expr::Object { properties, .. } => {
            let lowered = properties.iter()
                .map(|(k, v)| lower_expr(v).map(|v| (k.clone(), v)))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(HirExpr::ObjectLit { properties: lowered, ty: Type::Any })
        }
        Expr::This(_) => Ok(HirExpr::Local { name: "self".to_string(), ty: Type::Any }),
        Expr::New { class_name, .. } => {
            let type_id = crate::hir_class::get_class_type_id(class_name).unwrap_or(0);
            let properties = vec![
                ("__type_id".to_string(), HirExpr::IntLit(type_id)),
                ("__type".to_string(), HirExpr::StringLit(class_name.clone())),
            ];
            Ok(HirExpr::ObjectLit { properties, ty: Type::Any })
        }
        Expr::Spawn { subject_name, .. } => {
            let type_id = crate::hir_class::get_class_type_id(subject_name).unwrap_or(0);
            let properties = vec![
                ("__type_id".to_string(), HirExpr::IntLit(type_id)),
                ("__type".to_string(), HirExpr::StringLit(subject_name.clone())),
            ];
            Ok(HirExpr::ObjectLit { properties, ty: Type::Any })
        }
        Expr::ArrowFunction { params, body, span, .. } => {
            let (obj, _) = crate::hir_closure::lift_closure(params, body, *span)?;
            Ok(obj)
        }
        Expr::Ternary { condition, true_expr, false_expr, .. } => {
            let c = lower_expr(condition)?;
            let t = lower_expr(true_expr)?;
            let f = lower_expr(false_expr)?;
            let ty = if t.ty() == f.ty() { t.ty() } else { Type::Any };
            Ok(HirExpr::Ternary {
                condition: Box::new(c),
                true_expr: Box::new(t),
                false_expr: Box::new(f),
                ty,
            })
        }
        other => Err(crate::hir_ops::unsupported_expr(other)),
    }
}
