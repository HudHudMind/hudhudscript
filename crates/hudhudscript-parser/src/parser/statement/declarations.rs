use super::*;

pub fn parse_expr_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let expr = parse_expression(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected expression", span))?,
    )?;

    Ok(Stmt::Expr(expr))
}

pub fn parse_function_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    // Parse function name
    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected function name", span))?
        .as_str()
        .to_string();

    // Parse optional generic params, parameters, body
    let mut type_params = Vec::new();
    let mut params = Vec::new();
    let mut destruct_stmts = Vec::new();
    let mut destr_index = 0usize;
    let mut body_statements = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::generic_params => {
                type_params = parse_generic_params(item)?;
            }
            Rule::identifier => {
                params.push(item.as_str().to_string());
            }
            Rule::func_param => {
                let (pname, pattern) = parse_func_param_with_pattern(item, &mut destr_index)?;
                if let Some(pat) = pattern {
                    // Issue #1015: Desugar destructured param into a Stmt::Destructure
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
        is_generator: false,
        type_params,
        span,
    })
}

pub fn parse_async_function_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    // Parse function name
    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected function name", span))?
        .as_str()
        .to_string();

    // Parse optional generic params, parameters, body
    let mut type_params = Vec::new();
    let mut params = Vec::new();
    let mut destruct_stmts = Vec::new();
    let mut destr_index = 0usize;
    let mut body_statements = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::generic_params => {
                type_params = parse_generic_params(item)?;
            }
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
        is_async: true,
        is_generator: false,
        type_params,
        span,
    })
}

/// Parse generic type parameters: <T>, <T: Comparable>, <K, V> — Issue #658
pub(super) fn parse_generic_params(pair: Pair<Rule>) -> ParseResult<Vec<GenericParam>> {
    let mut type_params = Vec::new();
    for item in pair.into_inner() {
        if item.as_rule() == Rule::generic_param {
            let span = pair_to_span(&item);
            let mut inner = item.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let constraint = inner.next().map(|c| c.as_str().to_string());
            type_params.push(GenericParam {
                name,
                constraint,
                span,
            });
        }
    }
    Ok(type_params)
}

/// Parse a trait/interface declaration — Issue #659
pub(super) fn parse_trait_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected trait name", span))?
        .as_str()
        .to_string();

    let mut type_params = Vec::new();
    let mut methods = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::generic_params => {
                type_params = parse_generic_params(item)?;
            }
            Rule::trait_method_sig => {
                methods.push(parse_trait_method_sig(item)?);
            }
            _ => {}
        }
    }

    Ok(Stmt::Trait {
        name,
        type_params,
        methods,
        span,
    })
}

/// Parse a trait method signature — Issue #659
pub(super) fn parse_trait_method_sig(pair: Pair<Rule>) -> ParseResult<TraitMethodSig> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected method name", span))?
        .as_str()
        .to_string();

    let mut params = Vec::new();
    let mut return_type = None;

    for item in inner {
        match item.as_rule() {
            Rule::func_param => {
                let param = parse_func_param(item);
                params.push(param);
            }
            Rule::type_annotation => {
                // Return type annotation after ":"
                return_type = Some(parse_type_annotation(item));
            }
            _ => {}
        }
    }

    Ok(TraitMethodSig {
        name,
        params,
        return_type,
        span,
    })
}

pub fn parse_block(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut statements = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::statement => {
                if let Some(stmt) = parse_statement(inner_pair)? {
                    statements.push(stmt);
                }
            }
            Rule::block_statement => {
                if let Some(stmt) = parse_block_statement(inner_pair)? {
                    statements.push(stmt);
                }
            }
            _ => {}
        }
    }

    Ok(Stmt::Block { statements, span })
}

/// Parse a block or a single statement (used for braceless if/while support)
pub fn parse_block_or_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    match pair.as_rule() {
        Rule::block => parse_block(pair),
        Rule::block_statement => {
            if let Some(stmt) = parse_block_statement(pair)? {
                Ok(stmt)
            } else {
                Err(parse_codes::invalid_syntax("Expected statement", span))
            }
        }
        _ => {
            if let Some(stmt) = parse_statement(pair)? {
                Ok(stmt)
            } else {
                Err(parse_codes::invalid_syntax(
                    "Expected statement or block",
                    span,
                ))
            }
        }
    }
}

