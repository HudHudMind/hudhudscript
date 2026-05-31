//! Common utilities for declaration parsing
//!
//! Provides shared functionality to avoid code duplication (DRY principle).

use hudhudscript_ast::Expr;
use pest::iterators::Pair;

use crate::error::ParseResult;
use crate::parser::parse_expression;
use crate::pest_parser::Rule;

/// Generic body parser for declarations with field: value pairs
///
/// This eliminates code duplication across agent, task, tool, resource parsers.
/// Follows DRY principle - define once, use everywhere.
pub fn parse_field_body(pair: Pair<Rule>, field_rule: Rule) -> ParseResult<Vec<(String, Expr)>> {
    let mut fields = Vec::new();

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == field_rule {
            let mut field_inner = inner_pair.into_inner();
            let key = field_inner.next().unwrap().as_str().to_string();
            let value = parse_expression(field_inner.next().unwrap())?;
            fields.push((key, value));
        }
    }

    Ok(fields)
}
