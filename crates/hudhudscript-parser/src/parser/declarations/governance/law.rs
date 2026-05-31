use hudhudscript_ast::{Decl, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_expression};
use crate::pest_parser::Rule;

use super::parse_law_from_fields;

/// Parse a law declaration
pub fn parse_law_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected law name", span))?
        .as_str()
        .to_string();

    let body = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected law body", span))?;

    let fields = parse_law_body(body)?;

    let mut law = parse_law_from_fields(fields, span)?;
    law.name = name.clone();

    Ok(Stmt::Decl(Decl::Law {
        name: law.name,
        description: law.description,
        enforcement_level: law.enforcement_level,
        rules: law.rules,
        span,
    }))
}

fn parse_law_body(pair: Pair<Rule>) -> ParseResult<Vec<(String, hudhudscript_ast::Expr)>> {
    let mut fields = Vec::new();

    for inner_pair in pair.into_inner() {
        if let Rule::law_field = inner_pair.as_rule() {
            let mut field_inner = inner_pair.into_inner();
            let key = field_inner.next().unwrap().as_str().to_string();
            let value = parse_expression(field_inner.next().unwrap())?;
            fields.push((key, value));
        }
    }

    Ok(fields)
}
