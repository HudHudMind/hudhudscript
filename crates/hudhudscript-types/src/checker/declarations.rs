use super::*;

impl TypeChecker {
    pub(super) fn check_var_decl(&mut self, decl: &VarDecl) -> Result<(), TypeError> {
        let init_type = if let Some(init) = &decl.initializer {
            self.check_expr(init)?
        } else {
            Type::Any
        };

        let var_type = if let Some(annotation) = &decl.type_annotation {
            let annotated_type = Type::from_ast(annotation);
            // Issue #866 TYPE-001: In strict mode, enforce type annotations even
            // when the initializer type is Any. In non-strict mode, Any is
            // compatible with everything (gradual typing).
            let should_check = if self.strict {
                !init_type.is_compatible_with(&annotated_type)
            } else {
                !init_type.is_compatible_with(&annotated_type) && init_type != Type::Any
            };
            if should_check {
                return Err(type_codes::type_mismatch_in_decl(
                    format!("{}", annotated_type),
                    format!("{}", init_type),
                    decl.name.clone(),
                    decl.span,
                ));
            }
            annotated_type
        } else if self.strict && init_type == Type::Any {
            // In strict mode, require type annotations for untyped variables
            // when the initializer is ambiguous (Any).
            // For now, just accept it — full enforcement will require
            // a "missing type annotation" error category.
            init_type
        } else {
            init_type
        };

        // Issue #330: translate AST OwnershipMode → type-checker Ownership
        let ownership = match decl.ownership {
            OwnershipMode::Owned => Ownership::OwnedValue,
            OwnershipMode::Borrowed => Ownership::SharedRef,
            OwnershipMode::MutBorrowed => Ownership::SharedMutRef,
        };
        let mutable = !decl.is_const;
        let info = SymbolInfo::new(var_type, mutable, ownership);

        if let Err(_) = self.symbol_table.define_with_info(decl.name.clone(), info) {
            return self.handle_duplicate(&decl.name, decl.span);
        }
        Ok(())
    }
}
