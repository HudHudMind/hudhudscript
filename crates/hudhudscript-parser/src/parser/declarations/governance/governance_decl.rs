use hudhudscript_ast::Decl;
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_expression};
use crate::pest_parser::Rule;

use super::normalize_governance;

/// Parse a governance declaration: governance MyGov: democracy { ... }
pub fn parse_governance_decl(pair: Pair<Rule>) -> ParseResult<hudhudscript_ast::Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected governance name", span))?
        .as_str()
        .to_string();

    let mut base_type = String::from("default");
    let mut pending_field: Option<pest::iterators::Pair<Rule>> = None;

    if let Some(next) = inner.next() {
        if next.as_rule() == Rule::identifier {
            let base_type_raw = next.as_str().to_string();
            base_type = normalize_governance(&base_type_raw);
        } else {
            pending_field = Some(next);
        }
    }

    let mut fields = Vec::new();
    if let Some(fp) = pending_field {
        if fp.as_rule() == Rule::governance_field {
            let mut fi = fp.into_inner();
            if let (Some(k), Some(v)) = (fi.next(), fi.next()) {
                let value = parse_expression(v)?;
                fields.push((k.as_str().to_string(), value));
            }
        }
    }
    for field_pair in inner {
        if field_pair.as_rule() == Rule::governance_field {
            let mut fi = field_pair.into_inner();
            let key = fi.next().unwrap().as_str().to_string();
            let value = parse_expression(fi.next().unwrap())?;
            fields.push((key, value));
        }
    }

    Ok(hudhudscript_ast::Stmt::Decl(Decl::Governance {
        name,
        base_type,
        fields,
        span,
    }))
}
