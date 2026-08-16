use super::*;

pub(super) fn parse_arrow_function(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    // Check for async marker (non-silent rule for detection)
    let first = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected arrow function parameters", span))?;

    let (is_async, params_pair) = if first.as_rule() == Rule::async_marker {
        let p = inner.next().ok_or_else(|| {
            parse_codes::invalid_syntax("Expected arrow function parameters after async", span)
        })?;
        (true, p)
    } else {
        (false, first)
    };

    // Parse parameters (with destructuring pattern support — Issue #1015)
    let (params, destruct_stmts) = if let Rule::arrow_params = params_pair.as_rule() {
        parse_func_params_with_patterns(params_pair)?
    } else {
        (vec![], vec![])
    };

    // Parse body (expression or block)
    let body_pair = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected arrow function body", span))?;

    let body = match body_pair.as_rule() {
        Rule::block => {
            let block_stmt = parse_block(body_pair)?;
            let mut statements = if let Stmt::Block { statements, .. } = block_stmt {
                statements
            } else {
                return Err(parse_codes::invalid_syntax("Expected block", span));
            };
            if !destruct_stmts.is_empty() {
                let mut combined = destruct_stmts;
                combined.append(&mut statements);
                statements = combined;
            }
            ArrowFunctionBody::Block(statements)
        }
        _ => {
            let expr = parse_expression(body_pair)?;
            if destruct_stmts.is_empty() {
                ArrowFunctionBody::Expression(Box::new(expr))
            } else {
                // Convert expression body to block with destructure + return
                let mut statements = destruct_stmts;
                statements.push(Stmt::Return {
                    value: Some(expr),
                    span,
                });
                ArrowFunctionBody::Block(statements)
            }
        }
    };

    Ok(Expr::ArrowFunction {
        params,
        body,
        is_async,
        span,
    })
}

pub(super) fn parse_anon_function(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let inner = pair.into_inner();

    // Parse parameters (identifiers or rest params), optional async_kw, and block
    let mut params = Vec::new();
    let mut destruct_stmts = Vec::new();
    let mut destr_index = 0usize;
    let mut body_pair = None;
    let mut is_async = false;

    for token in inner {
        match token.as_rule() {
            Rule::async_kw => is_async = true,
            Rule::identifier => params.push(token.as_str().to_string()),
            Rule::func_param => {
                let (pname, pattern) =
                    crate::parser::parse_func_param_with_pattern(token, &mut destr_index)?;
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
                let name = token
                    .into_inner()
                    .next()
                    .ok_or_else(|| {
                        parse_codes::invalid_syntax("Expected rest parameter name", span)
                    })?
                    .as_str();
                params.push(format!("...{}", name));
            }
            Rule::block => {
                body_pair = Some(token);
                break;
            }
            _ => {
                // async_kw is silent so it won't appear — but check text just in case
                let _ = is_async; // suppress warning
            }
        }
    }

    // Check if grammar matched async_kw (it's silent, so we detect via pair text)
    // async detection is handled at grammar level via anon_function rule

    let body_pair =
        body_pair.ok_or_else(|| parse_codes::invalid_syntax("Expected function body", span))?;

    let block_stmt = parse_block(body_pair)?;
    let mut statements = if let Stmt::Block { statements, .. } = block_stmt {
        statements
    } else {
        vec![]
    };

    // Prepend destructuring statements to the function body
    if !destruct_stmts.is_empty() {
        destruct_stmts.append(&mut statements);
        statements = destruct_stmts;
    }

    Ok(Expr::ArrowFunction {
        params,
        body: ArrowFunctionBody::Block(statements),
        is_async,
        span,
    })
}

/// Parse function parameters (supports rest params with `...name`)
pub fn parse_func_params(pair: Pair<Rule>) -> ParseResult<Vec<String>> {
    let (params, _) = parse_func_params_with_patterns(pair)?;
    Ok(params)
}

/// Parse function/arrow parameters with destructuring pattern support — Issue #1015.
///
/// Returns (param_names, destructure_stmts). Destructured params get synthetic names
/// like `__destruct_0`, and the corresponding `Stmt::Destructure` is returned so
/// callers can prepend it to the function body.
pub(super) fn parse_func_params_with_patterns(
    pair: Pair<Rule>,
) -> ParseResult<(Vec<String>, Vec<Stmt>)> {
    let span = pair_to_span(&pair);
    let mut params = Vec::new();
    let mut destruct_stmts = Vec::new();
    let mut destr_index = 0usize;

    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::identifier => params.push(child.as_str().to_string()),
            Rule::func_param => {
                let (pname, pattern) =
                    crate::parser::parse_func_param_with_pattern(child, &mut destr_index)?;
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
            Rule::destruct_object_pattern | Rule::destruct_array_pattern => {
                let pattern = crate::parser::parse_destruct_pattern(child)?;
                let synthetic_name = format!("__destruct_{}", destr_index);
                destr_index += 1;
                destruct_stmts.push(Stmt::Destructure {
                    pattern,
                    value: Expr::Identifier(synthetic_name.clone(), span),
                    is_const: false,
                    span,
                });
                params.push(synthetic_name);
            }
            Rule::rest_param => {
                let name = child
                    .into_inner()
                    .next()
                    .ok_or_else(|| {
                        parse_codes::invalid_syntax("Expected rest parameter name", span)
                    })?
                    .as_str();
                params.push(format!("...{}", name));
            }
            _ => {}
        }
    }
    Ok((params, destruct_stmts))
}

pub(super) fn parse_template_string(pair: Pair<Rule>) -> ParseResult<Expr> {
    use crate::pest_parser::HudHudParser;
    use pest::Parser;

    let span = pair_to_span(&pair);
    let mut parts = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::template_text => {
                let text = inner_pair.as_str().to_string();
                parts.push(TemplateStringPart::Text(text));
            }
            Rule::template_interpolation => {
                // Extract the expression source from ${...} by stripping delimiters
                let raw = inner_pair.as_str();
                // raw is "${expr}" — strip "${" prefix (2 chars) and "}" suffix (1 char)
                let expr_src = if raw.len() >= 3 {
                    &raw[2..raw.len() - 1]
                } else {
                    ""
                };
                if !expr_src.trim().is_empty() {
                    // Re-parse the expression content using the full parser
                    let interp_pairs = HudHudParser::parse(Rule::expression, expr_src.trim())
                        .map_err(|e| {
                            crate::error::parse_codes::invalid_syntax(e.to_string(), span)
                        })?;
                    if let Some(expr_pair) = interp_pairs.into_iter().next() {
                        let expr = parse_expression(expr_pair)?;
                        parts.push(TemplateStringPart::Interpolation(Box::new(expr)));
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Expr::TemplateString { parts, span })
}
