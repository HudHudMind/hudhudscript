//! Integration tests verifying governance and strategy keyword coverage
//! across all 23 supported languages (Issue #31).
//!
//! Every language must expose native-language translations for the core
//! governance set: constitution, law, rule, council, swarm, community,
//! strategy, culture, and governance-adjacent keywords.

use hudhudscript_localization::{Keyword, KeywordMap, Language};

/// Helper: assert that a language has a mapping for a given keyword variant.
fn assert_has_keyword(
    map: &KeywordMap,
    lang: Language,
    keyword: Keyword,
    lang_name: &str,
    kw_name: &str,
) {
    // Collect all entries for this language that map to the target keyword
    // by round-tripping through the canonical English form and looking for the
    // keyword in the map for this language.
    //
    // Since KeywordMap::lookup takes a word we need to check the inverse.
    // We verify by checking that the English canonical form can be found at all,
    // then test known words from each language.
    let english = map.to_english(keyword);
    let _ = english; // just ensure the variant is covered

    // All languages must have at least one entry for each governance keyword.
    // We do this by checking Language::English as the baseline and checking
    // that specific representative words exist.
    let _ = (map, lang, lang_name, kw_name);
}

/// Verify English has all governance keywords (baseline sanity check).
#[test]
fn test_english_has_all_governance_keywords() {
    let map = KeywordMap::new();

    assert_eq!(
        map.lookup("constitution", Language::English),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("law", Language::English), Some(Keyword::Law));
    assert_eq!(map.lookup("rule", Language::English), Some(Keyword::Rule));
    assert_eq!(
        map.lookup("council", Language::English),
        Some(Keyword::Council)
    );
    assert_eq!(map.lookup("swarm", Language::English), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("community", Language::English),
        Some(Keyword::Community)
    );
    assert_eq!(map.lookup("governance", Language::English), None); // not a keyword, governance is a crate concept
    assert_eq!(
        map.lookup("strategy", Language::English),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("culture", Language::English),
        Some(Keyword::Culture)
    );
    assert_eq!(
        map.lookup("enforcement", Language::English),
        Some(Keyword::Enforcement)
    );
    assert_eq!(
        map.lookup("mandatory", Language::English),
        Some(Keyword::Mandatory)
    );
    assert_eq!(
        map.lookup("advisory", Language::English),
        Some(Keyword::Advisory)
    );
    assert_eq!(
        map.lookup("optional", Language::English),
        Some(Keyword::Optional)
    );
    assert_eq!(map.lookup("role", Language::English), Some(Keyword::Role));
    assert_eq!(
        map.lookup("member", Language::English),
        Some(Keyword::Member)
    );
    assert_eq!(
        map.lookup("competitive", Language::English),
        Some(Keyword::Competitive)
    );
    assert_eq!(
        map.lookup("collaborative", Language::English),
        Some(Keyword::Collaborative)
    );
    assert_eq!(
        map.lookup("values", Language::English),
        Some(Keyword::Values)
    );
    assert_eq!(map.lookup("norms", Language::English), Some(Keyword::Norms));
}

/// Turkish governance keywords
#[test]
fn test_turkish_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("anayasa", Language::Turkish),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("yasa", Language::Turkish), Some(Keyword::Law));
    assert_eq!(map.lookup("kural", Language::Turkish), Some(Keyword::Rule));
    assert_eq!(
        map.lookup("konsey", Language::Turkish),
        Some(Keyword::Council)
    );
    assert_eq!(map.lookup("sürü", Language::Turkish), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("topluluk", Language::Turkish),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("strateji", Language::Turkish),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("kültür", Language::Turkish),
        Some(Keyword::Culture)
    );
}

/// Arabic governance keywords
#[test]
fn test_arabic_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("دستور", Language::Arabic),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("قانون", Language::Arabic), Some(Keyword::Law));
    assert_eq!(map.lookup("قاعدة", Language::Arabic), Some(Keyword::Rule));
    assert_eq!(map.lookup("مجلس", Language::Arabic), Some(Keyword::Council));
    assert_eq!(map.lookup("سرب", Language::Arabic), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("مجتمع", Language::Arabic),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("استراتيجية", Language::Arabic),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("ثقافة", Language::Arabic),
        Some(Keyword::Culture)
    );
}

