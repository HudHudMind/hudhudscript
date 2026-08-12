use super::*;
use crate::parser::converters::arabic_to_ascii;
use crate::parser::expression::literals::number_literal_from_ascii;

/// Parse Turkish herbir loop: herbir (liste içinde eleman) { }
/// Reversed order compared to for-in: iterable comes first, then variable
pub fn parse_herbir_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    // iterable expression (e.g. ["elma", "muz"] or meyveler)
    let iterable = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected iterable", span))?,
    )?;

    // loop variable identifier (e.g. meyve)
    let variable = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected loop variable", span))?
        .as_str()
        .to_string();

    let body = parse_block(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected body", span))?,
    )?;

    Ok(Stmt::For {
        variable,
        iterable,
        body: Box::new(body),
        span,
    })
}

pub fn parse_switch_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    // Parse switch value
    let value = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected switch value", span))?,
    )?;

    let mut cases = Vec::new();
    let mut default = None;

    // Parse cases and default
    for case_pair in inner {
        match case_pair.as_rule() {
            Rule::case_clause => {
                let case_span = pair_to_span(&case_pair);
                let mut case_inner = case_pair.into_inner();

                let case_value = parse_expression(case_inner.next().ok_or_else(|| {
                    parse_codes::invalid_syntax("Expected case value", case_span)
                })?)?;

                let mut body = Vec::new();
                for stmt_pair in case_inner {
                    // stmt_pair is a case_body_stmt, which wraps a statement
                    let actual_stmt = if stmt_pair.as_rule() == Rule::case_body_stmt {
                        stmt_pair.into_inner().next()
                    } else {
                        Some(stmt_pair)
                    };
                    if let Some(s) = actual_stmt {
                        if let Some(stmt) = parse_statement(s)? {
                            body.push(stmt);
                        }
                    }
                }

                cases.push(SwitchCase {
                    value: case_value,
                    body,
                    span: case_span,
                });
            }
            Rule::default_clause => {
                let default_inner = case_pair.into_inner();
                let mut body = Vec::new();
                for stmt_pair in default_inner {
                    // stmt_pair is a case_body_stmt, which wraps a statement
                    let actual_stmt = if stmt_pair.as_rule() == Rule::case_body_stmt {
                        stmt_pair.into_inner().next()
                    } else {
                        Some(stmt_pair)
                    };
                    if let Some(s) = actual_stmt {
                        if let Some(stmt) = parse_statement(s)? {
                            body.push(stmt);
                        }
                    }
                }
                default = Some(body);
            }
            _ => {}
        }
    }

    Ok(Stmt::Switch {
        value,
        cases,
        default,
        span,
    })
}

pub fn parse_try_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    // Parse try block
    let try_block =
        Box::new(parse_block(inner.next().ok_or_else(|| {
            parse_codes::invalid_syntax("Expected try block", span)
        })?)?);

    let mut catch_clause = None;
    let mut finally_block = None;

    // Parse catch and finally
    for clause_pair in inner {
        match clause_pair.as_rule() {
            Rule::catch_clause => {
                let catch_span = pair_to_span(&clause_pair);
                let mut catch_inner = clause_pair.into_inner();

                let param = catch_inner
                    .next()
                    .ok_or_else(|| {
                        parse_codes::invalid_syntax("Expected catch parameter", catch_span)
                    })?
                    .as_str()
                    .to_string();

                let body = Box::new(parse_block(catch_inner.next().ok_or_else(|| {
                    parse_codes::invalid_syntax("Expected catch block", catch_span)
                })?)?);

                catch_clause = Some(CatchClause {
                    param,
                    body,
                    span: catch_span,
                });
            }
            Rule::finally_clause => {
                let mut finally_inner = clause_pair.into_inner();
                finally_block = Some(Box::new(parse_block(finally_inner.next().ok_or_else(
                    || parse_codes::invalid_syntax("Expected finally block", span),
                )?)?));
            }
            _ => {}
        }
    }

    Ok(Stmt::Try {
        try_block,
        catch_clause,
        finally_block,
        span,
    })
}

pub fn parse_throw_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let value = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected throw value", span))?,
    )?;

    Ok(Stmt::Throw { value, span })
}

