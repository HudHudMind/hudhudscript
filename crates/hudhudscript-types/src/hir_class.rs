//! Class and method dispatch support for typed HIR lowering.

use std::cell::RefCell;
use std::collections::HashMap;

use hudhudscript_ast::{ClassMember, Stmt};

use crate::hir::{HirExpr, HirModule};
use crate::hir_lower::{lower_function, HirLowerError};
use crate::types::Type;

#[derive(Debug, Clone, Default)]
pub struct ClassTable {
    pub classes: HashMap<String, ClassInfo>,
    pub methods: HashMap<String, Vec<(String, i64)>>,
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub type_id: i64,
    pub parent: Option<String>,
    pub methods: HashMap<String, String>,
}

thread_local! {
    static CURRENT_CLASS_TABLE: RefCell<Option<ClassTable>> = const { RefCell::new(None) };
}

pub fn with_class_table<R>(table: ClassTable, f: impl FnOnce() -> R) -> R {
    CURRENT_CLASS_TABLE.with(|cell| {
        *cell.borrow_mut() = Some(table);
    });
    let result = f();
    CURRENT_CLASS_TABLE.with(|cell| {
        *cell.borrow_mut() = None;
    });
    result
}

pub(crate) fn get_class_type_id(name: &str) -> Option<i64> {
    CURRENT_CLASS_TABLE.with(|cell| {
        cell.borrow()
            .as_ref()
            .and_then(|t| t.classes.get(name).map(|c| c.type_id))
    })
}

pub(crate) fn get_method_implementors(method: &str) -> Option<Vec<(String, i64)>> {
    CURRENT_CLASS_TABLE.with(|cell| {
        cell.borrow()
            .as_ref()
            .and_then(|t| t.methods.get(method).cloned())
    })
}

impl ClassTable {
    pub fn from_stmts(stmts: &[Stmt]) -> Self {
        let mut table = ClassTable::default();
        let mut next_type_id = 1i64;

        for stmt in stmts {
            if let Stmt::Class(decl) = stmt {
                let type_id = next_type_id;
                next_type_id += 1;

                let mut method_map = HashMap::new();
                if let Some(parent) = &decl.parent {
                    if let Some(parent_info) = table.classes.get(parent) {
                        method_map = parent_info.methods.clone();
                    }
                }

                for member in &decl.members {
                    if let ClassMember::Method { name, .. } = member {
                        let fn_name = format!("{}_{}", decl.name, name);
                        method_map.insert(name.clone(), fn_name);
                    }
                }

                let info = ClassInfo {
                    name: decl.name.clone(),
                    type_id,
                    parent: decl.parent.clone(),
                    methods: method_map.clone(),
                };

                for (method_name, _) in &method_map {
                    table
                        .methods
                        .entry(method_name.clone())
                        .or_default()
                        .push((decl.name.clone(), type_id));
                }

                table.classes.insert(decl.name.clone(), info);
            } else if let Stmt::Decl(hudhudscript_ast::Decl::Subject { name, ability_defs, .. }) = stmt {
                let type_id = next_type_id;
                next_type_id += 1;
                let mut method_map = HashMap::new();
                for ability in ability_defs {
                    let fn_name = format!("{}_{}", name, ability.name);
                    method_map.insert(ability.name.clone(), fn_name);
                }
                let info = ClassInfo {
                    name: name.clone(),
                    type_id,
                    parent: None,
                    methods: method_map.clone(),
                };
                for (method_name, _) in &method_map {
                    table
                        .methods
                        .entry(method_name.clone())
                        .or_default()
                        .push((name.clone(), type_id));
                }
                table.classes.insert(name.clone(), info);
            }
        }

        table
    }

    pub fn lower_methods(
        &self,
        stmts: &[Stmt],
        module: &mut HirModule,
    ) -> Result<(), HirLowerError> {
        for stmt in stmts {
            if let Stmt::Class(decl) = stmt {
                for member in &decl.members {
                    if let ClassMember::Method { name, params, body, .. } = member {
                        let fn_name = format!("{}_{}", decl.name, name);
                        let mut param_names = vec!["self".to_string()];
                        param_names.extend(params.iter().map(|p| p.name.clone()));
                        let func = lower_function(&fn_name, &param_names, body)?;
                        module.functions.insert(fn_name, func);
                    }
                }
            } else if let Stmt::Decl(hudhudscript_ast::Decl::Subject { name, ability_defs, .. }) = stmt {
                for ability in ability_defs {
                    let fn_name = format!("{}_{}", name, ability.name);
                    let mut param_names = Vec::new();
                    if !ability.params.iter().any(|p| p == "self") {
                        param_names.push("self".to_string());
                    }
                    param_names.extend(ability.params.iter().cloned());
                    let func = lower_function(&fn_name, &param_names, &ability.body)?;
                    module.functions.insert(fn_name, func);
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn dispatch_method_call(
    obj_hir: HirExpr,
    method: &str,
    lowered_args: Vec<HirExpr>,
    implementors: &[(String, i64)],
) -> HirExpr {
    let mut call_args = vec![obj_hir.clone()];
    call_args.extend(lowered_args);

    if implementors.len() == 1 {
        let (cls, _) = &implementors[0];
        return HirExpr::Call {
            callee: format!("{cls}_{method}"),
            args: call_args,
            ty: Type::Any,
        };
    }

    let tid = HirExpr::PropertyGet {
        object: Box::new(obj_hir),
        name: "__type_id".to_string(),
        ty: Type::Number,
    };

    let mut dispatch = HirExpr::Call {
        callee: format!("{}_{method}", implementors[0].0),
        args: call_args.clone(),
        ty: Type::Any,
    };

    for (cls, type_id) in implementors.iter().skip(1) {
        let cond = HirExpr::Binary {
            op: crate::hir::HirBinOp::Eq,
            lhs: Box::new(tid.clone()),
            rhs: Box::new(HirExpr::IntLit(*type_id)),
            ty: Type::Boolean,
        };
        let target_call = HirExpr::Call {
            callee: format!("{cls}_{method}"),
            args: call_args.clone(),
            ty: Type::Any,
        };
        dispatch = HirExpr::Ternary {
            condition: Box::new(cond),
            true_expr: Box::new(target_call),
            false_expr: Box::new(dispatch),
            ty: Type::Any,
        };
    }

    dispatch
}
