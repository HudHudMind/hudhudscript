use crate::keyword_normalizer::core::{is_word_char, replace_keyword};
use crate::keyword_normalizer::map_ar::AR_KEYWORDS;
use crate::keyword_normalizer::map_bn::BN_KEYWORDS;
use crate::keyword_normalizer::map_common::COMMON_KEYWORDS;
use crate::keyword_normalizer::map_de::DE_KEYWORDS;
use crate::keyword_normalizer::map_el::EL_KEYWORDS;
use crate::keyword_normalizer::map_es::ES_KEYWORDS;
use crate::keyword_normalizer::map_fa::FA_KEYWORDS;
use crate::keyword_normalizer::map_fr::FR_KEYWORDS;
use crate::keyword_normalizer::map_hi::HI_KEYWORDS;
use crate::keyword_normalizer::map_hr::HR_KEYWORDS;
use crate::keyword_normalizer::map_id::ID_KEYWORDS;
use crate::keyword_normalizer::map_it::IT_KEYWORDS;
use crate::keyword_normalizer::map_ja::JA_KEYWORDS;
use crate::keyword_normalizer::map_ko::KO_KEYWORDS;
use crate::keyword_normalizer::map_ku::KU_KEYWORDS;
use crate::keyword_normalizer::map_multi::MULTI_KEYWORDS;
use crate::keyword_normalizer::map_pl::PL_KEYWORDS;
use crate::keyword_normalizer::map_pt::PT_KEYWORDS;
use crate::keyword_normalizer::map_ru::RU_KEYWORDS;
use crate::keyword_normalizer::map_sr::SR_KEYWORDS;
use crate::keyword_normalizer::map_th::TH_KEYWORDS;
use crate::keyword_normalizer::map_tr::TR_KEYWORDS;
use crate::keyword_normalizer::map_vi::VI_KEYWORDS;
use crate::keyword_normalizer::map_zh::ZH_KEYWORDS;
use rustc_hash::FxHashMap;

pub fn normalize_keywords(source: &str) -> String {
    // Phase 1: multi-word entries go through the legacy pass. Each one
    // may touch spaces/punctuation, so identifier-level lookup can't handle
    // them.  Order in `MULTI_KEYWORDS` is preserved (longest-first already).
    let after_multi: String = if MULTI_KEYWORDS.is_empty() {
        source.to_owned()
    } else {
        let mut current = source.to_owned();
        let mut buf = String::with_capacity(source.len());
        for kw in MULTI_KEYWORDS {
            buf.clear();
            replace_keyword(&current, kw.from, kw.to, &mut buf);
            if buf != current {
                std::mem::swap(&mut current, &mut buf);
            }
        }
        current
    };

    // Phase 2: single-word FxHashMap lookup in one O(n) scan.
    single_pass_normalize(&after_multi, single_word_map())
}

fn single_word_map() -> &'static FxHashMap<&'static str, &'static str> {
    static SINGLE: std::sync::OnceLock<FxHashMap<&'static str, &'static str>> =
        std::sync::OnceLock::new();
    SINGLE.get_or_init(|| {
        let mut map = FxHashMap::with_capacity_and_hasher(3000, Default::default());

        let insert_slice = |map: &mut FxHashMap<_, _>, slice: &[(&'static str, &'static str)]| {
            for &(from, to) in slice {
                map.insert(from, to);
            }
        };

        insert_slice(&mut map, &*COMMON_KEYWORDS);
        insert_slice(&mut map, TR_KEYWORDS);
        insert_slice(&mut map, AR_KEYWORDS);
        insert_slice(&mut map, JA_KEYWORDS);
        insert_slice(&mut map, RU_KEYWORDS);
        insert_slice(&mut map, ES_KEYWORDS);
        insert_slice(&mut map, DE_KEYWORDS);
        insert_slice(&mut map, FR_KEYWORDS);
        insert_slice(&mut map, ZH_KEYWORDS);
        insert_slice(&mut map, HI_KEYWORDS);
        insert_slice(&mut map, KO_KEYWORDS);
        insert_slice(&mut map, BN_KEYWORDS);
        insert_slice(&mut map, EL_KEYWORDS);
        insert_slice(&mut map, TH_KEYWORDS);
        insert_slice(&mut map, VI_KEYWORDS);
        insert_slice(&mut map, ID_KEYWORDS);
        insert_slice(&mut map, IT_KEYWORDS);
        insert_slice(&mut map, PL_KEYWORDS);
        insert_slice(&mut map, PT_KEYWORDS);
        insert_slice(&mut map, FA_KEYWORDS);
        insert_slice(&mut map, KU_KEYWORDS);
        insert_slice(&mut map, HR_KEYWORDS);
        insert_slice(&mut map, SR_KEYWORDS);

        map
    })
}

fn single_pass_normalize(source: &str, map: &FxHashMap<&'static str, &'static str>) -> String {
    let mut out = String::with_capacity(source.len());
    let src_bytes = source.as_bytes();
    let src_len = src_bytes.len();
    let mut i = 0;

    let mut in_string: Option<u8> = None;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while i < src_len {
        let b = src_bytes[i];

        // Newline ends line comments
        if b == b'\n' {
            in_line_comment = false;
            out.push('\n');
            i += 1;
            continue;
        }

        // Inside line comment
        if in_line_comment {
            let ch = source[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }

        // Inside block comment
        if in_block_comment {
            if b == b'*' && i + 1 < src_len && src_bytes[i + 1] == b'/' {
                out.push_str("*/");
                i += 2;
                in_block_comment = false;
            } else {
                let ch = source[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
            continue;
        }

        // Inside string literal
        if let Some(quote) = in_string {
            if b == b'\\' && i + 1 < src_len {
                out.push('\\');
                let next_ch = source[i + 1..].chars().next().unwrap();
                out.push(next_ch);
                i += 1 + next_ch.len_utf8();
            } else if b == quote {
                in_string = None;
                out.push(b as char);
                i += 1;
            } else {
                let ch = source[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
            continue;
        }

        // Comment start
        if b == b'/' && i + 1 < src_len {
            if src_bytes[i + 1] == b'/' {
                in_line_comment = true;
                out.push_str("//");
                i += 2;
                continue;
            }
            if src_bytes[i + 1] == b'*' {
                in_block_comment = true;
                out.push_str("/*");
                i += 2;
                continue;
            }
        }

        // String literal start
        if b == b'"' || b == b'\'' || b == b'`' {
            in_string = Some(b);
            out.push(b as char);
            i += 1;
            continue;
        }

        // Try to match an identifier at this position.
        if is_word_char(source[i..].chars().next().unwrap_or('\0')) {
            let word_start = i;
            while i < src_len && is_word_char(source[i..].chars().next().unwrap_or('\0')) {
                i += source[i..].chars().next().unwrap().len_utf8();
            }
            let word = &source[word_start..i];
            if let Some(&canonical) = map.get(word) {
                out.push_str(canonical);
            } else {
                out.push_str(word);
            }
            continue;
        }

        // Other character
        let ch = source[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }

    out
}
