use super::*;

/// Parse a forget statement: forget "id" from my_store;
pub(super) fn parse_forget_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let target =
        Box::new(parse_expression(inner.next().ok_or_else(|| {
            parse_codes::invalid_syntax("Expected expression", span)
        })?)?);

    let store_name = inner.next().map(|p| p.as_str().to_string());

    Ok(Stmt::Forget {
        target,
        store_name,
        span,
    })
}

// ============================================================================
// Issue #668: Destructuring
// ============================================================================

/// Parse a destructuring statement: let/var/const { a, b } = expr or [a, b] = expr
pub(super) fn parse_destructure_stmt(pair: Pair<Rule>, is_const: bool) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let pattern_pair = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected destructuring pattern", span))?;

    let pattern = parse_destruct_pattern(pattern_pair)?;

    let value = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected expression", span))?,
    )?;

    Ok(Stmt::Destructure {
        pattern,
        value,
        is_const,
        span,
    })
}

/// Parse a destructuring pattern (array or object)
pub(crate) fn parse_destruct_pattern(pair: Pair<Rule>) -> ParseResult<Pattern> {
    match pair.as_rule() {
        Rule::destruct_array_pattern => {
            let mut elements = Vec::new();
            let mut rest = None;
            for item in pair.into_inner() {
                match item.as_rule() {
                    Rule::destruct_element => {
                        let child = item.into_inner().next().unwrap();
                        elements.push(parse_destruct_pattern(child)?);
                    }
                    Rule::rest_param => {
                        let name = item.into_inner().next().unwrap().as_str().to_string();
                        rest = Some(Box::new(Pattern::Identifier(name)));
                    }
                    Rule::identifier => {
                        elements.push(Pattern::Identifier(item.as_str().to_string()));
                    }
                    _ => {}
                }
            }
            Ok(Pattern::Array { elements, rest })
        }
        Rule::destruct_object_pattern => {
            let mut properties = Vec::new();
            let mut rest = None;
            for item in pair.into_inner() {
                match item.as_rule() {
                    Rule::destruct_prop => {
                        let mut prop_inner = item.into_inner();
                        let key = prop_inner.next().unwrap().as_str().to_string();
                        if let Some(next) = prop_inner.next() {
                            match next.as_rule() {
                                // key: pattern (nested destructuring)
                                Rule::destruct_array_pattern | Rule::destruct_object_pattern => {
                                    let nested = parse_destruct_pattern(next)?;
                                    properties.push((key, nested));
                                }
                                Rule::identifier => {
                                    // key: alias
                                    let alias = next.as_str().to_string();
                                    properties.push((key, Pattern::Identifier(alias)));
                                }
                                // key = default_value
                                _ => {
                                    let default_expr = parse_expression(next)?;
                                    properties.push((
                                        key.clone(),
                                        Pattern::IdentifierDefault(key, default_expr),
                                    ));
                                }
                            }
                        } else {
                            // Just identifier: shorthand { name } → ("name", Identifier("name"))
                            properties.push((key.clone(), Pattern::Identifier(key)));
                        }
                    }
                    Rule::rest_param => {
                        let name = item.into_inner().next().unwrap().as_str().to_string();
                        rest = Some(Box::new(Pattern::Identifier(name)));
                    }
                    _ => {}
                }
            }
            Ok(Pattern::Object { properties, rest })
        }
        Rule::identifier => Ok(Pattern::Identifier(pair.as_str().to_string())),
        _ => Ok(Pattern::Identifier("_".to_string())),
    }
}

// ============================================================================
// Issue #667: Generator functions and yield
// ============================================================================

/// Parse a generator function declaration: function* name(params) { body }
pub(super) fn parse_generator_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected function name", span))?
        .as_str()
        .to_string();

    let mut params = Vec::new();
    let mut destruct_stmts = Vec::new();
    let mut destr_index = 0usize;
    let mut body_statements = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::identifier => {
                params.push(item.as_str().to_string());
            }
            Rule::func_param => {
                let (pname, pattern) = parse_func_param_with_pattern(item, &mut destr_index)?;
                if let Some(pat) = pattern {
                    destruct_stmts.push(Stmt::Destructure {
                        pattern: pat,
                        value: Expr::Identifier(pname.clone(), span),
                        is_const: false,
                        span,
                    });
                }
                params.push(pname);
            }
            Rule::rest_param => {
                let pname = item.into_inner().next().unwrap().as_str();
                params.push(format!("...{}", pname));
            }
            Rule::block => {
                if let Stmt::Block { statements, .. } = parse_block(item)? {
                    body_statements = statements;
                }
            }
            _ => {}
        }
    }

    // Prepend destructuring statements to the function body
    if !destruct_stmts.is_empty() {
        destruct_stmts.append(&mut body_statements);
        body_statements = destruct_stmts;
    }

    Ok(Stmt::Function {
        name,
        params,
        body: body_statements,
        is_async: false,
        is_generator: true,
        type_params: Vec::new(),
        span,
    })
}

/// Parse a yield statement: yield expr;
pub(super) fn parse_yield_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let value = if let Some(expr_pair) = inner.next() {
        Some(parse_expression(expr_pair)?)
    } else {
        None
    };

    Ok(Stmt::Expr(Expr::Yield {
        value: value.map(Box::new),
        span,
    }))
}
