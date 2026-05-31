//! Provider and import declaration parsing

use hudhudscript_ast::{Decl, ImportKind, Stmt};
use pest::iterators::Pair;

use crate::error::{parse_codes, ParseResult};
use crate::parser::{pair_to_span, parse_expression};
use crate::pest_parser::Rule;

/// Parse a provider declaration
pub fn parse_provider_decl(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| parse_codes::invalid_syntax("Expected provider name", span))?
        .as_str()
        .to_string();

    let mut fields = Vec::new();

    for field_pair in inner {
        if let Rule::provider_field = field_pair.as_rule() {
            let mut field_inner = field_pair.into_inner();
            let key = field_inner.next().unwrap().as_str().to_string();
            let value = parse_expression(field_inner.next().unwrap())?;
            fields.push((key, value));
        }
    }

    Ok(Stmt::Decl(Decl::Provider { name, fields, span }))
}

/// Extract the module path string from a module_path pair.
fn extract_module_path(pair: Pair<Rule>) -> String {
    let module_path_inner = pair.into_inner().next().unwrap();
    match module_path_inner.as_rule() {
        Rule::string => {
            let s = module_path_inner.as_str();
            s[1..s.len() - 1].to_string()
        }
        _ => module_path_inner.as_str().to_string(),
    }
}

/// Parse an import statement
pub fn parse_import_stmt(pair: Pair<Rule>) -> ParseResult<Stmt> {
    let span = pair_to_span(&pair);
    let raw = pair.as_str().trim_start();

    // Detect ES6-style imports: text starts with "import"
    let is_es6 = raw.starts_with("import ");

    if is_es6 {
        // Determine which ES6 variant by looking at the raw text
        let after_import = raw["import".len()..].trim_start();

        if after_import.starts_with('{') {
            // ES6 named: import { a, b } from "path";
            let mut identifiers = Vec::new();
            let mut module = String::new();
            for token in pair.into_inner() {
                match token.as_rule() {
                    Rule::identifier => identifiers.push(token.as_str().to_string()),
                    Rule::module_path => module = extract_module_path(token),
                    _ => {}
                }
            }
            Ok(Stmt::Import {
                path: module,
                imports: ImportKind::Named(identifiers),
                span,
            })
        } else if after_import.starts_with('*') {
            // ES6 wildcard: import * as name from "path";
            let mut alias = String::new();
            let mut module = String::new();
            for token in pair.into_inner() {
                match token.as_rule() {
                    Rule::identifier => alias = token.as_str().to_string(),
                    Rule::module_path => module = extract_module_path(token),
                    _ => {}
                }
            }
            Ok(Stmt::Import {
                path: module,
                imports: ImportKind::Wildcard(alias),
                span,
            })
        } else {
            // ES6 default: import name from "path";
            let mut name = String::new();
            let mut module = String::new();
            for token in pair.into_inner() {
                match token.as_rule() {
                    Rule::identifier => name = token.as_str().to_string(),
                    Rule::module_path => module = extract_module_path(token),
                    _ => {}
                }
            }
            Ok(Stmt::Import {
                path: module,
                imports: ImportKind::Default(name),
                span,
            })
        }
    } else {
        // use-style imports (original HudHudScript syntax)
        let mut path: Option<String> = None;
        let mut alias: Option<String> = None;
        let mut items: Vec<String> = Vec::new();

        for token in pair.into_inner() {
            match token.as_rule() {
                Rule::module_path => {
                    path = Some(extract_module_path(token));
                }
                Rule::string => {
                    let s = token.as_str();
                    path = Some(s[1..s.len() - 1].to_string());
                }
                Rule::identifier => {
                    if alias.is_none() && path.is_some() {
                        alias = Some(token.as_str().to_string());
                    } else if path.is_none() {
                        path = Some(token.as_str().to_string());
                    } else {
                        items.push(token.as_str().to_string());
                    }
                }
                _ => {}
            }
        }

        let module =
            path.ok_or_else(|| parse_codes::invalid_syntax("Expected module path", span))?;

        Ok(Stmt::Decl(Decl::Import {
            module,
            alias,
            span,
        }))
    }
}
