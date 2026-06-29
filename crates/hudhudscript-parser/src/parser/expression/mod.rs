//! Expression parsing

use hudhudscript_ast::{
    ArrowFunctionBody, BinaryOp, Expr, Literal, Stmt, TemplateStringPart, UnaryOp,
};
use hudhudscript_lexer::is_reserved_keyword;
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::pest_parser::Rule;

use super::{arabic_to_ascii, japanese_numeral_to_number, pair_to_span, parse_block};

/// Parse escape sequences in a string literal (e.g., \"\\n\" → '\n')
fn parse_escape_sequences(s: &str) -> String {
    let mut value = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => value.push('\n'),
                Some('t') => value.push('\t'),
                Some('r') => value.push('\r'),
                Some('\\') => value.push('\\'),
                Some('"') => value.push('"'),
                Some(other) => {
                    value.push('\\');
                    value.push(other);
                }
                None => value.push('\\'),
            }
        } else {
            value.push(c);
        }
    }
    value
}

/// Parse an expression
pub fn parse_expression(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);

    match pair.as_rule() {
        Rule::expression => {
            // Recursively parse nested expressions
            let mut inner = pair.into_inner();
            if let Some(first) = inner.next() {
                parse_expression(first)
            } else {
                Err(parse_codes::invalid_syntax("Empty expression", span))
            }
        }
        Rule::ternary_expr => parse_ternary_expr(pair),
        Rule::logical_or_expr => parse_logical_or_expr(pair),
        Rule::logical_and_expr => parse_logical_and_expr(pair),
        Rule::equality_expr => parse_equality_expr(pair),
        Rule::comparison_expr => parse_comparison_expr(pair),
        Rule::additive_expr => parse_additive_expr(pair),
        Rule::multiplicative_expr => parse_multiplicative_expr(pair),
        Rule::unary_expr => parse_unary_expr(pair),
        Rule::postfix_expr => parse_postfix_expr(pair),
        Rule::primary => parse_primary(pair),
        Rule::new_expr => parse_new_expr(pair),
        Rule::spawn_expr => parse_spawn_expr(pair),
        Rule::view_expr => parse_view_expr(pair),
        Rule::arrow_function => parse_arrow_function(pair),
        Rule::anon_function => parse_anon_function(pair),
        Rule::template_string => parse_template_string(pair),
        Rule::array => parse_array(pair),
        Rule::object => parse_object(pair),
        Rule::number => parse_number(pair),
        Rule::string => parse_string(pair),
        Rule::boolean => parse_boolean(pair),
        Rule::null => Ok(Expr::Literal(Literal::Null, span)),
        Rule::this_kw_en => Ok(Expr::This(span)),
        Rule::self_kw_en => Ok(Expr::This(span)), // self_kw_en also maps to This
        Rule::identifier => parse_identifier(pair),
        _ => Err(parse_codes::invalid_syntax(
            format!("Unexpected rule: {:?}", pair.as_rule()),
            span,
        )),
    }
}

fn parse_ternary_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let condition = parse_expression(inner.next().ok_or_else(|| {
        parse_codes::invalid_syntax("Expected condition in ternary", span)
    })?)?;
    // ternary_q ("?") ve ternary_colon (":") SILENT — pairs'a girmez.
    // Eğer ternary ise bir sonraki pair doğrudan true_expr'dir.
    if let Some(true_pair) = inner.next() {
        let true_expr = parse_expression(true_pair)?;
        let false_expr = parse_expression(inner.next().ok_or_else(|| {
            parse_codes::invalid_syntax("Expected false expression in ternary", span)
        })?)?;
        Ok(Expr::Ternary { condition: Box::new(condition), true_expr: Box::new(true_expr), false_expr: Box::new(false_expr), span })
    } else {
        Ok(condition)
    }
}

fn parse_logical_or_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);

    // Collect all pairs
    let pairs: Vec<_> = pair.into_inner().collect();

    let mut inner = pairs.into_iter();
    let mut left = parse_logical_and_expr(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected left operand", span))?,
    )?;

    while let Some(_op_pair) = inner.next() {
        // op_pair is the operator (|| or or_kw)
        let right_pair = inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected right operand", span))?;
        let right = parse_logical_and_expr(right_pair)?;

        // All variants map to BinaryOp::Or
        left = Expr::Binary {
            left: Box::new(left),
            op: BinaryOp::Or,
            right: Box::new(right),
            span,
        };
    }

    Ok(left)
}

