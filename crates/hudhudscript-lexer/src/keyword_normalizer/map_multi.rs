use super::core::KwMap;

pub(crate) static MULTI_KEYWORDS: &[KwMap] = &[
    KwMap { from: "eşit değildir", to: "!=" },
    KwMap { from: "eşit değil", to: "!=" },
    KwMap {
        from: "değilse ama",
        to: "else if",
    },
    KwMap {
        from: "sino si",
        to: "else if",
    },
    KwMap {
        from: "иначе если",
        to: "else if",
    },
    KwMap {
        from: "وإلا إذا",
        to: "else if",
    },
    KwMap {
        from: "hiç biri değilse",
        to: "else",
    },
    KwMap {
        from: "hiçbiri değilse",
        to: "else",
    },
];
