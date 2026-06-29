//! Canonical reserved/keyword table (single source of truth).
//!
//! The parser imports this table from the lexer crate.  The normalizer maps
//! foreign-language keywords to the canonical English forms listed here;
//! the parser checks identifiers against the complete reserved set.
//!
//! Two sets are exported:
//! * `RESERVED_KEYWORDS` / `is_reserved_keyword` — full keyword set used by
//!   the normalizer and by diagnostic/lexer tests.
//! * `HARD_RESERVED_KEYWORDS` / `is_hard_reserved_keyword` — the smaller set
//!   of core language keywords that cannot be used as identifiers anywhere.
//!   Grammar-declared *contextual* keywords (governance, agent, DSL, etc.)
//!   are still reserved tokens but are allowed as identifiers when they
//!   appear in identifier positions.  This fixes the governance field-name
//!   bug while keeping keyword recognition intact in keyword positions.

use rustc_hash::FxHashSet;
use std::sync::LazyLock;

/// Core language keywords that cannot be used as identifiers.
static HARD_RESERVED_KEYWORDS: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    [
        // Logical operators (multilingual)
        "and", "or", "ve", "veya", "そして", "または",
        "و", "أو", "এবং", "অথবা", "и", "или",
        "und", "oder", "et", "ou", "और", "या",
        "dan", "atau", "và", "hoặc", "και", "ή",
        "lub", "และ", "หรือ", "ili", "یا", "yan", "û",
        // Language keywords (all supported languages)
        "let", "var", "const", "set",
        "if", "else", "while", "for", "in",
        "return", "break", "continue",
        "switch", "case", "default", "match", "enum",
        "function", "async", "await", "promise", "future",
        "class", "extends", "new", "super", "this", "self",
        "constructor", "static", "public", "private", "protected",
        "implements", "instanceof", "trait",
        "try", "catch", "finally", "throw",
        "import", "export", "use", "from", "as",
        "null",
        // Turkish core keywords
        "değişken", "tanım", "eğer", "değilse", "iken", "için", "içinde",
        "döndür", "sonlandır", "devam", "sınıf", "kalıtım", "yeni", "üst", "bu", "kendi",
        "yapıcı", "genel", "özel", "korumalı", "dene", "yakala", "sonunda", "fırlat",
        "içeAktar", "dışaAktar", "kullan", "den", "olarak",
        "işlev", "eşzamansız", "bekle",
    ]
    .into_iter()
    .collect()
});

/// Contextual keywords: reserved tokens in keyword positions but allowed as
/// identifiers in identifier positions (object field names, variable names,
/// property names, etc.).  This set must stay in sync with the grammar's
/// `_` keyword rules in `en.pest` and the other language pest files.
static CONTEXTUAL_KEYWORDS: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    [
        // Domain-specific agent / tool / resource / module keywords
        "agent", "tool", "resource", "mcp", "server",
        "config", "provider", "model",
        "state", "statemachine", "entity", "agentstate",
        "intent", "want", "spawn", "send", "receive", "perform", "require",
        "data", "governance",
        // Turkish equivalents of the domain-specific keywords above
        "ajan", "araç", "kaynak", "sunucu",
        "yapılandırma", "sağlayıcı", "model",
        "durum", "varlık", "niyet", "başlat", "gönder", "al", "gerçekleştir", "gerektir",
        "veri", "yönetişim",
        // Grammar-declared agent/governance keywords
        "action", "advisory", "collaborative", "communication_style", "community",
        "competitive", "consensus", "constitution", "contract", "council", "culture",
        "democracy", "effect", "enforcement", "event", "executor", "firstWins",
        "formal", "informal", "judge", "law", "mandatory", "member", "memory",
        "meritocracy", "monarchy", "oligarchy", "on", "onComplete", "onError",
        "onMemberComplete", "onMemberStart", "onStart", "onVote", "optional",
        "parliamentary", "parallel", "prosecutor", "protocol", "role", "roundRobin", "rule",
        "sequential", "store", "strategy", "subject", "swarm", "technical",
        "technocracy", "theocracy", "treaty", "trigger", "unanimous", "weighted", "when",
        "anarchy", "autocracy",
        // Domain DSL keywords (music, data-flow, network)
        "chord", "data_flow", "dataflow", "flow", "harmony", "layer", "melody",
        "network", "norms", "note", "rhythm", "scale", "state_machine", "tempo",
    ]
    .into_iter()
    .collect()
});

/// Complete reserved keyword set (hard + contextual).  This is the single
/// source of truth used by the keyword normalizer and lexer tests.
pub static RESERVED_KEYWORDS: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    let mut set = FxHashSet::default();
    set.extend(HARD_RESERVED_KEYWORDS.iter().copied());
    set.extend(CONTEXTUAL_KEYWORDS.iter().copied());
    set
});

/// Check whether `ident` is a reserved keyword (full set).
#[inline]
pub fn is_reserved_keyword(ident: &str) -> bool {
    RESERVED_KEYWORDS.contains(ident)
}

/// Check whether `ident` is a *hard* reserved keyword, i.e. cannot be used as
/// an identifier anywhere.
#[inline]
pub fn is_hard_reserved_keyword(ident: &str) -> bool {
    HARD_RESERVED_KEYWORDS.contains(ident)
}