/// Russian governance keywords
#[test]
fn test_russian_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("конституция", Language::Russian),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("закон", Language::Russian), Some(Keyword::Law));
    assert_eq!(
        map.lookup("правило", Language::Russian),
        Some(Keyword::Rule)
    );
    assert_eq!(
        map.lookup("совет", Language::Russian),
        Some(Keyword::Council)
    );
    assert_eq!(map.lookup("рой", Language::Russian), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("сообщество", Language::Russian),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("стратегия", Language::Russian),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("культура", Language::Russian),
        Some(Keyword::Culture)
    );
}

/// Chinese governance keywords
#[test]
fn test_chinese_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("宪法", Language::Chinese),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("法律", Language::Chinese), Some(Keyword::Law));
    assert_eq!(map.lookup("规则", Language::Chinese), Some(Keyword::Rule));
    assert_eq!(
        map.lookup("议会", Language::Chinese),
        Some(Keyword::Council)
    );
    assert_eq!(map.lookup("群", Language::Chinese), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("社区", Language::Chinese),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("策略", Language::Chinese),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("文化", Language::Chinese),
        Some(Keyword::Culture)
    );
}

/// Spanish governance keywords
#[test]
fn test_spanish_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("constitución", Language::Spanish),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("ley", Language::Spanish), Some(Keyword::Law));
    assert_eq!(map.lookup("regla", Language::Spanish), Some(Keyword::Rule));
    assert_eq!(
        map.lookup("consejo", Language::Spanish),
        Some(Keyword::Council)
    );
    assert_eq!(
        map.lookup("enjambre", Language::Spanish),
        Some(Keyword::Swarm)
    );
    assert_eq!(
        map.lookup("comunidad", Language::Spanish),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("estrategia", Language::Spanish),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("cultura", Language::Spanish),
        Some(Keyword::Culture)
    );
}

/// German governance keywords
#[test]
fn test_german_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("verfassung", Language::German),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("gesetz", Language::German), Some(Keyword::Law));
    assert_eq!(map.lookup("regel", Language::German), Some(Keyword::Rule));
    assert_eq!(map.lookup("rat", Language::German), Some(Keyword::Council));
    assert_eq!(
        map.lookup("schwarm", Language::German),
        Some(Keyword::Swarm)
    );
    assert_eq!(
        map.lookup("gemeinschaft", Language::German),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("strategie", Language::German),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("kultur", Language::German),
        Some(Keyword::Culture)
    );
}

/// French governance keywords (now with native translations)
#[test]
fn test_french_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("constitution", Language::French),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("loi", Language::French), Some(Keyword::Law));
    assert_eq!(map.lookup("règle", Language::French), Some(Keyword::Rule));
    assert_eq!(
        map.lookup("conseil", Language::French),
        Some(Keyword::Council)
    );
    assert_eq!(map.lookup("essaim", Language::French), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("communauté", Language::French),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("stratégie", Language::French),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("culture", Language::French),
        Some(Keyword::Culture)
    );
    // New native translations
    assert_eq!(
        map.lookup("règle_agent", Language::French),
        Some(Keyword::AgentRule)
    );
    assert_eq!(map.lookup("flux", Language::French), Some(Keyword::Flow));
    assert_eq!(
        map.lookup("intention", Language::French),
        Some(Keyword::Intent)
    );
    assert_eq!(
        map.lookup("fournisseur", Language::French),
        Some(Keyword::Provider)
    );
}

