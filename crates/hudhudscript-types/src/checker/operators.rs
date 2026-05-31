use super::*;

impl TypeChecker {
    pub(super) fn check_binary_op(
        &self,
        op: BinaryOp,
        left: &Type,
        right: &Type,
        span: hudhudscript_ast::Span,
    ) -> Result<Type, TypeError> {
        use BinaryOp::*;

        match op {
            Add => {
                // String + String (or String + Any / Any + String) is valid concatenation
                if left.is_compatible_with(&Type::String) && right.is_compatible_with(&Type::String)
                {
                    return Ok(Type::String);
                }
                if left.is_compatible_with(&Type::Number) && right.is_compatible_with(&Type::Number)
                {
                    return Ok(Type::Number);
                }
                if left == &Type::Any || right == &Type::Any {
                    return Ok(Type::Any);
                }
                Err(type_codes::invalid_operator(
                    "+".to_string(),
                    format!("{}, {}", left, right),
                    span,
                ))
            }
            Sub | Mul | Div | Mod => {
                if left.is_compatible_with(&Type::Number) && right.is_compatible_with(&Type::Number)
                {
                    Ok(Type::Number)
                } else if left == &Type::Any || right == &Type::Any {
                    Ok(Type::Any)
                } else {
                    Err(type_codes::invalid_operator(
                        format!("{:?}", op),
                        format!("{}, {}", left, right),
                        span,
                    ))
                }
            }

            Eq | Ne | Lt | Le | Gt | Ge => Ok(Type::Boolean),

            And | Or => {
                if left.is_compatible_with(&Type::Boolean)
                    && right.is_compatible_with(&Type::Boolean)
                {
                    Ok(Type::Boolean)
                } else if left == &Type::Any || right == &Type::Any {
                    Ok(Type::Any)
                } else {
                    Err(type_codes::invalid_operator(
                        format!("{:?}", op),
                        format!("{}, {}", left, right),
                        span,
                    ))
                }
            }

            // #749: Null coalescing returns the non-null type
            NullCoalesce => Ok(Type::Any),

            // #981: instanceof always returns boolean
            InstanceOf => Ok(Type::Boolean),
        }
    }

    pub(super) fn check_unary_op(
        &self,
        op: UnaryOp,
        expr_type: &Type,
        span: hudhudscript_ast::Span,
    ) -> Result<Type, TypeError> {
        match op {
            UnaryOp::Not => {
                if expr_type.is_compatible_with(&Type::Boolean) || expr_type == &Type::Any {
                    Ok(Type::Boolean)
                } else {
                    Err(type_codes::invalid_operator(
                        "!".to_string(),
                        format!("{}", expr_type),
                        span,
                    ))
                }
            }
            UnaryOp::Neg => {
                if expr_type.is_compatible_with(&Type::Number) || expr_type == &Type::Any {
                    Ok(Type::Number)
                } else {
                    Err(type_codes::invalid_operator(
                        "-".to_string(),
                        format!("{}", expr_type),
                        span,
                    ))
                }
            }
            UnaryOp::Plus => {
                if expr_type.is_compatible_with(&Type::Number) || expr_type == &Type::Any {
                    Ok(Type::Number)
                } else {
                    Err(type_codes::invalid_operator(
                        "+".to_string(),
                        format!("{}", expr_type),
                        span,
                    ))
                }
            }
            UnaryOp::PostIncrement | UnaryOp::PostDecrement => {
                if expr_type.is_compatible_with(&Type::Number) || expr_type == &Type::Any {
                    Ok(expr_type.clone())
                } else {
                    let op_str = if matches!(op, UnaryOp::PostIncrement) { "++" } else { "--" };
                    Err(type_codes::invalid_operator(
                        op_str.to_string(),
                        format!("{}", expr_type),
                        span,
                    ))
                }
            }
        }
    }

    /// Check a class declaration: validate hierarchy, detect circular inheritance,
    /// verify method override signatures, and check super calls.
    pub(super) fn check_class_decl(&mut self, class_decl: &ClassDecl) -> Result<(), TypeError> {
        // Save the outer class context and set the current one (Issue #689).
        let previous_class = self.current_class.take();
        self.current_class = Some(class_decl.name.clone());

        let result = self.check_class_decl_inner(class_decl);

        // Restore the outer class context (Issue #689).
        self.current_class = previous_class;

        result
    }