fn parse_logical_and_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);

    // Collect all pairs
    let pairs: Vec<_> = pair.into_inner().collect();

    let mut inner = pairs.into_iter();
    let mut left = parse_equality_expr(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected left operand", span))?,
    )?;

    while let Some(_op_pair) = inner.next() {
        // op_pair is the operator (&& or and_kw)
        let right_pair = inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected right operand", span))?;
        let right = parse_equality_expr(right_pair)?;

        // All variants map to BinaryOp::And
        left = Expr::Binary {
            left: Box::new(left),
            op: BinaryOp::And,
            right: Box::new(right),
            span,
        };
    }

    Ok(left)
}

fn parse_equality_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let mut left = parse_comparison_expr(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected left operand", span))?,
    )?;

    while let Some(op_pair) = inner.next() {
        let op_str = op_pair.as_str();

        let right = parse_comparison_expr(
            inner
                .next()
                .ok_or_else(|| parse_codes::invalid_syntax("Expected right operand", span))?,
        )?;

        let op = match op_str {
            "==" => BinaryOp::Eq,
            "!=" => BinaryOp::Ne,
            _ => {
                return Err(parse_codes::invalid_syntax(
                    format!("Unknown operator: {}", op_str),
                    span,
                ))
            }
        };

        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
            span,
        };
    }

    Ok(left)
}

fn parse_comparison_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let mut left = parse_additive_expr(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected left operand", span))?,
    )?;

    while let Some(op_pair) = inner.next() {
        match op_pair.as_rule() {
            Rule::instanceof_op => {
                // instanceof operator: left instanceof ClassName
                let class_name_pair = inner.next().ok_or_else(|| {
                    parse_codes::invalid_syntax("Expected class name after 'instanceof'", span)
                })?;
                let class_name = class_name_pair.as_str().to_string();
                let class_span = pair_to_span(&class_name_pair);
                left = Expr::Binary {
                    left: Box::new(left),
                    op: BinaryOp::InstanceOf,
                    right: Box::new(Expr::Identifier(class_name, class_span)),
                    span,
                };
            }
            _ => {
                let op_str = op_pair.as_str();
                let right =
                    parse_additive_expr(inner.next().ok_or_else(|| {
                        parse_codes::invalid_syntax("Expected right operand", span)
                    })?)?;

                let op = match op_str {
                    ">" => BinaryOp::Gt,
                    ">=" => BinaryOp::Ge,
                    "<" => BinaryOp::Lt,
                    "<=" => BinaryOp::Le,
                    _ => {
                        return Err(parse_codes::invalid_syntax(
                            format!("Unknown operator: {}", op_str),
                            span,
                        ))
                    }
                };

                left = Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    span,
                };
            }
        }
    }

    Ok(left)
}

fn parse_additive_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let mut left = parse_multiplicative_expr(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected left operand", span))?,
    )?;

    while let Some(op_pair) = inner.next() {
        let op_str = op_pair.as_str();
        let right = parse_multiplicative_expr(
            inner
                .next()
                .ok_or_else(|| parse_codes::invalid_syntax("Expected right operand", span))?,
        )?;

        let op = match op_str {
            "+" => BinaryOp::Add,
            "-" => BinaryOp::Sub,
            _ => {
                return Err(parse_codes::invalid_syntax(
                    format!("Unknown operator: {}", op_str),
                    span,
                ))
            }
        };

        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
            span,
        };
    }

    Ok(left)
}

fn parse_multiplicative_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let mut left = parse_unary_expr(
        inner
            .next()
            .ok_or_else(|| parse_codes::invalid_syntax("Expected left operand", span))?,
    )?;

    while let Some(op_pair) = inner.next() {
        let op_str = op_pair.as_str();
        let right = parse_unary_expr(
            inner
                .next()
                .ok_or_else(|| parse_codes::invalid_syntax("Expected right operand", span))?,
        )?;

        let op = match op_str {
            "*" => BinaryOp::Mul,
            "/" => BinaryOp::Div,
            "%" => BinaryOp::Mod,
            _ => {
                return Err(parse_codes::invalid_syntax(
                    format!("Unknown operator: {}", op_str),
                    span,
                ))
            }
        };

        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
            span,
        };
    }

    Ok(left)
}

pub mod unary_postfix;
pub mod functions;

pub use unary_postfix::*;
pub mod literals;
pub use literals::*;
pub use functions::*;
