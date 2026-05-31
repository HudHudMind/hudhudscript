//! Pest-based multi-language parser
//!
//! Parses HudHudScript source code in English, Turkish, or Japanese
//! using PEG grammar and produces a unified AST.

use hudhudscript_ast::Stmt;
use hudhudscript_lexer::normalize_keywords;
use pest::Parser;
use pest_derive::Parser;

use crate::error::{parse_codes, ParseResult};
use crate::parser;

#[derive(Parser)]
#[grammar = "grammar/languages/en.pest"]
#[grammar = "grammar/languages/tr.pest"]
#[grammar = "grammar/languages/ja.pest"]
#[grammar = "grammar/languages/ar.pest"]
#[grammar = "grammar/languages/bn.pest"]
#[grammar = "grammar/languages/ru.pest"]
#[grammar = "grammar/languages/es.pest"]
#[grammar = "grammar/languages/de.pest"]
#[grammar = "grammar/languages/fr.pest"]
#[grammar = "grammar/languages/it.pest"]
#[grammar = "grammar/languages/pt.pest"]
#[grammar = "grammar/languages/zh.pest"]
#[grammar = "grammar/languages/hi.pest"]
#[grammar = "grammar/languages/id.pest"]
#[grammar = "grammar/languages/vi.pest"]
#[grammar = "grammar/languages/el.pest"]
#[grammar = "grammar/languages/pl.pest"]
#[grammar = "grammar/languages/th.pest"]
#[grammar = "grammar/languages/bs.pest"]
#[grammar = "grammar/languages/hr.pest"]
#[grammar = "grammar/languages/sr.pest"]
#[grammar = "grammar/languages/fa.pest"]
#[grammar = "grammar/languages/ku.pest"]
#[grammar = "grammar/literals.pest"]
#[grammar = "grammar/base.pest"]
pub struct HudHudParser;

/// Parse source code into AST.
///
/// If the source starts with a `#!lang=...` (or equivalent) shebang directive,
/// it is automatically stripped before parsing so it does not cause a syntax error.
/// To also retrieve the locale code, use [`parse_with_lang_directive`].
pub fn parse(source: &str) -> ParseResult<Vec<Stmt>> {
    let (_locale, stmts) = parse_with_lang_directive(source)?;
    Ok(stmts)
}

/// Parse source code into AST, also returning the language directive locale (if any).
///
/// The first line may contain a shebang-style directive such as `#!lang=tr` or
/// `#!dil=tr`. If present, the directive is stripped before parsing and the locale
/// code is returned as `Some("tr")`. If absent, `None` is returned.
pub fn parse_with_lang_directive(source: &str) -> ParseResult<(Option<String>, Vec<Stmt>)> {
    // Extract language directive before any transformations
    let lang = crate::lang_directive::parse_lang_directive(source);

    // Strip the directive line so the PEG parser never sees it
    let source_without_directive = crate::lang_directive::strip_lang_directive(source);

    // Strip BiDi control characters for RTL languages
    let cleaned_source = crate::bidi::strip_bidi_controls(&source_without_directive);

    // Normalize multi-language keywords to canonical English before Pest parsing.
    // This eliminates the 22-language choice chains in the grammar.
    let normalized_source = normalize_keywords(&cleaned_source);

    let pairs = HudHudParser::parse(Rule::program, &normalized_source)
        .map_err(|e| parse_codes::invalid_syntax(e.to_string(), Default::default()))?;

    // Build the line-offset table once for O(1) line/col lookups in pair_to_span.
    // This eliminates the O(N²) cost of pest's line_col() rescanning.
    crate::parser::init_line_index(&normalized_source);

    let mut statements = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::program => {
                for inner_pair in pair.into_inner() {
                    if let Rule::statement = inner_pair.as_rule() {
                        if let Some(stmt) = parser::parse_statement(inner_pair)? {
                            statements.push(stmt);
                        }
                    }
                }
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    crate::parser::declarations::agent::EXTRA_DECLARATIONS.with(|extras| {
        statements.append(&mut extras.borrow_mut());
    });

    Ok((lang, statements))
}

/// Parse source code with error recovery, collecting multiple errors instead of
/// stopping at the first one.
///
/// When pest fails on a line, that line is recorded as an error and skipped,
/// then parsing resumes from the next line. Successfully parsed statements from
/// subsequent chunks are accumulated alongside the errors.
///
/// Returns `(statements, errors)`. If `errors` is empty the parse was clean.
pub fn parse_with_recovery(source: &str) -> (Vec<Stmt>, Vec<crate::error::ParseError>) {
    let mut all_stmts: Vec<Stmt> = Vec::new();
    let mut all_errors: Vec<crate::error::ParseError> = Vec::new();
    let mut remaining = source;
    let mut _line_offset: usize = 0;

    loop {
        if remaining.trim().is_empty() {
            break;
        }

        match parse(remaining) {
            Ok(stmts) => {
                all_stmts.extend(stmts);
                break;
            }
            Err(e) => {
                all_errors.push(e);
                // Skip the problematic line and retry the rest
                if let Some(newline_pos) = remaining.find('\n') {
                    _line_offset += 1;
                    remaining = &remaining[newline_pos + 1..];
                } else {
                    break; // No more lines to try
                }
            }
        }
    }

    (all_stmts, all_errors)
}