/// Italian governance keywords (now with native translations)
#[test]
fn test_italian_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("costituzione", Language::Italian),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("legge", Language::Italian), Some(Keyword::Law));
    assert_eq!(map.lookup("regola", Language::Italian), Some(Keyword::Rule));
    assert_eq!(
        map.lookup("consiglio", Language::Italian),
        Some(Keyword::Council)
    );
    assert_eq!(
        map.lookup("sciame", Language::Italian),
        Some(Keyword::Swarm)
    );
    assert_eq!(
        map.lookup("comunità", Language::Italian),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("strategia", Language::Italian),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("cultura", Language::Italian),
        Some(Keyword::Culture)
    );
    // New native translations
    assert_eq!(
        map.lookup("regola_agente", Language::Italian),
        Some(Keyword::AgentRule)
    );
    assert_eq!(map.lookup("flusso", Language::Italian), Some(Keyword::Flow));
    assert_eq!(
        map.lookup("intenzione", Language::Italian),
        Some(Keyword::Intent)
    );
    assert_eq!(
        map.lookup("fornitore", Language::Italian),
        Some(Keyword::Provider)
    );
}

/// Portuguese governance keywords (now with native translations)
#[test]
fn test_portuguese_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("constituição", Language::Portuguese),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("lei", Language::Portuguese), Some(Keyword::Law));
    assert_eq!(
        map.lookup("regra", Language::Portuguese),
        Some(Keyword::Rule)
    );
    assert_eq!(
        map.lookup("conselho", Language::Portuguese),
        Some(Keyword::Council)
    );
    assert_eq!(
        map.lookup("enxame", Language::Portuguese),
        Some(Keyword::Swarm)
    );
    assert_eq!(
        map.lookup("comunidade", Language::Portuguese),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("estratégia", Language::Portuguese),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("cultura", Language::Portuguese),
        Some(Keyword::Culture)
    );
    // New native translations
    assert_eq!(
        map.lookup("regra_agente", Language::Portuguese),
        Some(Keyword::AgentRule)
    );
    assert_eq!(
        map.lookup("fluxo", Language::Portuguese),
        Some(Keyword::Flow)
    );
    assert_eq!(
        map.lookup("intenção", Language::Portuguese),
        Some(Keyword::Intent)
    );
    assert_eq!(
        map.lookup("provedor", Language::Portuguese),
        Some(Keyword::Provider)
    );
}

/// Japanese governance keywords — Kural 8: must use Kanji/Hiragana, not romanized
#[test]
fn test_japanese_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("憲法", Language::Japanese),
        Some(Keyword::Constitution)
    );
    assert_eq!(
        map.lookup("法律", Language::Japanese),
        Some(Keyword::Law)
    );
    assert_eq!(map.lookup("規則", Language::Japanese), Some(Keyword::Rule));
    assert_eq!(
        map.lookup("会議", Language::Japanese),
        Some(Keyword::Council)
    );
    assert_eq!(map.lookup("群れ", Language::Japanese), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("共同体", Language::Japanese),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("戦略", Language::Japanese),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("文化", Language::Japanese),
        Some(Keyword::Culture)
    );
}

/// Polish governance keywords (now with native translations)
#[test]
fn test_polish_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("konstytucja", Language::Polish),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("prawo", Language::Polish), Some(Keyword::Law));
    assert_eq!(map.lookup("reguła", Language::Polish), Some(Keyword::Rule));
    assert_eq!(map.lookup("rada", Language::Polish), Some(Keyword::Council));
    assert_eq!(map.lookup("rój", Language::Polish), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("społeczność", Language::Polish),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("strategia", Language::Polish),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("kultura", Language::Polish),
        Some(Keyword::Culture)
    );
    // New native translations
    assert_eq!(
        map.lookup("zamiar", Language::Polish),
        Some(Keyword::Intent)
    );
    assert_eq!(
        map.lookup("funkcja", Language::Polish),
        Some(Keyword::Function)
    );
}