pub fn parse_match_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let value = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected match value", span))?,
    )?;

    let mut arms = Vec::new();
    for arm_pair in inner {
        if arm_pair.as_rule() == Rule::match_arm {
            let arm_span = pair_to_span(&arm_pair);
            let mut arm_inner = arm_pair.into_inner();

            let pattern_or_pair = arm_inner
                .next()
                .ok_or_else(|| parse_codes::invalid_syntax("Expected match pattern", arm_span))?;
            let pattern = parse_match_pattern_or(pattern_or_pair)?;

            // Check for optional guard expression (if expr) and body
            let mut guard: Option<Expr> = None;
            let mut body = Vec::new();
            for next_pair in arm_inner {
                match next_pair.as_rule() {
                    Rule::expression => {
                        // If we haven't seen the body yet and guard is None,
                        // this could be the guard or the body expression.
                        // Guard comes first (before =>), body comes after.
                        if guard.is_none() && body.is_empty() {
                            // Peek: if we already have content in body, this is body.
                            // Otherwise, first expression could be guard or body.
                            // The grammar ensures guard expression comes before =>,
                            // and body expression comes after =>.
                            // Since pest strips =>, we get expressions in order:
                            // [guard_expr?, body_expr?]
                            // We need to check if there's another expression after this one.
                            // Actually, pest delivers them in order. Let's collect and decide.
                            // Simpler: set as guard tentatively, will be overwritten if another expr follows.
                            guard = Some(parse_expression(next_pair)?);
                        } else if guard.is_some() && body.is_empty() {
                            // Second expression = body
                            let expr = parse_expression(next_pair)?;
                            body.push(Stmt::Expr(expr));
                        } else {
                            let expr = parse_expression(next_pair)?;
                            body.push(Stmt::Expr(expr));
                        }
                    }
                    Rule::block => {
                        if let Stmt::Block { statements, .. } = parse_block(next_pair)? {
                            body = statements;
                        }
                    }
                    _ => {}
                }
            }

            // If we only got one expression and no block, it's the body, not guard
            if body.is_empty() {
                if let Some(g) = guard.take() {
                    body.push(Stmt::Expr(g));
                }
            }

            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span: arm_span,
            });
        }
    }

    Ok(Stmt::Match { value, arms, span })
}

pub(super) fn parse_match_pattern_or(pair: Pair<Rule>) -> ParseResult<MatchPattern> {
    let span = pair_to_span(&pair);
    let patterns: Vec<Pair<Rule>> = pair.into_inner().collect();
    if patterns.len() == 1 {
        // Single pattern, no OR
        parse_match_pattern(patterns.into_iter().next().unwrap())
    } else if patterns.is_empty() {
        Err(parse_codes::invalid_syntax(
            "Expected at least one pattern",
            span,
        ))
    } else {
        // Multiple patterns joined by |
        let mut sub_patterns = Vec::new();
        for p in patterns {
            sub_patterns.push(parse_match_pattern(p)?);
        }
        Ok(MatchPattern::Or(sub_patterns))
    }
}

pub(super) fn parse_match_pattern(pair: Pair<Rule>) -> ParseResult<MatchPattern> {
    let span = pair_to_span(&pair);
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected pattern", span))?;

    Ok(match inner.as_rule() {
        Rule::wildcard_pattern => MatchPattern::Wildcard,
        Rule::identifier_pattern => MatchPattern::Identifier(inner.as_str().to_string()),
        Rule::literal_pattern => {
            // literal_pattern = { number | string | boolean | null }
            let lit_pair = inner
                .into_inner()
                .next()
                .ok_or_else(|| parse_codes::invalid_syntax("Expected literal", span))?;
            let lit = match lit_pair.as_rule() {
                Rule::number => {
                    let ascii = arabic_to_ascii(lit_pair.as_str());
                    number_literal_from_ascii(&ascii)
                }
                Rule::string => hudhudscript_ast::Literal::String(
                    lit_pair
                        .as_str()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                ),
                Rule::boolean => hudhudscript_ast::Literal::Boolean(lit_pair.as_str() == "true"),
                Rule::null => hudhudscript_ast::Literal::Null,
                _ => hudhudscript_ast::Literal::Null,
            };
            MatchPattern::Literal(lit)
        }
        Rule::enum_variant_pattern => {
            let mut ev_inner = inner.into_inner();
            let enum_name = ev_inner
                .next()
                .map(|p| p.as_str().to_string())
                .unwrap_or_default();
            let variant = ev_inner
                .next()
                .map(|p| p.as_str().to_string())
                .unwrap_or_default();
            let binding = ev_inner.next().map(|p| p.as_str().to_string());
            MatchPattern::EnumVariant {
                enum_name,
                variant,
                binding,
            }
        }
        _ => MatchPattern::Wildcard,
    })
}

pub fn parse_enum_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected enum name", span))?
        .as_str()
        .to_string();

    let mut variants = Vec::new();
    for variant_pair in inner {
        if variant_pair.as_rule() == Rule::enum_variant_decl {
            let variant_span = pair_to_span(&variant_pair);
            let mut v_inner = variant_pair.into_inner();
            let variant_name = v_inner
                .next()
                .ok_or_else(|| parse_codes::invalid_syntax("Expected variant name", variant_span))?
                .as_str()
                .to_string();
            let fields: Vec<String> = v_inner.map(|p| p.as_str().to_string()).collect();
            variants.push(EnumVariant {
                name: variant_name,
                fields,
                span: variant_span,
            });
        }
    }

    Ok(Stmt::EnumDecl {
        name,
        variants,
        span,
    })
}
