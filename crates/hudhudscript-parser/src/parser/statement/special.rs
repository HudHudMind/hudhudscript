use super::*;

pub(super) fn parse_access_modifier(pair: &Pair<Rule>) -> AccessModifier {
    match pair.as_str().trim() {
        "public" => AccessModifier::Public,
        "protected" => AccessModifier::Protected,
        _ => AccessModifier::Private,
    }
}

/// Parse a music declaration: note/chord/melody/harmony/rhythm/tempo/scale Name { key: value, ... }
pub fn parse_music_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let kind_pair = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected music kind", span))?;
    let kind = kind_pair.as_str().to_string();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected music declaration name", span))?
        .as_str()
        .to_string();

    let mut fields: Vec<(String, Expr)> = Vec::new();
    for field_pair in inner {
        if field_pair.as_rule() == Rule::music_field {
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

    Ok(Stmt::Decl(Decl::Music {
        kind,
        name,
        fields,
        span,
    }))
}

// ── SOP statement parsers ───────────────────────────────────────────────

pub(super) fn parse_spawn_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let subject_name = inner.next().unwrap().as_str().to_string();
    let mut args = Vec::new();
    for arg in inner {
        if arg.as_rule() == Rule::expression {
            args.push(parse_expression(arg)?);
        }
    }
    Ok(Stmt::Spawn {
        subject_name,
        args,
        span,
    })
}

pub(super) fn parse_despawn_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected subject name for despawn", span))?
        .as_str()
        .to_string();
    Ok(Stmt::Despawn { name, span })
}

pub(super) fn parse_send_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let msg_pair = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected message expression", span))?;
    let message = Box::new(parse_expression(msg_pair)?);
    let target_pair = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected target expression", span))?;
    let target = Box::new(parse_expression(target_pair)?);
    Ok(Stmt::Send {
        message,
        target,
        span,
    })
}

pub(super) fn parse_receive_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let variable = inner.next().unwrap().as_str().to_string();
    let source = Box::new(parse_expression(inner.next().unwrap())?);
    Ok(Stmt::Receive {
        variable,
        source,
        span,
    })
}

pub(super) fn parse_require_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let condition = Box::new(parse_expression(inner.next().unwrap())?);
    Ok(Stmt::Require { condition, span })
}

pub(super) fn parse_perform_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let action = Box::new(parse_expression(inner.next().unwrap())?);
    Ok(Stmt::Perform { action, span })
}

// ── RAG statement/declaration parsers ───────────────────────────────────

/// Parse a store declaration: store my_store { backend: "hnsw", dimensions: 1536 }
pub(super) fn parse_store_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected store name", span))?
        .as_str()
        .to_string();

    let mut fields: Vec<(String, Expr)> = Vec::new();
    for field_pair in inner {
        if field_pair.as_rule() == Rule::store_field {
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

    Ok(Stmt::Decl(Decl::Store { name, fields, span }))
}

/// Parse a remember statement: remember "text" in my_store;
pub(super) fn parse_remember_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let content =
        Box::new(parse_expression(inner.next().ok_or_else(|| {
            parse_codes::invalid_syntax("Expected expression", span)
        })?)?);

    let store_name = inner.next().map(|p| p.as_str().to_string());

    Ok(Stmt::Remember {
        content,
        store_name,
        span,
    })
}

/// Parse a recall statement: recall "query" from my_store;
pub(super) fn parse_recall_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let query =
        Box::new(parse_expression(inner.next().ok_or_else(|| {
            parse_codes::invalid_syntax("Expected expression", span)
        })?)?);

    let store_name = inner.next().map(|p| p.as_str().to_string());

    Ok(Stmt::Recall {
        query,
        store_name,
        span,
    })
}

/// Parse a data declaration: data { key = value; ... } or data MyData { key = value; ... }
///
/// Produces Stmt::Expr wrapping an Object literal with the key-value pairs.
pub(super) fn parse_data_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let inner_pairs: Vec<_> = pair.into_inner().collect();

    let mut name: Option<String> = None;
    let mut fields: Vec<(String, Expr)> = Vec::new();

    for p in inner_pairs {
        match p.as_rule() {
            Rule::identifier => {
                // If we haven't seen fields yet, this is the optional name
                if fields.is_empty() && name.is_none() {
                    name = Some(p.as_str().to_string());
                }
            }
            Rule::data_field => {
                let mut fi = p.into_inner();
                let key = match fi.next() {
                    Some(k) => k.as_str().to_string(),
                    None => continue,
                };
                let value = match fi.next() {
                    Some(v) => parse_expression(v)?,
                    None => continue,
                };
                fields.push((key, value));
            }
            _ => {}
        }
    }

    // Build an object literal from the fields
    let properties: Vec<(String, Expr)> = fields;
    let obj_expr = Expr::Object { properties, span };

    // If named, store as a let binding; otherwise just an expression statement
    if let Some(data_name) = name {
        Ok(Stmt::Let {
            name: data_name,
            value: obj_expr,
            span,
        })
    } else {
        Ok(Stmt::Expr(obj_expr))
    }
}

