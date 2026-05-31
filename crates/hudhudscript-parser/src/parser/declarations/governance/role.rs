use hudhudscript_ast::{Decl, Expr, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_expression};
use crate::pest_parser::Rule;

/// Parse a role declaration
///
/// Syntax: `role Fighter { can attack, can defend, description: "A fighter role" }`
pub fn parse_role_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected role name", span))?
        .as_str()
        .to_string();

    let mut capabilities = Vec::new();
    let mut fields = Vec::new();

    for member_pair in inner {
        if member_pair.as_rule() == Rule::role_member {
            let member_text = member_pair.as_str().trim();
            // can_kw is silent — detect by checking for ':' separator
            if member_text.contains(':') {
                // Field: identifier : expression
                let mut member_inner = member_pair.into_inner();
                if let Some(key_pair) = member_inner.next() {
                    let key = key_pair.as_str().to_string();
                    if let Some(val_pair) = member_inner.next() {
                        let value = parse_expression(val_pair)?;
                        fields.push((key, value));
                    }
                }
            } else {
                // can capability: identifier (comma-separated handled by multiple members)
                let mut member_inner = member_pair.into_inner();
                if let Some(cap) = member_inner.next() {
                    capabilities.push(cap.as_str().to_string());
                }
            }
        }
    }

    Ok(Stmt::Decl(Decl::Role {
        name,
        capabilities,
        fields,
        span,
    }))
}
