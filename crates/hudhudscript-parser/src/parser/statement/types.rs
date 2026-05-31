use super::*;

/// Parse a state machine declaration.
///
/// Syntax: `statemachine Name { state StateName { on event(EvtName) -> NextState } }`
///
/// Each `state_def` block is unrolled into a flat list of
/// `"StateName.EvtName"` → `"NextState"` string-pair fields so the
/// transition table survives into the AST without a dedicated variant.
pub fn parse_statemachine_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected state machine name", span))?
        .as_str()
        .to_string();

    // state_def = { state_kw ~ identifier ~ "{" ~ state_transition* ~ "}" }
    // state_transition = { on_kw ~ event_kw ~ "(" ~ identifier ~ ")" ~ "->" ~ identifier }
    let mut fields: Vec<(String, Expr)> = Vec::new();
    for state_pair in inner {
        if state_pair.as_rule() == Rule::state_def {
            let mut si = state_pair.into_inner();
            let state_name = match si.next() {
                Some(p) => p.as_str().to_string(),
                None => continue,
            };
            for trans_pair in si {
                if trans_pair.as_rule() == Rule::state_transition {
                    let mut ti = trans_pair.into_inner();
                    let event_name = match ti.next() {
                        Some(p) => p.as_str().to_string(),
                        None => continue,
                    };
                    let next_state = match ti.next() {
                        Some(p) => p.as_str().to_string(),
                        None => continue,
                    };
                    // Key: "StateName.eventName", Value: "NextStateName"
                    let key = format!("{}.{}", state_name, event_name);
                    fields.push((key, Expr::Literal(Literal::String(next_state), span)));
                }
            }
        }
    }

    Ok(Stmt::Decl(Decl::StateMachine { name, fields, span }))
}

/// Parse an event declaration.
///
/// Syntax: `event Name { fieldName: TypeName, ... }`
///
/// Event fields are simple `identifier : identifier` pairs (no expression
/// value).  We store the type name as a string literal so it is preserved.
pub fn parse_event_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected event name", span))?
        .as_str()
        .to_string();

    // event_field = { identifier ~ ":" ~ identifier ~ (comma | semicolon)? }
    let mut fields: Vec<(String, Expr)> = Vec::new();
    for field_pair in inner {
        if field_pair.as_rule() == Rule::event_field {
            let mut fi = field_pair.into_inner();
            let field_name = match fi.next() {
                Some(p) => p.as_str().to_string(),
                None => continue,
            };
            let type_name = match fi.next() {
                Some(p) => p.as_str().to_string(),
                None => String::new(),
            };
            fields.push((field_name, Expr::Literal(Literal::String(type_name), span)));
        }
    }

    Ok(Stmt::Decl(Decl::Event { name, fields, span }))
}

/// Parse a contract declaration.
///
/// Syntax: `contract Name: { field: value, ... }`
///
/// Contracts follow the same key-value pattern as agent/tool/resource.
pub fn parse_contract_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected contract name", span))?
        .as_str()
        .to_string();

    let mut fields: Vec<(String, Expr)> = Vec::new();
    // contract_body contains contract_field items: identifier ~ ":" ~ expression
    if let Some(body_pair) = inner.next() {
        for field_pair in body_pair.into_inner() {
            if field_pair.as_rule() == Rule::contract_field {
                let mut fi = field_pair.into_inner();
                let key = match fi.next() {
                    Some(p) => p.as_str().to_string(),
                    None => continue,
                };
                let value = match fi.next() {
                    Some(p) => parse_expression(p)?,
                    None => continue,
                };
                fields.push((key, value));
            }
        }
    }

    Ok(Stmt::Decl(Decl::Contract { name, fields, span }))
}

/// Parse a treaty declaration.
///
/// Syntax: `treaty Name: { field: value, ... }`
///
/// Treaties follow the same key-value pattern as contracts.
pub fn parse_treaty_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected treaty name", span))?
        .as_str()
        .to_string();

    let mut fields: Vec<(String, Expr)> = Vec::new();
    // treaty_body contains treaty_field items: identifier ~ ":" ~ expression
    if let Some(body_pair) = inner.next() {
        for field_pair in body_pair.into_inner() {
            if field_pair.as_rule() == Rule::treaty_field {
                let mut fi = field_pair.into_inner();
                let key = match fi.next() {
                    Some(p) => p.as_str().to_string(),
                    None => continue,
                };
                let value = match fi.next() {
                    Some(p) => parse_expression(p)?,
                    None => continue,
                };
                fields.push((key, value));
            }
        }
    }

    Ok(Stmt::Decl(Decl::Treaty { name, fields, span }))
}