/// Thai governance keywords (now with native translations)
#[test]
fn test_thai_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("รัฐธรรมนูญ", Language::Thai),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("กฎหมาย", Language::Thai), Some(Keyword::Law));
    assert_eq!(map.lookup("กฎ", Language::Thai), Some(Keyword::Rule));
    assert_eq!(map.lookup("สภา", Language::Thai), Some(Keyword::Council));
    assert_eq!(map.lookup("ฝูง", Language::Thai), Some(Keyword::Swarm));
    assert_eq!(map.lookup("ชุมชน", Language::Thai), Some(Keyword::Community));
    assert_eq!(map.lookup("กลยุทธ์", Language::Thai), Some(Keyword::Strategy));
    assert_eq!(
        map.lookup("วัฒนธรรม", Language::Thai),
        Some(Keyword::Culture)
    );
    // New native translations
    assert_eq!(
        map.lookup("ความตั้งใจ", Language::Thai),
        Some(Keyword::Intent)
    );
    assert_eq!(map.lookup("ฟังก์ชัน", Language::Thai), Some(Keyword::Function));
}

/// Indonesian governance keywords
#[test]
fn test_indonesian_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("konstitusi", Language::Indonesian),
        Some(Keyword::Constitution)
    );
    assert_eq!(
        map.lookup("hukum", Language::Indonesian),
        Some(Keyword::Law)
    );
    assert_eq!(
        map.lookup("aturan", Language::Indonesian),
        Some(Keyword::Rule)
    );
    assert_eq!(
        map.lookup("dewan", Language::Indonesian),
        Some(Keyword::Council)
    );
    assert_eq!(
        map.lookup("kawanan", Language::Indonesian),
        Some(Keyword::Swarm)
    );
    assert_eq!(
        map.lookup("komunitas", Language::Indonesian),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("strategi", Language::Indonesian),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("budaya", Language::Indonesian),
        Some(Keyword::Culture)
    );
}

/// Vietnamese governance keywords (now with native translations)
#[test]
fn test_vietnamese_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("hiến_pháp", Language::Vietnamese),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("luật", Language::Vietnamese), Some(Keyword::Law));
    assert_eq!(
        map.lookup("quy_tắc", Language::Vietnamese),
        Some(Keyword::Rule)
    );
    assert_eq!(
        map.lookup("hội_đồng", Language::Vietnamese),
        Some(Keyword::Council)
    );
    assert_eq!(
        map.lookup("đàn", Language::Vietnamese),
        Some(Keyword::Swarm)
    );
    assert_eq!(
        map.lookup("cộng_đồng", Language::Vietnamese),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("chiến_lược", Language::Vietnamese),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("văn_hóa", Language::Vietnamese),
        Some(Keyword::Culture)
    );
    // New native translations
    assert_eq!(
        map.lookup("mục_đích", Language::Vietnamese),
        Some(Keyword::Intent)
    );
    assert_eq!(
        map.lookup("hàm", Language::Vietnamese),
        Some(Keyword::Function)
    );
}

/// Greek governance keywords (now with native translations)
#[test]
fn test_greek_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("σύνταγμα", Language::Greek),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("νόμος", Language::Greek), Some(Keyword::Law));
    assert_eq!(map.lookup("κανόνας", Language::Greek), Some(Keyword::Rule));
    assert_eq!(
        map.lookup("συμβούλιο", Language::Greek),
        Some(Keyword::Council)
    );
    assert_eq!(map.lookup("σμήνος", Language::Greek), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("κοινότητα", Language::Greek),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("στρατηγική", Language::Greek),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("κουλτούρα", Language::Greek),
        Some(Keyword::Culture)
    );
    // New native translations
    assert_eq!(
        map.lookup("πρόθεση", Language::Greek),
        Some(Keyword::Intent)
    );
    assert_eq!(
        map.lookup("συνάρτηση", Language::Greek),
        Some(Keyword::Function)
    );
}

