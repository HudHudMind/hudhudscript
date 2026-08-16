use super::*;

impl TypeChecker {
    pub fn check_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::Literal(lit, _) => Ok(self.literal_type(lit)),

            Expr::Identifier(name, span) => self
                .symbol_table
                .lookup(name)
                .cloned()
                .ok_or_else(|| type_codes::undefined_variable(name.clone(), *span)),

            Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;
                self.check_binary_op(*op, &left_type, &right_type, *span)
            }

            Expr::Unary { op, expr, span } => {
                let expr_type = self.check_expr(expr)?;
                self.check_unary_op(*op, &expr_type, *span)
            }

            Expr::Call { callee, args, span } => {
                // Issue #443: detect atomically() calls and track context
                let is_atomically_call = matches!(
                    callee.as_ref(),
                    Expr::Identifier(name, _) if name == "atomically"
                );

                let callee_type = self.check_expr(callee)?;

                if is_atomically_call {
                    // Check args inside atomically context
                    let prev = self.in_atomically_block;
                    self.in_atomically_block = true;
                    for arg in args {
                        self.check_expr(arg)?;
                    }
                    self.in_atomically_block = prev;
                    return Ok(Type::Any);
                }

                match callee_type {
                    Type::Function {
                        params,
                        return_type,
                    } => {
                        if args.len() != params.len() {
                            return Err(type_codes::wrong_argument_count(
                                params.len(),
                                args.len(),
                                *span,
                            ));
                        }

                        for (arg, param_type) in args.iter().zip(params.iter()) {
                            let arg_type = self.check_expr(arg)?;
                            if !arg_type.is_compatible_with(param_type) {
                                return Err(type_codes::mismatch(
                                    format!("{}", param_type),
                                    format!("{}", arg_type),
                                    arg.span(),
                                ));
                            }
                        }

                        Ok(*return_type)
                    }
                    Type::Any => Ok(Type::Any),
                    _ => Err(type_codes::invalid_operator(
                        "call".to_string(),
                        format!("{}", callee_type),
                        *span,
                    )),
                }
            }

            Expr::Array { elements, .. } => {
                if elements.is_empty() {
                    return Ok(Type::Array(Box::new(Type::Any)));
                }

                let first_type = self.check_expr(&elements[0])?;
                for elem in &elements[1..] {
                    let elem_type = self.check_expr(elem)?;
                    if !elem_type.is_compatible_with(&first_type) {
                        return Ok(Type::Array(Box::new(Type::Any)));
                    }
                }

                Ok(Type::Array(Box::new(first_type)))
            }

            Expr::Index {
                object,
                index,
                span,
            } => {
                let obj_type = self.check_expr(object)?;
                let idx_type = self.check_expr(index)?;

                match obj_type {
                    Type::Array(elem_type) => {
                        if !idx_type.is_compatible_with(&Type::Number) {
                            return Err(type_codes::mismatch(
                                "Number".to_string(),
                                format!("{}", idx_type),
                                *span,
                            ));
                        }
                        Ok(*elem_type)
                    }
                    Type::Any => Ok(Type::Any),
                    _ => Err(type_codes::invalid_index(format!("{}", obj_type), *span)),
                }
            }

            Expr::Member {
                object,
                property,
                span,
            } => {
                let obj_type = self.check_expr(object)?;

                match &obj_type {
                    Type::Object(props) => props.get(property).cloned().ok_or_else(|| {
                        type_codes::invalid_member(format!("{}", obj_type), property.clone(), *span)
                    }),
                    Type::Any => Ok(Type::Any),
                    _ => Err(type_codes::invalid_member(
                        format!("{}", obj_type),
                        property.clone(),
                        *span,
                    )),
                }
            }

            // #749: Optional member access — always returns the property type or Null
            Expr::OptionalMember {
                object, span: _, ..
            } => {
                let _ = self.check_expr(object)?;
                Ok(Type::Any) // ?. can return Null or the property type
            }

            Expr::Await { expr, span } => {
                // Issue #443: reject await inside atomically() blocks
                if self.in_atomically_block {
                    return Err(type_codes::await_in_atomically(*span));
                }

                let expr_type = self.check_expr(expr)?;

                match expr_type {
                    Type::Promise(inner) => Ok(*inner),
                    Type::Any => Ok(Type::Any),
                    _ => Err(type_codes::invalid_await(format!("{}", expr_type), *span)),
                }
            }

            Expr::Object { properties, .. } => {
                let mut prop_types = HashMap::new();
                for (name, expr) in properties {
                    let ty = self.check_expr(expr)?;
                    prop_types.insert(name.clone(), ty);
                }
                Ok(Type::Object(prop_types))
            }

            Expr::TemplateString { .. } => {
                // Template strings always evaluate to String
                Ok(Type::String)
            }

            Expr::ArrowFunction { params, body, .. } => {
                // Arrow functions are Function type
                // Infer parameter types and return type
                let param_types: Vec<Type> = params.iter().map(|_| Type::Any).collect();

                let return_type = match body {
                    ArrowFunctionBody::Expression(expr) => {
                        // For expression body, infer the type of the expression
                        self.check_expr(expr).unwrap_or(Type::Any)
                    }
                    ArrowFunctionBody::Block(_) => {
                        // For block body, we'd need to analyze return statements
                        // Currently, use Any
                        Type::Any
                    }
                };

                Ok(Type::Function {
                    params: param_types,
                    return_type: Box::new(return_type),
                })
            }

            Expr::New {
                class_name,
                args,
                span,
            } => {
                // Check that the class exists in scope
                if let Some(class_type) = self.symbol_table.lookup(class_name) {
                    match class_type {
                        Type::Class { .. } | Type::Any => {}
                        _ => {
                            return Err(type_codes::mismatch(
                                "class".to_string(),
                                format!("{}", class_type),
                                *span,
                            ));
                        }
                    }
                }
                // Check argument expressions
                for arg in args {
                    let _ = self.check_expr(arg)?;
                }
                // Return Instance type
                Ok(Type::Instance(class_name.clone()))
            }

            Expr::Spread { expr, .. } => self.check_expr(expr),

            Expr::Yield { value, .. } => {
                if let Some(expr) = value {
                    self.check_expr(expr)?;
                }
                Ok(Type::Any)
            }

            Expr::Spawn { args, .. } => {
                for arg in args {
                    self.check_expr(arg)?;
                }
                Ok(Type::Any)
            }

            Expr::ViewAs { instance, .. } => {
                self.check_expr(instance)?;
                Ok(Type::Any)
            }

            Expr::This(_) => {
                // 'this' refers to the current instance (Issue #689)
                if let Some(ref class_name) = self.current_class {
                    Ok(Type::Instance(class_name.clone()))
                } else {
                    Ok(Type::Any)
                }
            }
            Expr::Perform { action, .. } => {
                self.check_expr(action)?;
                Ok(Type::Any)
            }
            Expr::Ternary {
                true_expr,
                false_expr,
                ..
            } => {
                let t = self.check_expr(true_expr)?;
                let f = self.check_expr(false_expr)?;
                if t == f {
                    Ok(t)
                } else {
                    Ok(Type::Any)
                }
            }
        }
    }

    pub(super) fn literal_type(&self, lit: &Literal) -> Type {
        match lit {
            Literal::String(_) => Type::String,
            Literal::Number(_, _) => Type::Number,
            Literal::Int(_) => Type::Number,
            Literal::BigInt(_) => Type::Number,
            Literal::Boolean(_) => Type::Boolean,
            Literal::Null => Type::Null,
        }
    }
}