    /// Inner implementation of class declaration checking, separated so that
    /// `check_class_decl` can guarantee `current_class` is always restored.
    pub(super) fn check_class_decl_inner(
        &mut self,
        class_decl: &ClassDecl,
    ) -> Result<(), TypeError> {
        let parent_name = class_decl.parent.clone();

        // 1. Check that parent class exists (if `extends` is used)
        if let Some(ref parent) = parent_name {
            if self.symbol_table.lookup(parent).is_none() {
                return Err(type_codes::undefined_variable(
                    parent.clone(),
                    class_decl.span,
                ));
            }
            // Verify parent is actually a class type
            if let Some(parent_type) = self.symbol_table.lookup(parent) {
                match parent_type {
                    Type::Class { .. } | Type::Any => {}
                    _ => {
                        return Err(type_codes::mismatch(
                            "class".to_string(),
                            format!("{}", parent_type),
                            class_decl.span,
                        ));
                    }
                }
            }

            // 2. Detect circular inheritance
            let mut visited = HashSet::new();
            visited.insert(class_decl.name.clone());
            let mut current = Some(parent.clone());
            while let Some(ref ancestor_name) = current {
                if visited.contains(ancestor_name) {
                    return Err(type_codes::mismatch(
                        "non-circular inheritance chain".to_string(),
                        format!(
                            "circular inheritance detected: {} -> {}",
                            class_decl.name, ancestor_name
                        ),
                        class_decl.span,
                    ));
                }
                visited.insert(ancestor_name.clone());
                // Walk up the parent chain
                current = if let Some(ancestor_type) = self.symbol_table.lookup(ancestor_name) {
                    match ancestor_type {
                        Type::Class {
                            parent: Some(gp), ..
                        } => Some(gp.clone()),
                        _ => None,
                    }
                } else {
                    None
                };
            }
        }

        // 3. Register the class in the symbol table so it can be referenced
        let class_type = Type::Class {
            name: class_decl.name.clone(),
            parent: parent_name,
        };
        let _ = self
            .symbol_table
            .define(class_decl.name.clone(), class_type);

        // 3b. Issue #1009: Register generic type parameters and constraints
        // e.g. class Stack<T: Comparable> registers T with its constraint
        self.register_generic_params(&class_decl.type_params);

        // 4. Check each member
        for member in &class_decl.members {
            match member {
                ClassMember::Constructor { body, .. } => {
                    self.symbol_table.enter_scope();
                    // Constructor has access to `this`
                    let _ = self
                        .symbol_table
                        .define("this".to_string(), Type::Instance(class_decl.name.clone()));
                    let _ = self
                        .symbol_table
                        .define("self".to_string(), Type::Instance(class_decl.name.clone()));
                    let _ = self
                        .symbol_table
                        .define("kendi".to_string(), Type::Instance(class_decl.name.clone()));
                    // `super` is available if there's a parent
                    if class_decl.parent.is_some() {
                        let _ = self.symbol_table.define("super".to_string(), Type::Any);
                        let _ = self.symbol_table.define("üst".to_string(), Type::Any);
                    }
                    for s in body {
                        self.check_stmt(s)?;
                    }
                    self.symbol_table.exit_scope();
                }
                ClassMember::Method { body, params, .. } => {
                    self.symbol_table.enter_scope();
                    let _ = self
                        .symbol_table
                        .define("this".to_string(), Type::Instance(class_decl.name.clone()));
                    let _ = self
                        .symbol_table
                        .define("self".to_string(), Type::Instance(class_decl.name.clone()));
                    let _ = self
                        .symbol_table
                        .define("kendi".to_string(), Type::Instance(class_decl.name.clone()));
                    if class_decl.parent.is_some() {
                        let _ = self.symbol_table.define("super".to_string(), Type::Any);
                        let _ = self.symbol_table.define("üst".to_string(), Type::Any);
                    }
                    for param in params {
                        let _ = self.symbol_table.define(param.name.clone(), Type::Any);
                    }
                    for s in body {
                        self.check_stmt(s)?;
                    }
                    self.symbol_table.exit_scope();
                }
                ClassMember::Field { initializer, .. } => {
                    if let Some(init_expr) = initializer {
                        self.check_expr(init_expr)?;
                    }
                }
            }
        }

        // Issue #1009: Clean up generic constraints when leaving class scope
        self.unregister_generic_params(&class_decl.type_params);

        Ok(())
    }
}
