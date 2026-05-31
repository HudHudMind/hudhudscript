//! Subject declaration parsing (SOP)
//!
//! Handles parsing of subject, relation, and effect declarations.

use hudhudscript_ast::{Decl, Decorator, Expr, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_expression, parse_statement};
use crate::parser::statement::declarations::parse_block;
use crate::pest_parser::Rule;

/// Parse a subject declaration
///
/// Syntax: `subject Player has Fighter, Trader { state health: 100, can attack, on attack(...) {...} }`
pub fn parse_subject_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected subject name", span))?
        .as_str()
        .to_string();

    // Collect of_subject, roles (from `of` and `has` clauses) and body members
    let mut of_subject = None;
    let mut roles = Vec::new();
    let mut states = Vec::new();
    let mut capabilities = Vec::new();
    let mut ability_defs = Vec::new();
    let mut intents = Vec::new();
    let mut uses = Vec::new();
    let mut memory = Vec::new();
    let mut perception = Vec::new();
    let mut fields = Vec::new();

    for pair in inner {
        match pair.as_rule() {
            Rule::of_clause => {
                of_subject = Some(pair.into_inner().next()
                    .map(|p| p.as_str().to_string())
                    .unwrap_or_default());
            }
            Rule::has_clause => {
                for ident in pair.into_inner() {
                    roles.push(ident.as_str().to_string());
                }
            }
            Rule::subject_body => {
                for member in pair.into_inner() {
                    match member.as_rule() {
                        Rule::subject_member => {
                            parse_subject_member(
                                member,
                                &mut states,
                                &mut capabilities,
                                &mut ability_defs,
                                &mut intents,
                                &mut uses,
                                &mut memory,
                                &mut perception,
                                &mut fields,
                            )?;
                        }
                        Rule::ability_decl => {
                            // on attack(...) { ... } directly inside subject body
                            let span = pair_to_span(&member);
                            let stmt = parse_ability_decl(member)?;
                            if let Stmt::Decl(Decl::Ability { name, params, body, .. }) = stmt {
                                capabilities.push(name.clone());
                                ability_defs.push(hudhudscript_ast::SubjectAbilityDef {
                                    name,
                                    params,
                                    body,
                                    span,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::Subject {
        name,
        decorators: Vec::new(), // Decorators parsed via decorated_subject_decl
        of_subject,
        roles,
        states,
        capabilities,
        ability_defs,
        intents,
        uses,
        memory,
        perception,
        fields,
        span,
    }))
}

#[allow(clippy::too_many_arguments)]
fn parse_subject_member(
    pair: Pair<Rule>,
    states: &mut Vec<(String, Expr)>,
    capabilities: &mut Vec<String>,
    ability_defs: &mut Vec<hudhudscript_ast::SubjectAbilityDef>,
    intents: &mut Vec<String>,
    uses: &mut Vec<(String, Option<String>)>,
    memory: &mut Vec<(String, Expr)>,
    perception: &mut Vec<(String, Expr)>,
    fields: &mut Vec<(String, Expr)>,
) -> ParseResult<()> {
    // Keywords (state_kw, can_kw, etc.) are silent rules in pest,
    // so we detect the member type from the raw text of the member.
    let text = pair.as_str().trim();
    let pairs: Vec<Pair<Rule>> = pair.into_inner().collect();

    if text.starts_with("on ") || text.starts_with("on\t") {
        // on attack(self, target) { body } — capability definition inside subject
        // First pair is the capability name (with params), rest is body
        if let Some(first) = pairs.first() {
            let cap_text = first.as_str().trim();
            // Parse "attack(self, target)" format
            if let Some(paren_pos) = cap_text.find('(') {
                let cap_name = cap_text[..paren_pos].trim().to_string();
                let params_str = &cap_text[paren_pos + 1..];
                let params: Vec<String> = params_str
                    .trim_end_matches(')')
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                let mut body = Vec::new();
                for p in &pairs[1..] {
                    if p.as_rule() == Rule::block {
                        let block_stmt = parse_block(p.clone())?;
                        if let Stmt::Block { statements, .. } = block_stmt {
                            body = statements;
                        }
                    }
                }
                let span = pair_to_span(&first);
                ability_defs.push(hudhudscript_ast::SubjectAbilityDef {
                    name: cap_name,
                    params,
                    body,
                    span,
                });
                // Also add to capabilities list so `can` reflects it
                if let Some(last) = ability_defs.last() {
                    capabilities.push(last.name.clone());
                }
            }
        }
    } else if text.starts_with("state ") || text.starts_with("state\t") {
        // state health: 100 — first pair is identifier (name), second is expression (value)
        if pairs.len() >= 2 {
            let name = pairs[0].as_str().to_string();
            let value = parse_expression(pairs[1].clone())?;
            states.push((name, value));
        }
    } else if text.starts_with("can ") || text.starts_with("can\t") {
        // can attack — first pair is identifier (capability name)
        if let Some(p) = pairs.first() {
            capabilities.push(p.as_str().to_string());
        }
    } else if text.starts_with("intent ") || text.starts_with("intent\t") {
        // intent Attack — first pair is identifier (intent name)
        if let Some(p) = pairs.first() {
            intents.push(p.as_str().to_string());
        }
    } else if text.starts_with("uses ") || text.starts_with("uses\t") {
        // uses provider_name [via channel] — identifiers
        let provider = pairs
            .first()
            .map(|p| p.as_str().to_string())
            .unwrap_or_default();
        // If there's a second identifier, it's the via channel (via_kw is silent)
        let via_channel = if pairs.len() >= 2 {
            Some(pairs[1].as_str().to_string())
        } else {
            None
        };
        uses.push((provider, via_channel));
    } else if text.starts_with("memory ")
        || text.starts_with("memory\t")
        || text.starts_with("memory{")
    {
        // memory { key: value, ... }
        let mut i = 0;
        while i < pairs.len() {
            if pairs[i].as_rule() == Rule::identifier {
                let key = pairs[i].as_str().to_string();
                if i + 1 < pairs.len() {
                    let value = parse_expression(pairs[i + 1].clone())?;
                    memory.push((key, value));
                    i += 2;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    } else if text.starts_with("perception ")
        || text.starts_with("perception\t")
        || text.starts_with("perception{")
    {
        // perception { key: value, ... }
        let mut i = 0;
        while i < pairs.len() {
            if pairs[i].as_rule() == Rule::identifier {
                let key = pairs[i].as_str().to_string();
                if i + 1 < pairs.len() {
                    let value = parse_expression(pairs[i + 1].clone())?;
                    perception.push((key, value));
                    i += 2;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    } else {
        // Regular field: key: value
        if pairs.len() >= 2 && pairs[0].as_rule() == Rule::identifier {
            let key = pairs[0].as_str().to_string();
            let value = parse_expression(pairs[1].clone())?;
            fields.push((key, value));
        } else if let Some(first) = pairs.first() {
            if first.as_rule() == Rule::identifier {
                // Standalone identifier — treat as a capability
                fields.push((
                    first.as_str().to_string(),
                    Expr::Literal(
                        hudhudscript_ast::Literal::Boolean(true),
                        hudhudscript_ast::Span::default(),
                    ),
                ));
            }
        }
    }

    Ok(())
}

/// Parse a relation declaration
///
/// Syntax: `relation Player <-> Merchant { trust: 50 }`
pub fn parse_relation_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let subject_a = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected first subject name", span))?
        .as_str()
        .to_string();

    let subject_b = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected second subject name", span))?
        .as_str()
        .to_string();

    let mut fields = Vec::new();
    for pair in inner {
        if pair.as_rule() == Rule::relation_field {
            let mut field_inner = pair.into_inner();
            let key = field_inner.next().unwrap().as_str().to_string();
            let value = parse_expression(field_inner.next().unwrap())?;
            fields.push((key, value));
        }
    }

    Ok(Stmt::Decl(Decl::Relation {
        subject_a,
        subject_b,
        fields,
        span,
    }))
}

/// Parse an effect declaration
///
/// Syntax: `effect on Damage(target, amount) { body }`
pub fn parse_effect_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let event_name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected event name", span))?
        .as_str()
        .to_string();

    let mut params = Vec::new();
    let mut body = Vec::new();
    for item in inner {
        match item.as_rule() {
            Rule::func_param => {
                for param_inner in item.into_inner() {
                    if param_inner.as_rule() == Rule::identifier {
                        params.push(param_inner.as_str().to_string());
                    }
                }
            }
            Rule::block => {
                let block_stmt = parse_block(item)?;
                if let Stmt::Block { statements, .. } = block_stmt {
                    body = statements;
                }
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::Effect {
        event_name,
        params,
        body,
        span,
    }))
}

/// Parse a capability declaration: on attack(self, target) { body }
/// or subject-scoped: on Knight.attack(self, target) { body }
pub fn parse_ability_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    use hudhudscript_ast::Decl;
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let raw_name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected ability name", span))?
        .as_str()
        .to_string();

    // Support subject-scoped capability: on Knight.attack(...) → subject_type=Knight, name=attack
    let (subject_type, name) = if let Some(dot_pos) = raw_name.find('.') {
        let subj = raw_name[..dot_pos].to_string();
        let cap = raw_name[dot_pos + 1..].to_string();
        if cap.is_empty() {
            return Err(parse_codes::invalid_syntax(
                "Expected ability name after . in subject-scoped ability",
                span,
            ));
        }
        (Some(subj), cap)
    } else {
        (None, raw_name)
    };

    let mut params = Vec::new();
    let mut body_stmts = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::func_param => {
                for param_inner in item.into_inner() {
                    if param_inner.as_rule() == Rule::identifier {
                        params.push(param_inner.as_str().to_string());
                    }
                }
            }
            Rule::block => {
                let block_stmt = parse_block(item)?;
                if let Stmt::Block { statements, .. } = block_stmt {
                    body_stmts = statements;
                }
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::Ability {
        name,
        subject_type,
        params,
        body: body_stmts,
        span,
    }))
}

/// Parse a decorator: @ai, @payment(provider: "stripe"), etc.
fn parse_decorator(pair: Pair<Rule>) -> ParseResult<Decorator> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected decorator name", span))?
        .as_str()
        .to_string();

    let mut params = Vec::new();
    for p in inner {
        if p.as_rule() == Rule::decorator_params {
            for param_pair in p.into_inner() {
                if param_pair.as_rule() == Rule::decorator_param {
                    let mut param_inner = param_pair.into_inner();
                    let key = param_inner.next().unwrap().as_str().to_string();
                    if let Some(value_pair) = param_inner.next() {
                        let value = parse_expression(value_pair)?;
                        params.push((key, value));
                    }
                }
            }
        }
    }

    Ok(Decorator { name, params, span })
}

/// Parse a decorated subject declaration: @ai @payment subject Player has Fighter { ... }
pub fn parse_decorated_subject_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let inner = pair.into_inner();

    // Parse decorators first
    let mut decorators = Vec::new();
    let mut name = String::new();
    let mut of_subject = None;
    let mut roles = Vec::new();
    let mut states = Vec::new();
    let mut capabilities = Vec::new();
    let mut ability_defs = Vec::new();
    let mut intents = Vec::new();
    let mut uses = Vec::new();
    let mut memory = Vec::new();
    let mut perception = Vec::new();
    let mut fields = Vec::new();
    let mut found_name = false;

    for p in inner {
        match p.as_rule() {
            Rule::decorator => {
                decorators.push(parse_decorator(p)?);
            }
            Rule::identifier if !found_name => {
                name = p.as_str().to_string();
                found_name = true;
            }
            Rule::of_clause => {
                of_subject = Some(p.into_inner().next()
                    .map(|ident| ident.as_str().to_string())
                    .unwrap_or_default());
            }
            Rule::has_clause => {
                for ident in p.into_inner() {
                    roles.push(ident.as_str().to_string());
                }
            }
            Rule::subject_body => {
                for member in p.into_inner() {
                    match member.as_rule() {
                        Rule::subject_member => {
                            parse_subject_member(
                                member,
                                &mut states,
                                &mut capabilities,
                                &mut ability_defs,
                                &mut intents,
                                &mut uses,
                                &mut memory,
                                &mut perception,
                                &mut fields,
                            )?;
                        }
                        Rule::ability_decl => {
                            let span = pair_to_span(&member);
                            let stmt = parse_ability_decl(member)?;
                            if let Stmt::Decl(Decl::Ability { name, params, body, .. }) = stmt {
                                capabilities.push(name.clone());
                                ability_defs.push(hudhudscript_ast::SubjectAbilityDef {
                                    name,
                                    params,
                                    body,
                                    span,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::Subject {
        name,
        decorators,
        of_subject,
        roles,
        states,
        capabilities,
        ability_defs,
        intents,
        uses,
        memory,
        perception,
        fields,
        span,
    }))
}