/// Parse an export statement.
///
/// Grammar alternatives:
///   export { func1, func2 } from "module";  (re-export)
///   export { func1, func2 };
///   export default expression;
///   export function/class/var/let/const declaration
///   export identifier;
pub(super) fn parse_export_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let inner_pairs: Vec<_> = pair.into_inner().collect();

    // The grammar's first child after export_kw varies by alternative.
    // We need to figure out what we got.
    if inner_pairs.is_empty() {
        return Err(parse_codes::invalid_syntax("Empty export statement", span));
    }

    // Check if this is a re-export: identifiers followed by a module_path
    // The grammar puts module_path as the last child when `from` clause is present.
    let has_source = inner_pairs.iter().any(|p| p.as_rule() == Rule::module_path);

    let first = inner_pairs[0].clone();
    match first.as_rule() {
        // export declaration (function, class, var, let, const)
        Rule::async_function_decl => {
            let item = parse_async_function_decl(first)?;
            Ok(Stmt::Export {
                item: Box::new(item),
                source: None,
                span,
            })
        }
        Rule::function_decl => {
            let item = parse_function_decl(first)?;
            Ok(Stmt::Export {
                item: Box::new(item),
                source: None,
                span,
            })
        }
        Rule::class_decl => {
            let item = parse_class_decl(first)?;
            Ok(Stmt::Export {
                item: Box::new(item),
                source: None,
                span,
            })
        }
        Rule::var_stmt => {
            let item = parse_var_stmt(first)?;
            Ok(Stmt::Export {
                item: Box::new(item),
                source: None,
                span,
            })
        }
        Rule::let_stmt => {
            let item = parse_let_stmt(first)?;
            Ok(Stmt::Export {
                item: Box::new(item),
                source: None,
                span,
            })
        }
        Rule::const_stmt => {
            let item = parse_const_stmt(first)?;
            Ok(Stmt::Export {
                item: Box::new(item),
                source: None,
                span,
            })
        }
        // export { ident1, ident2 } [from "module"] or export default expr or export ident
        Rule::identifier => {
            // Could be "default" followed by expression, or just identifiers
            let first_str = first.as_str();
            if first_str == "default" {
                // export default expression
                if let Some(expr_pair) = inner_pairs.into_iter().nth(1) {
                    let expr = parse_expression(expr_pair)?;
                    let item = Stmt::Expr(expr);
                    Ok(Stmt::Export {
                        item: Box::new(item),
                        source: None,
                        span,
                    })
                } else {
                    Err(parse_codes::invalid_syntax(
                        "Expected expression after export default",
                        span,
                    ))
                }
            } else {
                // Extract module source path if present (re-export)
                let source = if has_source {
                    inner_pairs
                        .iter()
                        .find(|p| p.as_rule() == Rule::module_path)
                        .map(|p| {
                            let raw = p.as_str();
                            // Strip surrounding quotes if present
                            raw.trim_matches('"').trim_matches('\'').to_string()
                        })
                } else {
                    None
                };

                // export { ident1, ident2 } or export ident — collect all identifiers
                // Wrap as a block of expression statements for multiple exports
                let names: Vec<Stmt> = inner_pairs
                    .into_iter()
                    .filter(|p| p.as_rule() == Rule::identifier)
                    .map(|p| {
                        let s = pair_to_span(&p);
                        Stmt::Expr(Expr::Identifier(p.as_str().to_string(), s))
                    })
                    .collect();
                if names.len() == 1 {
                    Ok(Stmt::Export {
                        item: Box::new(names.into_iter().next().unwrap()),
                        source,
                        span,
                    })
                } else {
                    let block = Stmt::Block {
                        statements: names,
                        span,
                    };
                    Ok(Stmt::Export {
                        item: Box::new(block),
                        source,
                        span,
                    })
                }
            }
        }
        Rule::expression => {
            // export default expression (when "default" is consumed by grammar)
            let expr = parse_expression(first)?;
            let item = Stmt::Expr(expr);
            Ok(Stmt::Export {
                item: Box::new(item),
                source: None,
                span,
            })
        }
        _ => {
            // Fallback: treat as expression
            let expr = parse_expression(first)?;
            Ok(Stmt::Export {
                item: Box::new(Stmt::Expr(expr)),
                source: None,
                span,
            })
        }
    }
}
