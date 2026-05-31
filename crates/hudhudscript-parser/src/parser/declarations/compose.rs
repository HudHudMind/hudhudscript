//! SOP0007: Composition rules parsing for Harrison & Ossher SOP

use hudhudscript_ast::{ComposeMode, ComposeRule, Decl, FieldCorrespondence, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::pair_to_span;
use crate::pest_parser::Rule;

/// Parse a compose declaration: compose Knight { on attack: combine [A, B] }
pub fn parse_compose_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let base_subject = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected subject name in compose", span))?
        .as_str()
        .to_string();

    let mut rules = Vec::new();
    let mut field_rules = Vec::new();
    for rule_pair in inner {
        match rule_pair.as_rule() {
            Rule::compose_rule => {
                let mut rule_inner = rule_pair.into_inner();
                let ability_name = rule_inner
                    .next()
                    .map(|p| p.as_str().to_string())
                    .unwrap_or_default();
                if let Some(mode_pair) = rule_inner.next() {
                    let mode = parse_compose_mode(mode_pair)?;
                    rules.push(ComposeRule { ability_name, mode });
                }
            }
            Rule::compose_state_rule => {
                let mut state_inner = rule_pair.into_inner();
                let field_name = state_inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
                let mode_text = state_inner.next().map(|p| p.as_str().trim().to_string()).unwrap_or_default();
                let corr = if mode_text == "correspond" {
                    FieldCorrespondence::Correspond
                } else {
                    FieldCorrespondence::Separate
                };
                field_rules.push((field_name, corr));
            }
            _ => {}
        }
    }

    Ok(Stmt::Decl(Decl::Compose {
        base_subject,
        rules,
        field_rules,
        span,
    }))
}

fn parse_compose_mode(pair: Pair<Rule>) -> ParseResult<ComposeMode> {
    let span = pair_to_span(&pair);
    let raw = pair.as_str().trim().to_string();
    let inner: Vec<Pair<Rule>> = pair.into_inner().collect();

    if raw.starts_with("combine") {
        let subjects: Vec<String> = inner
            .iter()
            .filter(|p| p.as_rule() == Rule::identifier)
            .map(|p| p.as_str().to_string())
            .collect();
        if subjects.is_empty() {
            return Err(parse_codes::invalid_syntax("combine requires at least one subject", span));
        }
        Ok(ComposeMode::Combine(subjects))
    } else if raw.starts_with("override") {
        let subject = inner.first().map(|p| p.as_str().to_string()).unwrap_or_default();
        Ok(ComposeMode::Override(subject))
    } else if raw.starts_with("before") {
        let subject = inner.first().map(|p| p.as_str().to_string()).unwrap_or_default();
        Ok(ComposeMode::Before(subject))
    } else if raw.starts_with("after") {
        let subject = inner.first().map(|p| p.as_str().to_string()).unwrap_or_default();
        Ok(ComposeMode::After(subject))
    } else {
        Err(parse_codes::invalid_syntax(
            &format!("Unknown composition mode: {}", raw),
            span,
        ))
    }
}
