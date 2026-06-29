use super::*;
use hudhudscript_lexer::is_hard_reserved_keyword;


pub(super) fn parse_number(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let num_str = pair.as_str();

    // Try Japanese numeral first
    if let Some(value) = japanese_numeral_to_number(num_str) {
        return Ok(Expr::Literal(Literal::Number(value, false), span));
    }

    // Convert Arabic-Indic digits to ASCII before parsing
    let num_str = arabic_to_ascii(num_str);
    let is_float = num_str.contains('.') || num_str.contains('e') || num_str.contains('E');
    let value = num_str
        .parse::<f64>()
        .map_err(|_| parse_codes::invalid_syntax("Invalid number", span))?;
    Ok(Expr::Literal(Literal::Number(value, is_float), span))
}

pub(super) fn parse_string(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let s = pair.as_str();
    let raw = &s[1..s.len() - 1]; // Remove surrounding quotes
    let mut value = String::with_capacity(raw.len());
    let mut chars = raw.chars();
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
    Ok(Expr::Literal(Literal::String(value), span))
}

pub(super) fn parse_boolean(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let value = matches!(pair.as_str(), "true" | "doğru" | "shin");
    Ok(Expr::Literal(Literal::Boolean(value), span))
}

pub(super) fn parse_identifier(pair: Pair<Rule>) -> ParseResult<Expr> {
    let span = pair_to_span(&pair);
    let ident = pair.as_str();

    // Check if it's a Japanese numeral (single character)
    if let Some(value) = japanese_numeral_to_number(ident) {
        return Ok(Expr::Literal(Literal::Number(value, false), span));
    }

    // this/self keywords in all supported languages → Expr::This
    let this_keywords = [
        "this",
        "self", // English
        "bu",
        "kendi", // Turkish
        "これ",
        "自分", // Japanese
        "هذا",
        "نفسه", // Arabic
        "এটি",
        "নিজে", // Bengali
        "это",
        "себя", // Russian
        "esto",
        "mismo", // Spanish
        "dies",
        "selbst", // German
        "ceci",
        "soi", // French
        "questo",
        "sé", // Italian
        "isto",
        "si", // Portuguese
        "这个",
        "自己", // Chinese
        "यह",
        "स्वयं", // Hindi
        "ini",
        "diri", // Indonesian
        "này",
        "bản_thân", // Vietnamese
        "αυτό",
        "εαυτός", // Greek
        "to",
        "siebie", // Polish
        "นี้",
        "ตัวเอง", // Thai
        "خود",   // Persian/Kurdish
    ];

    if this_keywords.contains(&ident) {
        return Ok(Expr::This(span));
    }

    if ident == "super" {
        return Ok(Expr::Identifier(ident.to_string(), span));
    }

    // Check if it's a reserved keyword using the lexer canonical table.
    if is_hard_reserved_keyword(ident) {
        return Err(parse_codes::invalid_syntax(
            format!(
                "'{}' is a reserved keyword and cannot be used as an identifier",
                ident
            ),
            span,
        ));
    }

    Ok(Expr::Identifier(ident.to_string(), span))
}

