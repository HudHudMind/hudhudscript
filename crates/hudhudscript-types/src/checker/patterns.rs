use super::*;

impl TypeChecker {
    pub(super) fn check_match_exhaustiveness(
        &mut self,
        subject_type: &Type,
        arms: &[hudhudscript_ast::MatchArm],
        span: hudhudscript_ast::Span,
    ) {
        let variants = match subject_type {
            Type::Union(types) => types,
            _ => return, // Only check union types
        };

        // If any arm has a wildcard pattern (directly or in an Or), the match is exhaustive.
        if arms
            .iter()
            .any(|arm| Self::pattern_has_wildcard(&arm.pattern))
        {
            return;
        }

        // If any arm has an identifier pattern (catch-all binding), it's exhaustive.
        if arms
            .iter()
            .any(|arm| Self::pattern_is_catch_all(&arm.pattern))
        {
            return;
        }

        // Collect the set of types that are covered by literal/type patterns.
        let mut covered: HashSet<String> = HashSet::new();
        for arm in arms {
            Self::collect_covered_types(&arm.pattern, &mut covered);
        }

        // Check which union variants are not covered.
        let missing: Vec<String> = variants
            .iter()
            .filter(|v| !covered.contains(&v.to_string()))
            .map(|v| v.to_string())
            .collect();

        if !missing.is_empty() {
            let warning = type_codes::non_exhaustive_match(
                subject_type.to_string(),
                missing.join(", "),
                span,
            );
            self.warnings.push(warning);
        }
    }

    /// Returns true if a pattern contains a wildcard `_`.
    pub(super) fn pattern_has_wildcard(pattern: &hudhudscript_ast::MatchPattern) -> bool {
        use hudhudscript_ast::MatchPattern;
        match pattern {
            MatchPattern::Wildcard => true,
            MatchPattern::Or(sub) => sub.iter().any(Self::pattern_has_wildcard),
            _ => false,
        }
    }

    /// Returns true if a pattern is a simple identifier binding (acts as catch-all).
    pub(super) fn pattern_is_catch_all(pattern: &hudhudscript_ast::MatchPattern) -> bool {
        use hudhudscript_ast::MatchPattern;
        match pattern {
            MatchPattern::Identifier(_) => true,
            MatchPattern::Or(sub) => sub.iter().any(Self::pattern_is_catch_all),
            _ => false,
        }
    }

    /// Collect the type names covered by a pattern into the given set.
    pub(super) fn collect_covered_types(
        pattern: &hudhudscript_ast::MatchPattern,
        covered: &mut HashSet<String>,
    ) {
        use hudhudscript_ast::MatchPattern;
        match pattern {
            MatchPattern::Literal(lit) => {
                // Map literal kinds to the type they cover
                let type_name = match lit {
                    Literal::String(_) => "String",
                    Literal::Number(_, _) => "Number",
                    Literal::Boolean(_) => "Boolean",
                    Literal::Null => "Null",
                };
                covered.insert(type_name.to_string());
            }
            MatchPattern::EnumVariant { enum_name, .. } => {
                covered.insert(enum_name.clone());
            }
            MatchPattern::Or(sub) => {
                for p in sub {
                    Self::collect_covered_types(p, covered);
                }
            }
            MatchPattern::Wildcard | MatchPattern::Identifier(_) => {}
        }
    }

    /// Introduce bindings from a match pattern into the symbol table as `Type::Any`.
    pub(super) fn bind_match_pattern_types(&mut self, pattern: &hudhudscript_ast::MatchPattern) {
        use hudhudscript_ast::MatchPattern;
        match pattern {
            MatchPattern::Identifier(ident) => {
                let _ = self.symbol_table.define(ident.clone(), Type::Any);
            }
            MatchPattern::EnumVariant {
                binding: Some(b), ..
            } => {
                let _ = self.symbol_table.define(b.clone(), Type::Any);
            }
            MatchPattern::Or(sub_patterns) => {
                for sub in sub_patterns {
                    self.bind_match_pattern_types(sub);
                }
            }
            _ => {}
        }
    }
}