/// Parse a statement inside a block (semicolons optional)
pub fn parse_block_statement(pair: Pair<Rule>) -> ParseResult<Option<Stmt>> {
    if let Some(inner_pair) = pair.into_inner().next() {
        return Ok(Some(match inner_pair.as_rule() {
            Rule::destructure_var_stmt => parse_destructure_stmt(inner_pair, false)?,
            Rule::destructure_let_stmt => parse_destructure_stmt(inner_pair, false)?,
            Rule::destructure_const_stmt => parse_destructure_stmt(inner_pair, true)?,
            Rule::var_stmt => parse_var_stmt(inner_pair)?,
            Rule::let_stmt => parse_let_stmt(inner_pair)?,
            Rule::const_stmt => parse_const_stmt(inner_pair)?,
            Rule::assignment_stmt => parse_assignment_stmt(inner_pair)?,
            Rule::return_stmt => parse_return_stmt(inner_pair)?,
            Rule::break_stmt => parse_break_stmt(inner_pair)?,
            Rule::continue_stmt => parse_continue_stmt(inner_pair)?,
            Rule::yield_stmt => parse_yield_stmt(inner_pair)?,
            Rule::if_stmt => parse_if_stmt(inner_pair)?,
            Rule::while_stmt => parse_while_stmt(inner_pair)?,
            Rule::for_stmt => parse_for_stmt(inner_pair)?,
            Rule::for_c_style_stmt => parse_for_c_style_stmt(inner_pair)?,
            Rule::for_range_stmt => parse_for_range_stmt(inner_pair)?,
            Rule::herbir_stmt => parse_herbir_stmt(inner_pair)?,
            Rule::try_stmt => parse_try_stmt(inner_pair)?,
            Rule::throw_stmt => parse_throw_stmt(inner_pair)?,
            Rule::switch_stmt => parse_switch_stmt(inner_pair)?,
            Rule::match_stmt => parse_match_stmt(inner_pair)?,
            Rule::function_decl => parse_function_decl(inner_pair)?,
            Rule::async_function_decl => parse_async_function_decl(inner_pair)?,
            Rule::generator_decl => parse_generator_decl(inner_pair)?,
            Rule::block => parse_block(inner_pair)?,
            Rule::block_expr_stmt => parse_expr_stmt(inner_pair)?,
            _ => return Ok(None),
        }));
    }
    Ok(None)
}

// ============================================================================
// Entity, StateMachine, Event, Contract, Treaty parsers
//
// These grammar rules are defined in base.pest and listed in the top-level
// `statement` rule, but were previously unhandled (silently dropped).
//
// The AST does not yet have dedicated Stmt/Decl variants for these types.
// They are represented as Stmt::Decl(Decl::Agent { .. }) as a pragmatic
// marker that preserves the name and fields.
//
// EventDecl, ContractDecl, TreatyDecl) in a future refactor.
// ============================================================================

/// Parse an entity declaration.
///
/// Syntax: `entity Name { data fieldName: TypeName = expr, ... }`
///
/// Entity fields carry a qualifier (`data` or `agentstate`) followed by
/// `fieldName: TypeName` with an optional default expression.  We store each
/// field as a string key mapped to its default expression (or Null when no
/// default is present) so the information survives into the AST.
pub fn parse_entity_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected entity name", span))?
        .as_str()
        .to_string();

    // entity_field = { (data_kw | agentstate_kw) ~ identifier ~ ":" ~ identifier ~ ("=" ~ expression)? ~ (comma | semicolon)? }
    let mut fields: Vec<(String, Expr)> = Vec::new();
    for field_pair in inner {
        if field_pair.as_rule() == Rule::entity_field {
            let mut fi = field_pair.into_inner();
            // Skip the qualifier keyword (data / agentstate) — it matches _{ } silent rules,
            // so Pest does not emit it as a child pair.  The first visible child is the field name.
            let field_name = match fi.next() {
                Some(p) => p.as_str().to_string(),
                None => continue,
            };
            // Next is the type name (identifier) — store it as a string literal temporarily.
            let type_name = match fi.next() {
                Some(p) => p.as_str().to_string(),
                None => String::new(),
            };
            // Optional default expression.
            let value = if let Some(expr_pair) = fi.next() {
                parse_expression(expr_pair)?
            } else {
                // No default — represent the type as a string literal so it isn't lost.
                Expr::Literal(Literal::String(type_name), span)
            };
            fields.push((field_name, value));
        }
    }

    Ok(Stmt::Decl(Decl::Entity { name, fields, span }))
}