pub fn parse_class_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let source_text = pair.as_str();
    let is_abstract = source_text.trim_start().starts_with("abstract");
    let mut inner = pair.into_inner();

    // Parse class name
    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected class name", span))?
        .as_str()
        .to_string();

    // Check for generic params, parent class, implements, and members
    let mut type_params = Vec::new();
    let mut parent = None;
    let mut implements = Vec::new();
    let mut members = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::generic_params => {
                type_params = parse_generic_params(item)?;
            }
            Rule::identifier => {
                // Parent class name (after extends/<-)
                parent = Some(item.as_str().to_string());
            }
            Rule::implements_clause => {
                // Parse trait names from implements clause — Issue #659
                for trait_item in item.into_inner() {
                    if trait_item.as_rule() == Rule::identifier {
                        implements.push(trait_item.as_str().to_string());
                    }
                }
            }
            Rule::class_member => {
                members.push(parse_class_member(item)?);
            }
            _ => {}
        }
    }

    Ok(Stmt::Class(ClassDecl {
        name,
        parent,
        is_abstract,
        type_params,
        implements,
        members,
        span,
    }))
}

pub(super) fn parse_class_member(pair: Pair<Rule>) -> ParseResult<ClassMember> {
    let span = pair_to_span(&pair);

    let first = pair
        .into_inner()
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected class member", span))?;

    match first.as_rule() {
        Rule::class_constructor => parse_constructor_method(first),
        Rule::class_method => parse_class_method(first),
        Rule::class_field => parse_class_field(first),
        _ => Err(parse_codes::invalid_syntax(
            format!("Unknown class member type: {:?}", first.as_rule()),
            span,
        )),
    }
}

pub(super) fn parse_class_field(pair: Pair<Rule>) -> ParseResult<ClassMember> {
    let span = pair_to_span(&pair);

    let mut access = AccessModifier::Private;
    let mut is_static = false;
    let mut name = String::new();
    let mut initializer = None;

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::access_modifier => {
                access = parse_access_modifier(&item);
            }
            Rule::identifier => {
                name = item.as_str().to_string();
            }
            Rule::expression => {
                initializer = Some(parse_expression(item)?);
            }
            _ => {
                // static_kw, var_kw, let_kw, const_kw are silent - detected via text
                let text = item.as_str();
                if text == "static" {
                    is_static = true;
                }
            }
        }
    }

    Ok(ClassMember::Field {
        access,
        is_static,
        name,
        initializer,
        span,
    })
}

pub(super) fn parse_class_method(pair: Pair<Rule>) -> ParseResult<ClassMember> {
    let span = pair_to_span(&pair);

    let mut access = AccessModifier::Private;
    let mut is_static = false;
    let mut name = String::new();
    let mut params = Vec::new();
    let mut body = Vec::new();

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::access_modifier => {
                access = parse_access_modifier(&item);
            }
            Rule::static_marker => {
                is_static = true;
            }
            Rule::identifier => {
                if name.is_empty() {
                    name = item.as_str().to_string();
                } else {
                    params.push(Param {
                        name: item.as_str().to_string(),
                        type_annotation: None,
                        span: pair_to_span(&item),
                    });
                }
            }
            Rule::func_param => {
                let item_span = pair_to_span(&item);
                if let Some(child) = item.into_inner().next() {
                    let param_name = match child.as_rule() {
                        Rule::rest_param => {
                            let inner_name = child
                                .into_inner()
                                .next()
                                .map(|p| p.as_str().to_string())
                                .unwrap_or_default();
                            format!("...{}", inner_name)
                        }
                        _ => child.as_str().to_string(),
                    };
                    params.push(Param {
                        name: param_name,
                        type_annotation: None,
                        span: item_span,
                    });
                }
            }
            Rule::block => {
                if let Stmt::Block { statements, .. } = parse_block(item)? {
                    body = statements;
                }
            }
            _ => {}
        }
    }

    Ok(ClassMember::Method {
        access,
        is_static,
        name,
        params,
        body,
        span,
    })
}

pub(super) fn parse_constructor_method(pair: Pair<Rule>) -> ParseResult<ClassMember> {
    let span = pair_to_span(&pair);

    let mut params = Vec::new();
    let mut body = Vec::new();

    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::identifier => {
                let item_span = pair_to_span(&item);
                params.push(Param {
                    name: item.as_str().to_string(),
                    type_annotation: None,
                    span: item_span,
                });
            }
            Rule::func_param => {
                let item_span = pair_to_span(&item);
                if let Some(child) = item.into_inner().next() {
                    let name = match child.as_rule() {
                        Rule::rest_param => {
                            let inner_name = child
                                .into_inner()
                                .next()
                                .map(|p| p.as_str().to_string())
                                .unwrap_or_default();
                            format!("...{}", inner_name)
                        }
                        _ => child.as_str().to_string(),
                    };
                    params.push(Param {
                        name,
                        type_annotation: None,
                        span: item_span,
                    });
                }
            }
            Rule::block => {
                if let Stmt::Block { statements, .. } = parse_block(item)? {
                    body = statements;
                }
            }
            _ => {}
        }
    }

    Ok(ClassMember::Constructor { params, body, span })
}