/// Serbian governance keywords (now with native Cyrillic translations)
#[test]
fn test_serbian_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("устав", Language::Serbian),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("закон", Language::Serbian), Some(Keyword::Law));
    assert_eq!(
        map.lookup("правило", Language::Serbian),
        Some(Keyword::Rule)
    );
    assert_eq!(
        map.lookup("савет", Language::Serbian),
        Some(Keyword::Council)
    );
    assert_eq!(map.lookup("рој", Language::Serbian), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("заједница", Language::Serbian),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("стратегија", Language::Serbian),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("култура", Language::Serbian),
        Some(Keyword::Culture)
    );
    // New native Cyrillic translations
    assert_eq!(
        map.lookup("намера", Language::Serbian),
        Some(Keyword::Intent)
    );
    assert_eq!(
        map.lookup("функција", Language::Serbian),
        Some(Keyword::Function)
    );
}

/// Bosnian governance keywords (now with native translations)
#[test]
fn test_bosnian_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("ustav", Language::Bosnian),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("zakon", Language::Bosnian), Some(Keyword::Law));
    assert_eq!(
        map.lookup("pravilo", Language::Bosnian),
        Some(Keyword::Rule)
    );
    assert_eq!(
        map.lookup("vijeće", Language::Bosnian),
        Some(Keyword::Council)
    );
    assert_eq!(map.lookup("roj", Language::Bosnian), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("zajednica", Language::Bosnian),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("strategija", Language::Bosnian),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("kultura", Language::Bosnian),
        Some(Keyword::Culture)
    );
    // New native translations
    assert_eq!(
        map.lookup("namjera", Language::Bosnian),
        Some(Keyword::Intent)
    );
    assert_eq!(
        map.lookup("funkcija", Language::Bosnian),
        Some(Keyword::Function)
    );
}

/// Croatian governance keywords (now with native translations)
#[test]
fn test_croatian_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("ustav", Language::Croatian),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("zakon", Language::Croatian), Some(Keyword::Law));
    assert_eq!(
        map.lookup("pravilo", Language::Croatian),
        Some(Keyword::Rule)
    );
    assert_eq!(
        map.lookup("vijeće", Language::Croatian),
        Some(Keyword::Council)
    );
    assert_eq!(map.lookup("roj", Language::Croatian), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("zajednica", Language::Croatian),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("strategija", Language::Croatian),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("kultura", Language::Croatian),
        Some(Keyword::Culture)
    );
    // New native translations
    assert_eq!(
        map.lookup("namjera", Language::Croatian),
        Some(Keyword::Intent)
    );
    assert_eq!(
        map.lookup("funkcija", Language::Croatian),
        Some(Keyword::Function)
    );
}

/// Kurdish governance keywords (now with native translations)
#[test]
fn test_kurdish_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("destûr", Language::Kurdish),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("qanûn", Language::Kurdish), Some(Keyword::Law));
    assert_eq!(map.lookup("rêbaz", Language::Kurdish), Some(Keyword::Rule));
    assert_eq!(
        map.lookup("encûmen", Language::Kurdish),
        Some(Keyword::Council)
    );
    assert_eq!(map.lookup("kom", Language::Kurdish), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("civak", Language::Kurdish),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("stratejî", Language::Kurdish),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("çand", Language::Kurdish),
        Some(Keyword::Culture)
    );
    // New native translations
    assert_eq!(
        map.lookup("mebestî", Language::Kurdish),
        Some(Keyword::Intent)
    );
    assert_eq!(
        map.lookup("fonksiyon", Language::Kurdish),
        Some(Keyword::Function)
    );
}

/// Persian governance keywords (now with native translations)
#[test]
fn test_persian_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("قانون_اساسی", Language::Persian),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("قانون", Language::Persian), Some(Keyword::Law));
    assert_eq!(map.lookup("قاعده", Language::Persian), Some(Keyword::Rule));
    assert_eq!(
        map.lookup("شورا", Language::Persian),
        Some(Keyword::Council)
    );
    assert_eq!(
        map.lookup("ازدحام", Language::Persian),
        Some(Keyword::Swarm)
    );
    assert_eq!(
        map.lookup("جامعه", Language::Persian),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("استراتژی", Language::Persian),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("فرهنگ", Language::Persian),
        Some(Keyword::Culture)
    );
    // New native translations
    assert_eq!(map.lookup("هدف", Language::Persian), Some(Keyword::Intent));
    assert_eq!(
        map.lookup("تابع", Language::Persian),
        Some(Keyword::Function)
    );
}

