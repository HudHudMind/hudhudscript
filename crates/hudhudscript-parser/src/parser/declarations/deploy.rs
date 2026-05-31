//! Deploy declaration parsing (#539)
//!
//! Parses `deploy App { target { web { ... } } github { ... } }`.

use hudhudscript_ast::{Decl, DeployProviderDecl, DeployTargetDecl, Expr, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_expression};
use crate::pest_parser::Rule;

/// Parse a deploy declaration: `deploy MyApp { target { ... } github { ... } }`
pub fn parse_deploy_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected deploy name", span))?
        .as_str()
        .to_string();

    let mut targets: Vec<DeployTargetDecl> = Vec::new();
    let mut providers: Vec<DeployProviderDecl> = Vec::new();
    let mut fields: Vec<(String, Expr)> = Vec::new();

    for member_pair in inner {
        if member_pair.as_rule() == Rule::deploy_member {
            for child in member_pair.into_inner() {
                match child.as_rule() {
                    Rule::deploy_target => {
                        targets.extend(parse_deploy_target(child)?);
                    }
                    Rule::deploy_provider => {
                        providers.push(parse_deploy_provider(child)?);
                    }
                    Rule::deploy_field => {
                        if let Some(field) = parse_deploy_field(child)? {
                            fields.push(field);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(Stmt::Decl(Decl::Deploy {
        name,
        targets,
        providers,
        fields,
        span,
    }))
}

/// Parse a deploy target block: `target { web { ... } desktop { ... } }`
/// Returns multiple DeployTargetDecl (one per platform entry)
fn parse_deploy_target(pair: Pair<Rule>) -> ParseResult<Vec<DeployTargetDecl>> {
    let mut targets = Vec::new();

    for entry in pair.into_inner() {
        if entry.as_rule() == Rule::deploy_target_entry {
            let span = pair_to_span(&entry);
            let mut inner = entry.into_inner();

            let platform = inner
                .next()
                .ok_or_else(|| parse_codes::invalid_syntax("Expected platform name", span))?
                .as_str()
                .to_string();

            let mut fields: Vec<(String, Expr)> = Vec::new();
            for field_pair in inner {
                if field_pair.as_rule() == Rule::deploy_field {
                    if let Some(field) = parse_deploy_field(field_pair)? {
                        fields.push(field);
                    }
                }
            }

            targets.push(DeployTargetDecl {
                platform,
                fields,
                span,
            });
        }
    }

    Ok(targets)
}

/// Parse a deploy provider: `github { repo: "...", on: "push" }`
fn parse_deploy_provider(pair: Pair<Rule>) -> ParseResult<DeployProviderDecl> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected provider name", span))?
        .as_str()
        .to_string();

    let mut fields: Vec<(String, Expr)> = Vec::new();
    for field_pair in inner {
        if field_pair.as_rule() == Rule::deploy_field {
            if let Some(field) = parse_deploy_field(field_pair)? {
                fields.push(field);
            }
        }
    }

    Ok(DeployProviderDecl { name, fields, span })
}

/// Parse a single deploy field: `key: value`
fn parse_deploy_field(pair: Pair<Rule>) -> ParseResult<Option<(String, Expr)>> {
    let mut inner = pair.into_inner();
    let key = match inner.next() {
        Some(p) => p.as_str().to_string(),
        None => return Ok(None),
    };
    let value = match inner.next() {
        Some(p) => parse_expression(p)?,
        None => return Ok(None),
    };
    Ok(Some((key, value)))
}
