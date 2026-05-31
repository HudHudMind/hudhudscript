mod keywords_early;
mod keywords_mid;
mod keywords_late;

use std::sync::LazyLock;

pub(crate) static COMMON_KEYWORDS: LazyLock<Vec<(&'static str, &'static str)>> =
    LazyLock::new(|| {
        let mut v = Vec::new();
        v.extend_from_slice(keywords_early::KEYWORDS);
        v.extend_from_slice(keywords_mid::KEYWORDS);
        v.extend_from_slice(keywords_late::KEYWORDS);
        v
    });
