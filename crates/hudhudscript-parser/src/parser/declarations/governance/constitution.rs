use hudhudscript_ast::{Decl, Expr, LawDecl, Literal, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_expression};
use crate::pest_parser::Rule;

/// Parse a constitution declaration
pub fn parse_constitution_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected constitution name", span))?
        .as_str()
        .to_string();

    let body = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected constitution body", span))?;

    let fields = parse_constitution_body(body)?;

    let mut _description = None;
    let mut laws = Vec::new();

    for (key, value) in &fields {
        match key.as_str() {
            "description" => {
                if let Expr::Literal(Literal::String(s), _) = value {
                    _description = Some(s.clone());
                }
            }
            "laws" => {
                if let Expr::Array {
                    elements: law_exprs,
                    ..
                } = value
                {
                    for law_expr in law_exprs {
                        if let Expr::Object {
                            properties: law_fields,
                            span: law_span,
                        } = law_expr
                        {
                            if let Ok(law) = parse_law_from_fields(law_fields.clone(), *law_span) {
                                laws.push(law);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::Constitution {
        name,
        description: _description,
        laws,
        span,
    }))
}

fn parse_constitution_body(pair: Pair<Rule>) -> ParseResult<Vec<(String, Expr)>> {
    let mut fields = Vec::new();

    for inner_pair in pair.into_inner() {
        if let Rule::constitution_field = inner_pair.as_rule() {
            let mut field_inner = inner_pair.into_inner();
            let key = field_inner.next().unwrap().as_str().to_string();
            let value = parse_expression(field_inner.next().unwrap())?;
            fields.push((key, value));
        }
    }

    Ok(fields)
}

pub(crate) fn parse_law_from_fields(
    fields: Vec<(String, Expr)>,
    span: hudhudscript_ast::Span,
) -> ParseResult<LawDecl> {
    let mut name = String::new();
    let mut description = String::new();
    let mut enforcement_level = String::from("mandatory");
    let mut rules = Vec::new();

    for (key, value) in fields {
        match key.as_str() {
            "name" => {
                if let Expr::Literal(Literal::String(s), _) = value {
                    name = s;
                }
            }
            "description" => {
                if let Expr::Literal(Literal::String(s), _) = value {
                    description = s;
                }
            }
            "enforcement" => {
                if let Expr::Identifier(s, _) = value {
                    enforcement_level = s;
                }
            }
            "rules" => {
                if let Expr::Array {
                    elements: rule_exprs,
                    ..
                } = value
                {
                    for rule_expr in rule_exprs {
                        rules.push(rule_expr);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(LawDecl {
        name,
        description,
        enforcement_level,
        rules,
        span,
    })
}
