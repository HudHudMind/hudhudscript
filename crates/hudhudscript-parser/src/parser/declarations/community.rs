//! Community declaration parsing

use hudhudscript_ast::{CultureDecl, Decl, Expr, Literal, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_expression};
use crate::pest_parser::Rule;

/// Parse a community declaration
pub fn parse_community_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected community name", span))?
        .as_str()
        .to_string();

    let body = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected community body", span))?;

    let fields = parse_community_body(body)?;

    // Extract fields
    let mut members = Vec::new();
    let mut councils = Vec::new();
    let mut culture = CultureDecl {
        values: Vec::new(),
        norms: Vec::new(),
        communication_style: String::from("formal"),
        span,
    };

    for (key, value) in fields {
        match key.as_str() {
            "members" => {
                if let Expr::Array {
                    elements: member_exprs,
                    ..
                } = value
                {
                    for member_expr in member_exprs {
                        if let Expr::Literal(Literal::String(s), _) = &member_expr {
                            members.push(s.clone());
                        } else if let Expr::Identifier(s, _) = &member_expr {
                            members.push(s.clone());
                        }
                    }
                }
            }
            "councils" => {
                if let Expr::Array {
                    elements: council_exprs,
                    ..
                } = value
                {
                    for council_expr in council_exprs {
                        if let Expr::Literal(Literal::String(s), _) = &council_expr {
                            councils.push(s.clone());
                        } else if let Expr::Identifier(s, _) = &council_expr {
                            councils.push(s.clone());
                        }
                    }
                }
            }
            "culture" => {
                if let Expr::Object {
                    properties: culture_fields,
                    span: culture_span,
                } = value
                {
                    culture = parse_culture_from_fields(culture_fields, culture_span)?;
                }
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::Community {
        name,
        members,
        councils,
        culture,
        span,
    }))
}

fn parse_community_body(pair: Pair<Rule>) -> ParseResult<Vec<(String, Expr)>> {
    let mut fields = Vec::new();

    for inner_pair in pair.into_inner() {
        if let Rule::community_field = inner_pair.as_rule() {
            let mut field_inner = inner_pair.into_inner();
            let key = field_inner.next().unwrap().as_str().to_string();
            let value = parse_expression(field_inner.next().unwrap())?;
            fields.push((key, value));
        }
    }

    Ok(fields)
}

fn parse_culture_from_fields(
    fields: Vec<(String, Expr)>,
    span: hudhudscript_ast::Span,
) -> ParseResult<CultureDecl> {
    let mut values = Vec::new();
    let mut norms = Vec::new();
    let mut communication_style = String::from("formal");

    for (key, val) in fields {
        match key.as_str() {
            "values" => {
                if let Expr::Array {
                    elements: value_exprs,
                    ..
                } = val
                {
                    for value_expr in value_exprs {
                        if let Expr::Literal(Literal::String(s), _) = value_expr {
                            values.push(s);
                        }
                    }
                }
            }
            "norms" => {
                if let Expr::Array {
                    elements: norm_exprs,
                    ..
                } = val
                {
                    for norm_expr in norm_exprs {
                        if let Expr::Literal(Literal::String(s), _) = norm_expr {
                            norms.push(s);
                        }
                    }
                }
            }
            "communication_style" => {
                if let Expr::Identifier(s, _) = val {
                    communication_style = s;
                } else if let Expr::Literal(Literal::String(s), _) = val {
                    communication_style = s;
                }
            }
            _ => {}
        }
    }

    Ok(CultureDecl {
        values,
        norms,
        communication_style,
        span,
    })
}
