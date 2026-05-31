#[inline]
pub fn is_ident_start(ch: char) -> bool {
    if (ch as u32) < 128 {
        matches!(ch, 'a'..='z' | 'A'..='Z' | '_')
    } else {
        ch.is_alphabetic()
    }
}

/// Identifier-continuation predicate with ASCII fast-path.
/// See `is_ident_start` for the rationale.
#[inline]
pub fn is_ident_continue(ch: char) -> bool {
    if (ch as u32) < 128 {
        matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
    } else {
        ch.is_alphanumeric()
    }
}