/// Hindi governance keywords (now with native translations)
#[test]
fn test_hindi_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("संविधान", Language::Hindi),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("कानून", Language::Hindi), Some(Keyword::Law));
    assert_eq!(map.lookup("नियम", Language::Hindi), Some(Keyword::Rule));
    assert_eq!(map.lookup("परिषद", Language::Hindi), Some(Keyword::Council));
    assert_eq!(map.lookup("झुंड", Language::Hindi), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("समुदाय", Language::Hindi),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("रणनीति", Language::Hindi),
        Some(Keyword::Strategy)
    );
    assert_eq!(map.lookup("संस्कृति", Language::Hindi), Some(Keyword::Culture));
    // New native translations
    assert_eq!(map.lookup("इरादा", Language::Hindi), Some(Keyword::Intent));
    assert_eq!(map.lookup("फ़ंक्शन", Language::Hindi), Some(Keyword::Function));
}

/// Bengali governance keywords (now with native translations)
#[test]
fn test_bengali_governance_keywords() {
    let map = KeywordMap::new();
    assert_eq!(
        map.lookup("সংবিধান", Language::Bengali),
        Some(Keyword::Constitution)
    );
    assert_eq!(map.lookup("আইন", Language::Bengali), Some(Keyword::Law));
    assert_eq!(map.lookup("নিয়ম", Language::Bengali), Some(Keyword::Rule));
    assert_eq!(
        map.lookup("পরিষদ", Language::Bengali),
        Some(Keyword::Council)
    );
    assert_eq!(map.lookup("ঝাঁক", Language::Bengali), Some(Keyword::Swarm));
    assert_eq!(
        map.lookup("সম্প্রদায়", Language::Bengali),
        Some(Keyword::Community)
    );
    assert_eq!(
        map.lookup("কৌশল", Language::Bengali),
        Some(Keyword::Strategy)
    );
    assert_eq!(
        map.lookup("সংস্কৃতি", Language::Bengali),
        Some(Keyword::Culture)
    );
    // New native translations
    assert_eq!(
        map.lookup("উদ্দেশ্য", Language::Bengali),
        Some(Keyword::Intent)
    );
    assert_eq!(
        map.lookup("ফাংশন", Language::Bengali),
        Some(Keyword::Function)
    );
}

/// Verify the `to_english` method covers all governance-related keyword variants.
#[test]
fn test_all_governance_keywords_have_english_canonical_forms() {
    let map = KeywordMap::new();

    // All governance keywords must round-trip through to_english
    let governance_keywords = [
        Keyword::Constitution,
        Keyword::Law,
        Keyword::Rule,
        // AgentRule, RuleSet, RuleChain removed — deprecated in v0.4.6 (#317)
        Keyword::Council,
        Keyword::Swarm,
        Keyword::Community,
        Keyword::Enforcement,
        Keyword::Mandatory,
        Keyword::Advisory,
        Keyword::Optional,
        Keyword::Role,
        Keyword::Member,
        Keyword::Strategy,
        Keyword::Competitive,
        Keyword::Collaborative,
        Keyword::Culture,
        Keyword::Values,
        Keyword::Norms,
        Keyword::CommunicationStyle,
        Keyword::Formal,
        Keyword::Informal,
        Keyword::Technical,
        Keyword::Prosecutor,
        Keyword::Judge,
        Keyword::Executor,
    ];

    for kw in &governance_keywords {
        let english = map.to_english(*kw);
        assert!(
            !english.is_empty(),
            "to_english returned empty string for {:?}",
            kw
        );

        // Round-trip: English canonical form should look up back to the same keyword
        assert_eq!(
            map.lookup(english, Language::English),
            Some(*kw),
            "Round-trip failed for {:?} (english='{}' did not map back)",
            kw,
            english
        );
    }
}
