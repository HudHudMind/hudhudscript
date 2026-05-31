//! Action declaration parsing

use hudhudscript_ast::{Decl, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::pair_to_span;
use crate::pest_parser::Rule;

use super::common::parse_field_body;

/// Parse an action declaration
pub fn parse_action_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected action name", span))?
        .as_str()
        .to_string();

    let body = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected action body", span))?;

    let fields = parse_field_body(body, Rule::action_field)?;

    Ok(Stmt::Decl(Decl::Action { name, fields, span }))
}
