//! Public API tests for hudhudscript-localization

use hudhudscript_localization::{
    interpolate, pluralize, ArabicPlural, EnglishPlural, FallbackChain, Keyword, KeywordMap,
    Language, LanguageDetector, Locale, MessageCatalog, PluralCategory, PluralRule, RussianPlural,
    Translator, TurkishPlural, WordOrder,
};
use std::collections::HashMap;

// ── Language — codes ────────────────────────────────────────────────

#[test]
fn lang_code_english() {
    assert_eq!(Language::English.code(), "en");
}
#[test]
fn lang_code_turkish() {
    assert_eq!(Language::Turkish.code(), "tr");
}
#[test]
fn lang_code_japanese() {
    assert_eq!(Language::Japanese.code(), "ja");
}
#[test]
fn lang_code_arabic() {
    assert_eq!(Language::Arabic.code(), "ar");
}
#[test]
fn lang_code_russian() {
    assert_eq!(Language::Russian.code(), "ru");
}
#[test]
fn lang_code_chinese() {
    assert_eq!(Language::Chinese.code(), "zh");
}
#[test]
fn lang_code_spanish() {
    assert_eq!(Language::Spanish.code(), "es");
}
#[test]
fn lang_code_german() {
    assert_eq!(Language::German.code(), "de");
}
#[test]
fn lang_code_french() {
    assert_eq!(Language::French.code(), "fr");
}
#[test]
fn lang_code_italian() {
    assert_eq!(Language::Italian.code(), "it");
}
#[test]
fn lang_code_portuguese() {
    assert_eq!(Language::Portuguese.code(), "pt");
}
#[test]
fn lang_code_polish() {
    assert_eq!(Language::Polish.code(), "pl");
}
#[test]
fn lang_code_thai() {
    assert_eq!(Language::Thai.code(), "th");
}
#[test]
fn lang_code_indonesian() {
    assert_eq!(Language::Indonesian.code(), "id");
}
#[test]
fn lang_code_vietnamese() {
    assert_eq!(Language::Vietnamese.code(), "vi");
}
#[test]
fn lang_code_greek() {
    assert_eq!(Language::Greek.code(), "el");
}
#[test]
fn lang_code_serbian() {
    assert_eq!(Language::Serbian.code(), "sr");
}
#[test]
fn lang_code_bosnian() {
    assert_eq!(Language::Bosnian.code(), "bs");
}
#[test]
fn lang_code_croatian() {
    assert_eq!(Language::Croatian.code(), "hr");
}
#[test]
fn lang_code_kurdish() {
    assert_eq!(Language::Kurdish.code(), "ku");
}
#[test]
fn lang_code_persian() {
    assert_eq!(Language::Persian.code(), "fa");
}
#[test]
fn lang_code_hindi() {
    assert_eq!(Language::Hindi.code(), "hi");
}
#[test]
fn lang_code_bengali() {
    assert_eq!(Language::Bengali.code(), "bn");
}

// ── Language — RTL ──────────────────────────────────────────────────

#[test]
fn lang_rtl_arabic() {
    assert!(Language::Arabic.is_rtl());
}
#[test]
fn lang_rtl_persian() {
    assert!(Language::Persian.is_rtl());
}
#[test]
fn lang_rtl_kurdish() {
    assert!(Language::Kurdish.is_rtl());
}
#[test]
fn lang_rtl_english_false() {
    assert!(!Language::English.is_rtl());
}
#[test]
fn lang_rtl_turkish_false() {
    assert!(!Language::Turkish.is_rtl());
}
#[test]
fn lang_rtl_japanese_false() {
    assert!(!Language::Japanese.is_rtl());
}

// ── Language — detection ────────────────────────────────────────────

#[test]
fn detect_english_default() {
    assert_eq!(LanguageDetector::detect("let x = 42"), Language::English);
}
#[test]
fn detect_turkish() {
    assert_eq!(
        LanguageDetector::detect("postgres sunucusunu db olarak kullan"),
        Language::Turkish
    );
}
#[test]
fn detect_japanese() {
    assert_eq!(
        LanguageDetector::detect("postgres を db として つかう"),
        Language::Japanese
    );
}
#[test]
fn detect_arabic() {
    assert_eq!(
        LanguageDetector::detect("استخدم قاعدة البيانات"),
        Language::Arabic
    );
}
#[test]
fn detect_persian() {
    assert_eq!(LanguageDetector::detect("پ چ ژ گ"), Language::Persian);
}
#[test]
fn detect_hindi() {
    assert_eq!(LanguageDetector::detect("उपयोग_करो"), Language::Hindi);
}
#[test]
fn detect_bengali() {
    assert_eq!(LanguageDetector::detect("ব্যবহার_করো"), Language::Bengali);
}
#[test]
fn detect_serbian_cyrillic() {
    assert_eq!(LanguageDetector::detect("ђ ћ џ"), Language::Serbian);
}
#[test]
fn detect_russian() {
    assert_eq!(
        LanguageDetector::detect("использовать базу"),
        Language::Russian
    );
}
#[test]
fn detect_chinese() {
    assert_eq!(LanguageDetector::detect("使用 数据库"), Language::Chinese);
}
#[test]
fn detect_german() {
    assert_eq!(
        LanguageDetector::detect("verwenden datenbank"),
        Language::German
    );
}
#[test]
fn detect_french() {
    assert_eq!(LanguageDetector::detect("utiliser base"), Language::French);
}
#[test]
fn detect_portuguese() {
    assert_eq!(
        LanguageDetector::detect("usar banco então"),
        Language::Portuguese
    );
}
#[test]
fn detect_spanish() {
    assert_eq!(
        LanguageDetector::detect("si entonces usar"),
        Language::Spanish
    );
}
#[test]
fn detect_kurdish() {
    assert_eq!(
        LanguageDetector::detect("bikar_bîne navnîşan"),
        Language::Kurdish
    );
}
#[test]
fn detect_supported_count_23() {
    // Updated: Korean (ko) was added as the 24th language
    assert_eq!(LanguageDetector::supported_languages().len(), 24);
}

// ── KeywordMap — lookup basic ───────────────────────────────────────

#[test]
fn kw_lookup_english_use() {
    let map = KeywordMap::new();
    assert_eq!(map.lookup("use", Language::English), Some(Keyword::Use));
}
#[test]
fn kw_lookup_english_agent() {
    let map = KeywordMap::new();
    assert_eq!(map.lookup("agent", Language::English), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_english_if() {
    let map = KeywordMap::new();
    assert_eq!(map.lookup("if", Language::English), Some(Keyword::If));
}
#[test]
fn kw_lookup_turkish_use() {
    let map = KeywordMap::new();
    assert_eq!(map.lookup("kullan", Language::Turkish), Some(Keyword::Use));
}
#[test]
fn kw_lookup_turkish_agent() {
    let map = KeywordMap::new();
    assert_eq!(map.lookup("ajan", Language::Turkish), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_japanese_use() {
    let map = KeywordMap::new();
    // Kural 8: Kanji/Hiragana required, not romanized
    assert_eq!(map.lookup("使う", Language::Japanese), Some(Keyword::Use));
}
#[test]
fn kw_lookup_missing_returns_none() {
    let map = KeywordMap::new();
    assert_eq!(map.lookup("nonexistent_xyz", Language::English), None);
}
#[test]
fn kw_default_identical_to_new() {
    let map = KeywordMap::default();
    assert_eq!(map.lookup("use", Language::English), Some(Keyword::Use));
}

// ── KeywordMap — to_english coverage ────────────────────────────────

#[test]
fn kw_to_english_use() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Use), "use");
}
#[test]
fn kw_to_english_as() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::As), "as");
}
#[test]
fn kw_to_english_agent() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Agent), "agent");
}
#[test]
fn kw_to_english_if() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::If), "if");
}
#[test]
fn kw_to_english_else() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Else), "else");
}
#[test]
fn kw_to_english_while() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::While), "while");
}
#[test]
fn kw_to_english_return() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Return), "return");
}
#[test]
fn kw_to_english_true_false() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::True), "true");
    assert_eq!(m.to_english(Keyword::False), "false");
}
#[test]
fn kw_to_english_class() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Class), "class");
}
#[test]
fn kw_to_english_async_await() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Async), "async");
    assert_eq!(m.to_english(Keyword::Await), "await");
}

// ── MessageCatalog — JSON ───────────────────────────────────────────

#[test]
fn catalog_from_json_str() {
    let json = r#"{"_locale": "tr-TR", "greeting": "Merhaba", "farewell": "Hoşça kal"}"#;
    let cat = MessageCatalog::from_json_str(json).unwrap();
    assert_eq!(cat.locale(), "tr-TR");
    assert_eq!(cat.get("greeting"), Some("Merhaba"));
    assert_eq!(cat.get("farewell"), Some("Hoşça kal"));
    assert_eq!(cat.len(), 2);
}
#[test]
fn catalog_json_skips_metadata() {
    let json = r#"{"_locale":"en","_ver":"1","hi":"Hello"}"#;
    let cat = MessageCatalog::from_json_str(json).unwrap();
    assert_eq!(cat.get("_ver"), None);
    assert_eq!(cat.get("hi"), Some("Hello"));
    assert_eq!(cat.len(), 1);
}
#[test]
fn catalog_json_no_locale() {
    let cat = MessageCatalog::from_json_str(r#"{"k":"v"}"#).unwrap();
    assert_eq!(cat.locale(), "");
}
#[test]
fn catalog_json_skips_non_string_values() {
    let cat = MessageCatalog::from_json_str(r#"{"n":42,"s":"ok"}"#).unwrap();
    assert_eq!(cat.get("n"), None);
    assert_eq!(cat.get("s"), Some("ok"));
}
#[test]
fn catalog_json_invalid() {
    assert!(MessageCatalog::from_json_str("not json{{{").is_err());
}

// ── MessageCatalog — YAML ───────────────────────────────────────────

#[test]
fn catalog_from_yaml_str() {
    let yaml = "_locale: en\ngreeting: Hello\nfarewell: Goodbye\n";
    let cat = MessageCatalog::from_yaml_str(yaml).unwrap();
    assert_eq!(cat.locale(), "en");
    assert_eq!(cat.get("greeting"), Some("Hello"));
    assert_eq!(cat.len(), 2);
}
#[test]
fn catalog_yaml_skips_metadata() {
    let yaml = "_locale: tr\n_meta: x\nk: v\n";
    let cat = MessageCatalog::from_yaml_str(yaml).unwrap();
    assert_eq!(cat.get("_meta"), None);
    assert_eq!(cat.get("k"), Some("v"));
}
#[test]
fn catalog_yaml_invalid() {
    assert!(MessageCatalog::from_yaml_str(":\n  :\n    : [invalid").is_err());
}

// ── MessageCatalog — fallback ───────────────────────────────────────

#[test]
fn catalog_fallback_primary_hit() {
    let primary = MessageCatalog::from_map(
        "tr",
        [("hello".into(), "Merhaba".into())].into_iter().collect(),
    );
    let fb = MessageCatalog::from_map(
        "en",
        [("hello".into(), "Hello".into())].into_iter().collect(),
    );
    assert_eq!(primary.get_with_fallback("hello", &[fb]), "Merhaba");
}
#[test]
fn catalog_fallback_to_secondary() {
    let primary = MessageCatalog::new("tr");
    let fb = MessageCatalog::from_map(
        "en",
        [("bye".into(), "Goodbye".into())].into_iter().collect(),
    );
    assert_eq!(primary.get_with_fallback("bye", &[fb]), "Goodbye");
}
#[test]
fn catalog_fallback_returns_key_if_missing_everywhere() {
    let primary = MessageCatalog::new("tr");
    let fb = MessageCatalog::new("en");
    assert_eq!(primary.get_with_fallback("nope", &[fb]), "nope");
}
#[test]
fn catalog_fallback_chain_multi() {
    let p = MessageCatalog::new("ja");
    let fb1 = MessageCatalog::new("zh");
    let fb2 = MessageCatalog::from_map("en", [("k".into(), "val".into())].into_iter().collect());
    assert_eq!(p.get_with_fallback("k", &[fb1, fb2]), "val");
}
#[test]
fn catalog_new_is_empty() {
    let c = MessageCatalog::new("en");
    assert!(c.is_empty());
    assert_eq!(c.len(), 0);
}
#[test]
fn catalog_insert() {
    let mut c = MessageCatalog::new("en");
    c.insert("k", "v");
    assert_eq!(c.get("k"), Some("v"));
    assert_eq!(c.len(), 1);
}

// ── Plural rules — English ──────────────────────────────────────────

#[test]
fn plural_english_one() {
    assert_eq!(EnglishPlural.select(1.0), PluralCategory::One);
}
#[test]
fn plural_english_zero() {
    assert_eq!(EnglishPlural.select(0.0), PluralCategory::Other);
}
#[test]
fn plural_english_many() {
    assert_eq!(EnglishPlural.select(5.0), PluralCategory::Other);
}

// ── Plural rules — Arabic ───────────────────────────────────────────

#[test]
fn plural_arabic_zero() {
    assert_eq!(ArabicPlural.select(0.0), PluralCategory::Zero);
}
#[test]
fn plural_arabic_one() {
    assert_eq!(ArabicPlural.select(1.0), PluralCategory::One);
}
#[test]
fn plural_arabic_two() {
    assert_eq!(ArabicPlural.select(2.0), PluralCategory::Two);
}
#[test]
fn plural_arabic_few() {
    assert_eq!(ArabicPlural.select(5.0), PluralCategory::Few);
}
#[test]
fn plural_arabic_many() {
    assert_eq!(ArabicPlural.select(11.0), PluralCategory::Many);
}
#[test]
fn plural_arabic_other() {
    assert_eq!(ArabicPlural.select(100.0), PluralCategory::Other);
}
#[test]
fn plural_arabic_103_few() {
    assert_eq!(ArabicPlural.select(103.0), PluralCategory::Few);
}
#[test]
fn plural_arabic_111_many() {
    assert_eq!(ArabicPlural.select(111.0), PluralCategory::Many);
}

// ── Plural rules — Russian ──────────────────────────────────────────

#[test]
fn plural_russian_1_one() {
    assert_eq!(RussianPlural.select(1.0), PluralCategory::One);
}
#[test]
fn plural_russian_2_few() {
    assert_eq!(RussianPlural.select(2.0), PluralCategory::Few);
}
#[test]
fn plural_russian_5_many() {
    assert_eq!(RussianPlural.select(5.0), PluralCategory::Many);
}
#[test]
fn plural_russian_11_many() {
    assert_eq!(RussianPlural.select(11.0), PluralCategory::Many);
}
#[test]
fn plural_russian_21_one() {
    assert_eq!(RussianPlural.select(21.0), PluralCategory::One);
}
#[test]
fn plural_russian_22_few() {
    assert_eq!(RussianPlural.select(22.0), PluralCategory::Few);
}
#[test]
fn plural_russian_0_many() {
    assert_eq!(RussianPlural.select(0.0), PluralCategory::Many);
}
#[test]
fn plural_russian_111_many() {
    assert_eq!(RussianPlural.select(111.0), PluralCategory::Many);
}

// ── Plural rules — Turkish ──────────────────────────────────────────

#[test]
fn plural_turkish_one() {
    assert_eq!(TurkishPlural.select(1.0), PluralCategory::One);
}
#[test]
fn plural_turkish_other() {
    assert_eq!(TurkishPlural.select(0.0), PluralCategory::Other);
    assert_eq!(TurkishPlural.select(42.0), PluralCategory::Other);
}

// ── pluralize function ──────────────────────────────────────────────

#[test]
fn pluralize_english_one_item() {
    let mut msgs = HashMap::new();
    msgs.insert("items.one".into(), "# item".into());
    msgs.insert("items.other".into(), "# items".into());
    assert_eq!(pluralize("items", 1.0, &EnglishPlural, &msgs), "1 item");
}
#[test]
fn pluralize_english_five_items() {
    let mut msgs = HashMap::new();
    msgs.insert("items.one".into(), "# item".into());
    msgs.insert("items.other".into(), "# items".into());
    assert_eq!(pluralize("items", 5.0, &EnglishPlural, &msgs), "5 items");
}
#[test]
fn pluralize_fallback_to_other() {
    let mut msgs = HashMap::new();
    msgs.insert("f.other".into(), "# files".into());
    assert_eq!(pluralize("f", 0.0, &ArabicPlural, &msgs), "0 files");
}
#[test]
fn pluralize_missing_key_returns_key() {
    let msgs = HashMap::new();
    assert_eq!(pluralize("missing", 1.0, &EnglishPlural, &msgs), "missing");
}
#[test]
fn pluralize_fractional_count() {
    let mut msgs = HashMap::new();
    msgs.insert("items.other".into(), "# items".into());
    assert_eq!(pluralize("items", 2.5, &EnglishPlural, &msgs), "2.5 items");
}
#[test]
fn pluralize_arabic_all_categories() {
    let mut msgs = HashMap::new();
    msgs.insert("i.zero".into(), "no items".into());
    msgs.insert("i.one".into(), "# item".into());
    msgs.insert("i.two".into(), "# dual".into());
    msgs.insert("i.few".into(), "# few".into());
    msgs.insert("i.many".into(), "# many".into());
    msgs.insert("i.other".into(), "# other".into());
    assert_eq!(pluralize("i", 0.0, &ArabicPlural, &msgs), "no items");
    assert_eq!(pluralize("i", 1.0, &ArabicPlural, &msgs), "1 item");
    assert_eq!(pluralize("i", 2.0, &ArabicPlural, &msgs), "2 dual");
    assert_eq!(pluralize("i", 5.0, &ArabicPlural, &msgs), "5 few");
    assert_eq!(pluralize("i", 11.0, &ArabicPlural, &msgs), "11 many");
    assert_eq!(pluralize("i", 100.0, &ArabicPlural, &msgs), "100 other");
}

// ── interpolation ───────────────────────────────────────────────────

#[test]
fn interpolate_simple_variable() {
    let mut vars = HashMap::new();
    vars.insert("name".into(), "World".into());
    assert_eq!(interpolate("Hello, {name}!", &vars), "Hello, World!");
}
#[test]
fn interpolate_multiple_variables() {
    let mut vars = HashMap::new();
    vars.insert("first".into(), "A".into());
    vars.insert("last".into(), "B".into());
    assert_eq!(interpolate("{first} {last}", &vars), "A B");
}
#[test]
fn interpolate_unknown_preserved() {
    let vars = HashMap::new();
    assert_eq!(interpolate("{x}", &vars), "{x}");
}
#[test]
fn interpolate_no_placeholders() {
    let vars = HashMap::new();
    assert_eq!(interpolate("plain", &vars), "plain");
}
#[test]
fn interpolate_empty_template() {
    let vars = HashMap::new();
    assert_eq!(interpolate("", &vars), "");
}
#[test]
fn interpolate_consecutive() {
    let mut vars = HashMap::new();
    vars.insert("a".into(), "X".into());
    vars.insert("b".into(), "Y".into());
    assert_eq!(interpolate("{a}{b}", &vars), "XY");
}
#[test]
fn interpolate_unmatched_brace() {
    let vars = HashMap::new();
    assert_eq!(interpolate("{ world", &vars), "{ world");
}
#[test]
fn interpolate_plural_one() {
    let mut vars = HashMap::new();
    vars.insert("count".into(), "1".into());
    assert_eq!(
        interpolate("{count, plural, one{# item} other{# items}}", &vars),
        "1 item"
    );
}
#[test]
fn interpolate_plural_other() {
    let mut vars = HashMap::new();
    vars.insert("count".into(), "5".into());
    assert_eq!(
        interpolate("{count, plural, one{# item} other{# items}}", &vars),
        "5 items"
    );
}
#[test]
fn interpolate_plural_zero() {
    let mut vars = HashMap::new();
    vars.insert("count".into(), "0".into());
    assert_eq!(
        interpolate(
            "{count, plural, zero{none} one{# item} other{# items}}",
            &vars
        ),
        "none"
    );
}
#[test]
fn interpolate_mixed() {
    let mut vars = HashMap::new();
    vars.insert("user".into(), "Alice".into());
    vars.insert("count".into(), "3".into());
    let t = "{user} has {count, plural, one{# msg} other{# msgs}}";
    assert_eq!(interpolate(t, &vars), "Alice has 3 msgs");
}
#[test]
fn interpolate_non_plural_kind_preserved() {
    let mut vars = HashMap::new();
    vars.insert("x".into(), "5".into());
    let t = "{x, select, one{un} other{des}}";
    assert_eq!(interpolate(t, &vars), t);
}

// ── Locale parsing ──────────────────────────────────────────────────

#[test]
fn locale_parse_simple() {
    let l = Locale::parse("en");
    assert_eq!(l.language, "en");
    assert_eq!(l.region, None);
    assert_eq!(l.script, None);
}
#[test]
fn locale_parse_with_region() {
    let l = Locale::parse("tr-TR");
    assert_eq!(l.language, "tr");
    assert_eq!(l.region, Some("TR".into()));
}
#[test]
fn locale_parse_with_script_and_region() {
    let l = Locale::parse("zh-Hans-CN");
    assert_eq!(l.language, "zh");
    assert_eq!(l.script, Some("Hans".into()));
    assert_eq!(l.region, Some("CN".into()));
}
#[test]
fn locale_parse_underscore() {
    let l = Locale::parse("en_US");
    assert_eq!(l.language, "en");
    assert_eq!(l.region, Some("US".into()));
}
#[test]
fn locale_parse_encoding_suffix() {
    let l = Locale::parse("tr_TR.UTF-8");
    assert_eq!(l.language, "tr");
    assert_eq!(l.region, Some("TR".into()));
}
#[test]
fn locale_to_tag() {
    assert_eq!(Locale::parse("tr-TR").to_tag(), "tr-TR");
    assert_eq!(Locale::parse("zh-Hans-CN").to_tag(), "zh-Hans-CN");
    assert_eq!(Locale::parse("en").to_tag(), "en");
}
#[test]
fn locale_display() {
    let l = Locale::parse("en-US");
    assert_eq!(format!("{}", l), "en-US");
}
#[test]
fn locale_eq_and_hash() {
    let l1 = Locale::parse("en-US");
    let l2 = Locale::parse("en-US");
    let l3 = Locale::parse("en-GB");
    assert_eq!(l1, l2);
    assert_ne!(l1, l3);
    let mut set = std::collections::HashSet::new();
    set.insert(l1.clone());
    assert!(set.contains(&l2));
}
#[test]
fn locale_parse_script_only() {
    let l = Locale::parse("zh-Hans");
    assert_eq!(l.script, Some("Hans".into()));
    assert_eq!(l.region, None);
}

// ── FallbackChain ───────────────────────────────────────────────────

#[test]
fn fallback_chain_basic() {
    let chain = FallbackChain::new(&Locale::parse("tr-TR"));
    let tags: Vec<&str> = chain.iter().collect();
    assert_eq!(tags, vec!["tr-TR", "tr", "en"]);
}
#[test]
fn fallback_chain_english_no_duplicate() {
    let chain = FallbackChain::new(&Locale::parse("en"));
    let tags: Vec<&str> = chain.iter().collect();
    assert_eq!(tags, vec!["en"]);
}
#[test]
fn fallback_chain_with_script() {
    let chain = FallbackChain::new(&Locale::parse("zh-Hans-CN"));
    let tags: Vec<&str> = chain.iter().collect();
    assert_eq!(tags, vec!["zh-Hans-CN", "zh-Hans", "zh", "en"]);
}
#[test]
fn fallback_chain_custom() {
    let chain = FallbackChain::from_tags(vec!["bs".into(), "hr".into(), "sr".into(), "en".into()]);
    assert_eq!(chain.tags().len(), 4);
    let tags: Vec<&str> = chain.iter().collect();
    assert_eq!(tags[0], "bs");
}
#[test]
fn fallback_chain_tags_accessor() {
    let chain = FallbackChain::new(&Locale::parse("fr"));
    assert_eq!(chain.tags(), &["fr".to_string(), "en".to_string()]);
}

// ── Translator ──────────────────────────────────────────────────────

#[test]
fn translator_english_passthrough() {
    let t = Translator::new();
    let s = "use postgres as db";
    assert_eq!(t.translate_to_canonical(s), s);
}
#[test]
fn translator_turkish_keywords() {
    let t = Translator::new();
    let result = t.translate_to_canonical("postgres kullan db olarak");
    assert!(result.contains("use"));
    assert!(result.contains("as"));
}
#[test]
fn translator_detect_language() {
    let t = Translator::new();
    assert_eq!(t.detect_language("use postgres"), Language::English);
    assert_eq!(t.detect_language("kullan postgres"), Language::Turkish);
}
#[test]
fn translator_default_trait() {
    let t = Translator::default();
    assert_eq!(t.detect_language("use x"), Language::English);
}
#[test]
fn translator_preserves_punctuation() {
    let t = Translator::new();
    let result = t.translate_to_canonical("eğer(x) ise");
    assert!(result.contains("if"));
    assert!(result.contains("("));
}
#[test]
fn translator_unknown_words_preserved() {
    let t = Translator::new();
    let result = t.translate_to_canonical("kullan foobarbaz olarak db");
    assert!(result.contains("foobarbaz"));
    assert!(result.contains("use"));
}
#[test]
fn translator_last_word_handled() {
    let t = Translator::new();
    let result = t.translate_to_canonical("postgres kullan");
    assert!(result.contains("use"));
}

// ── Language — name ──────────────────────────────────────────────────

#[test]
fn lang_name_english() {
    assert_eq!(Language::English.name(), "English");
}
#[test]
fn lang_name_turkish() {
    assert_eq!(Language::Turkish.name(), "Türkçe");
}
#[test]
fn lang_name_japanese() {
    assert_eq!(Language::Japanese.name(), "日本語");
}
#[test]
fn lang_name_arabic() {
    assert_eq!(Language::Arabic.name(), "العربية");
}
#[test]
fn lang_name_russian() {
    assert_eq!(Language::Russian.name(), "Русский");
}
#[test]
fn lang_name_chinese() {
    assert_eq!(Language::Chinese.name(), "中文");
}
#[test]
fn lang_name_spanish() {
    assert_eq!(Language::Spanish.name(), "Español");
}
#[test]
fn lang_name_german() {
    assert_eq!(Language::German.name(), "Deutsch");
}
#[test]
fn lang_name_french() {
    assert_eq!(Language::French.name(), "Français");
}
#[test]
fn lang_name_italian() {
    assert_eq!(Language::Italian.name(), "Italiano");
}
#[test]
fn lang_name_portuguese() {
    assert_eq!(Language::Portuguese.name(), "Português");
}
#[test]
fn lang_name_polish() {
    assert_eq!(Language::Polish.name(), "Polski");
}
#[test]
fn lang_name_thai() {
    assert_eq!(Language::Thai.name(), "ไทย");
}
#[test]
fn lang_name_indonesian() {
    assert_eq!(Language::Indonesian.name(), "Bahasa Indonesia");
}
#[test]
fn lang_name_vietnamese() {
    assert_eq!(Language::Vietnamese.name(), "Tiếng Việt");
}
#[test]
fn lang_name_greek() {
    assert_eq!(Language::Greek.name(), "Ελληνικά");
}
#[test]
fn lang_name_serbian() {
    assert_eq!(Language::Serbian.name(), "Српски");
}
#[test]
fn lang_name_bosnian() {
    assert_eq!(Language::Bosnian.name(), "Bosanski");
}
#[test]
fn lang_name_croatian() {
    assert_eq!(Language::Croatian.name(), "Hrvatski");
}
#[test]
fn lang_name_kurdish() {
    assert_eq!(Language::Kurdish.name(), "Kurdî");
}
#[test]
fn lang_name_persian() {
    assert_eq!(Language::Persian.name(), "فارسی");
}
#[test]
fn lang_name_hindi() {
    assert_eq!(Language::Hindi.name(), "हिन्दी");
}
#[test]
fn lang_name_bengali() {
    assert_eq!(Language::Bengali.name(), "বাংলা");
}

// ── Language — word_order ────────────────────────────────────────────

#[test]
fn lang_word_order_english_svo() {
    assert_eq!(Language::English.word_order(), WordOrder::SVO);
}
#[test]
fn lang_word_order_turkish_sov() {
    assert_eq!(Language::Turkish.word_order(), WordOrder::SOV);
}
#[test]
fn lang_word_order_arabic_vso() {
    assert_eq!(Language::Arabic.word_order(), WordOrder::VSO);
}
#[test]
fn lang_word_order_german_v2() {
    assert_eq!(Language::German.word_order(), WordOrder::V2);
}
#[test]
fn lang_word_order_japanese_sov() {
    assert_eq!(Language::Japanese.word_order(), WordOrder::SOV);
}
#[test]
fn lang_word_order_persian_sov() {
    assert_eq!(Language::Persian.word_order(), WordOrder::SOV);
}
#[test]
fn lang_word_order_hindi_sov() {
    assert_eq!(Language::Hindi.word_order(), WordOrder::SOV);
}
#[test]
fn lang_word_order_bengali_sov() {
    assert_eq!(Language::Bengali.word_order(), WordOrder::SOV);
}
#[test]
fn lang_word_order_kurdish_sov() {
    assert_eq!(Language::Kurdish.word_order(), WordOrder::SOV);
}
#[test]
fn lang_word_order_french_svo() {
    assert_eq!(Language::French.word_order(), WordOrder::SVO);
}
#[test]
fn lang_word_order_russian_svo() {
    assert_eq!(Language::Russian.word_order(), WordOrder::SVO);
}
#[test]
fn lang_word_order_chinese_svo() {
    assert_eq!(Language::Chinese.word_order(), WordOrder::SVO);
}

// ── Language — uses_particles / is_agglutinative ─────────────────────

#[test]
fn lang_uses_particles_japanese() {
    assert!(Language::Japanese.uses_particles());
}
#[test]
fn lang_uses_particles_chinese() {
    assert!(Language::Chinese.uses_particles());
}
#[test]
fn lang_uses_particles_english_false() {
    assert!(!Language::English.uses_particles());
}
#[test]
fn lang_uses_particles_turkish_false() {
    assert!(!Language::Turkish.uses_particles());
}
#[test]
fn lang_uses_particles_arabic_false() {
    assert!(!Language::Arabic.uses_particles());
}
#[test]
fn lang_is_agglutinative_turkish() {
    assert!(Language::Turkish.is_agglutinative());
}
#[test]
fn lang_is_agglutinative_japanese() {
    assert!(Language::Japanese.is_agglutinative());
}
#[test]
fn lang_is_agglutinative_english_false() {
    assert!(!Language::English.is_agglutinative());
}
#[test]
fn lang_is_agglutinative_chinese_false() {
    assert!(!Language::Chinese.is_agglutinative());
}

// ── Language — additional detection ──────────────────────────────────

#[test]
fn detect_italian() {
    assert_eq!(
        LanguageDetector::detect("agente se allora usare"),
        Language::Italian
    );
}
#[test]
fn detect_supported_contains_all() {
    let langs = LanguageDetector::supported_languages();
    assert!(langs.contains(&Language::English));
    assert!(langs.contains(&Language::Turkish));
    assert!(langs.contains(&Language::Arabic));
    assert!(langs.contains(&Language::Bengali));
    assert!(langs.contains(&Language::Persian));
    assert!(langs.contains(&Language::Kurdish));
}

// ── KeywordMap — language-specific lookups ───────────────────────────

#[test]
fn kw_lookup_turkish_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("eğer", Language::Turkish), Some(Keyword::If));
}
#[test]
fn kw_lookup_japanese_agent() {
    let m = KeywordMap::new();
    // Kural 8: Katakana required, not romanized
    assert_eq!(
        m.lookup("エージェント", Language::Japanese),
        Some(Keyword::Agent)
    );
}
#[test]
fn kw_lookup_japanese_if() {
    let m = KeywordMap::new();
    // Kural 8: Hiragana required, not romanized
    assert_eq!(m.lookup("もし", Language::Japanese), Some(Keyword::If));
}
#[test]
fn kw_lookup_arabic_use() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("استخدم", Language::Arabic), Some(Keyword::Use));
}
#[test]
fn kw_lookup_arabic_agent() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("وكيل", Language::Arabic), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_arabic_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("إذا", Language::Arabic), Some(Keyword::If));
}
#[test]
fn kw_lookup_russian_use() {
    let m = KeywordMap::new();
    assert_eq!(
        m.lookup("использовать", Language::Russian),
        Some(Keyword::Use)
    );
}
#[test]
fn kw_lookup_russian_agent() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("агент", Language::Russian), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_russian_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("если", Language::Russian), Some(Keyword::If));
}
#[test]
fn kw_lookup_chinese_use() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("使用", Language::Chinese), Some(Keyword::Use));
}
#[test]
fn kw_lookup_chinese_agent() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("代理", Language::Chinese), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_chinese_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("如果", Language::Chinese), Some(Keyword::If));
}
#[test]
fn kw_lookup_spanish_use() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("usar", Language::Spanish), Some(Keyword::Use));
}
#[test]
fn kw_lookup_spanish_agent() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("agente", Language::Spanish), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_spanish_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("si", Language::Spanish), Some(Keyword::If));
}
#[test]
fn kw_lookup_german_use() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("verwenden", Language::German), Some(Keyword::Use));
}
#[test]
fn kw_lookup_german_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("wenn", Language::German), Some(Keyword::If));
}
#[test]
fn kw_lookup_french_use() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("utiliser", Language::French), Some(Keyword::Use));
}
#[test]
fn kw_lookup_french_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("si", Language::French), Some(Keyword::If));
}
#[test]
fn kw_lookup_italian_use() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("usare", Language::Italian), Some(Keyword::Use));
}
#[test]
fn kw_lookup_italian_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("se", Language::Italian), Some(Keyword::If));
}
#[test]
fn kw_lookup_portuguese_use() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("usar", Language::Portuguese), Some(Keyword::Use));
}
#[test]
fn kw_lookup_portuguese_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("se", Language::Portuguese), Some(Keyword::If));
}
#[test]
fn kw_lookup_polish_let() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("niech", Language::Polish), Some(Keyword::Let));
}
#[test]
fn kw_lookup_polish_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("jeśli", Language::Polish), Some(Keyword::If));
}
#[test]
fn kw_lookup_polish_provider() {
    let m = KeywordMap::new();
    assert_eq!(
        m.lookup("dostawca", Language::Polish),
        Some(Keyword::Provider)
    );
}
#[test]
fn kw_lookup_polish_model() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("model", Language::Polish), Some(Keyword::Model));
}
#[test]
fn kw_lookup_polish_call() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("wywołaj", Language::Polish), Some(Keyword::Call));
}
#[test]
fn kw_lookup_thai_let() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("ให้", Language::Thai), Some(Keyword::Let));
}
#[test]
fn kw_lookup_thai_agent() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("เอเจนต์", Language::Thai), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_thai_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("ถ้า", Language::Thai), Some(Keyword::If));
}
#[test]
fn kw_lookup_indonesian_let() {
    let m = KeywordMap::new();
    assert_eq!(
        m.lookup("biarkan", Language::Indonesian),
        Some(Keyword::Let)
    );
}
#[test]
fn kw_lookup_indonesian_agent() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("agen", Language::Indonesian), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_indonesian_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("jika", Language::Indonesian), Some(Keyword::If));
}
#[test]
fn kw_lookup_vietnamese_let() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("cho", Language::Vietnamese), Some(Keyword::Let));
}
#[test]
fn kw_lookup_vietnamese_agent() {
    let m = KeywordMap::new();
    assert_eq!(
        m.lookup("đại_lý", Language::Vietnamese),
        Some(Keyword::Agent)
    );
}
#[test]
fn kw_lookup_vietnamese_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("nếu", Language::Vietnamese), Some(Keyword::If));
}
#[test]
fn kw_lookup_vietnamese_for() {
    let m = KeywordMap::new();
    assert_eq!(
        m.lookup("đối_với", Language::Vietnamese),
        Some(Keyword::For)
    );
}
#[test]
fn kw_lookup_greek_let() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("ας", Language::Greek), Some(Keyword::Let));
}
#[test]
fn kw_lookup_greek_agent() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("πράκτορας", Language::Greek), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_greek_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("αν", Language::Greek), Some(Keyword::If));
}
#[test]
fn kw_lookup_serbian_let() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("нека", Language::Serbian), Some(Keyword::Let));
}
#[test]
fn kw_lookup_serbian_agent() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("агент", Language::Serbian), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_serbian_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("ако", Language::Serbian), Some(Keyword::If));
}
#[test]
fn kw_lookup_bosnian_let() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("neka", Language::Bosnian), Some(Keyword::Let));
}
#[test]
fn kw_lookup_bosnian_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("ako", Language::Bosnian), Some(Keyword::If));
}
#[test]
fn kw_lookup_croatian_let() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("neka", Language::Croatian), Some(Keyword::Let));
}
#[test]
fn kw_lookup_croatian_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("ako", Language::Croatian), Some(Keyword::If));
}
#[test]
fn kw_lookup_kurdish_let() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("bila", Language::Kurdish), Some(Keyword::Let));
}
#[test]
fn kw_lookup_kurdish_agent() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("ajans", Language::Kurdish), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_kurdish_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("eger", Language::Kurdish), Some(Keyword::If));
}
#[test]
fn kw_lookup_persian_let() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("بگذار", Language::Persian), Some(Keyword::Let));
}
#[test]
fn kw_lookup_persian_agent() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("عامل", Language::Persian), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_persian_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("اگر", Language::Persian), Some(Keyword::If));
}
#[test]
fn kw_lookup_hindi_let() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("मान_लो", Language::Hindi), Some(Keyword::Let));
}
#[test]
fn kw_lookup_hindi_agent() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("एजेंट", Language::Hindi), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_hindi_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("अगर", Language::Hindi), Some(Keyword::If));
}
#[test]
fn kw_lookup_bengali_let() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("ধরো", Language::Bengali), Some(Keyword::Let));
}
#[test]
fn kw_lookup_bengali_agent() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("এজেন্ট", Language::Bengali), Some(Keyword::Agent));
}
#[test]
fn kw_lookup_bengali_if() {
    let m = KeywordMap::new();
    assert_eq!(m.lookup("যদি", Language::Bengali), Some(Keyword::If));
}
#[test]
fn kw_lookup_bengali_catch() {
    let m = KeywordMap::new();
    assert_eq!(
        m.lookup("ধরে_ফেলো", Language::Bengali),
        Some(Keyword::Catch)
    );
}

// ── KeywordMap — comprehensive to_english ────────────────────────────

#[test]
fn kw_to_english_import_export_from() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Import), "import");
    assert_eq!(m.to_english(Keyword::Export), "export");
    assert_eq!(m.to_english(Keyword::From), "from");
}
#[test]
fn kw_to_english_tool_resource() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Tool), "tool");
    assert_eq!(m.to_english(Keyword::Resource), "resource");
}
#[test]
fn kw_to_english_mcp_server_config() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Mcp), "mcp");
    assert_eq!(m.to_english(Keyword::Server), "server");
    assert_eq!(m.to_english(Keyword::Config), "config");
}
#[test]
fn kw_to_english_then_for_foreach_in() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Then), "then");
    assert_eq!(m.to_english(Keyword::For), "for");
    assert_eq!(m.to_english(Keyword::ForEach), "foreach");
    assert_eq!(m.to_english(Keyword::In), "in");
}
#[test]
fn kw_to_english_break_continue_switch_case_default() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Break), "break");
    assert_eq!(m.to_english(Keyword::Continue), "continue");
    assert_eq!(m.to_english(Keyword::Switch), "switch");
    assert_eq!(m.to_english(Keyword::Case), "case");
    assert_eq!(m.to_english(Keyword::Default), "default");
    assert_eq!(m.to_english(Keyword::Function), "function");
}
#[test]
fn kw_to_english_and_or() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::And), "and");
    assert_eq!(m.to_english(Keyword::Or), "or");
}
#[test]
fn kw_to_english_try_catch_finally_throw() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Try), "try");
    assert_eq!(m.to_english(Keyword::Catch), "catch");
    assert_eq!(m.to_english(Keyword::Finally), "finally");
    assert_eq!(m.to_english(Keyword::Throw), "throw");
}
#[test]
fn kw_to_english_let_var_const_data_set() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Let), "let");
    assert_eq!(m.to_english(Keyword::Var), "var");
    assert_eq!(m.to_english(Keyword::Const), "const");
    assert_eq!(m.to_english(Keyword::Data), "data");
    assert_eq!(m.to_english(Keyword::Set), "set");
}
#[test]
fn kw_to_english_oop() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Class), "class");
    assert_eq!(m.to_english(Keyword::Constructor), "constructor");
    assert_eq!(m.to_english(Keyword::This), "this");
    assert_eq!(m.to_english(Keyword::SelfKeyword), "self");
    assert_eq!(m.to_english(Keyword::Super), "super");
    assert_eq!(m.to_english(Keyword::New), "new");
    assert_eq!(m.to_english(Keyword::Extends), "extends");
    assert_eq!(m.to_english(Keyword::Public), "public");
    assert_eq!(m.to_english(Keyword::Private), "private");
    assert_eq!(m.to_english(Keyword::Protected), "protected");
    assert_eq!(m.to_english(Keyword::Static), "static");
}
#[test]
fn kw_to_english_intent_based() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Entity), "entity");
    assert_eq!(m.to_english(Keyword::State), "state");
    assert_eq!(m.to_english(Keyword::StateMachine), "statemachine");
    assert_eq!(m.to_english(Keyword::AgentState), "agent_state");
    assert_eq!(m.to_english(Keyword::Event), "event");
    assert_eq!(m.to_english(Keyword::Action), "action");
    assert_eq!(m.to_english(Keyword::Transform), "transform");
}
#[test]
fn kw_to_english_governance() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Constitution), "constitution");
    assert_eq!(m.to_english(Keyword::Law), "law");
    assert_eq!(m.to_english(Keyword::Rule), "rule");
    assert_eq!(m.to_english(Keyword::Council), "council");
    assert_eq!(m.to_english(Keyword::Swarm), "swarm");
    assert_eq!(m.to_english(Keyword::Community), "community");
    assert_eq!(m.to_english(Keyword::Enforcement), "enforcement");
    assert_eq!(m.to_english(Keyword::Mandatory), "mandatory");
    assert_eq!(m.to_english(Keyword::Advisory), "advisory");
    assert_eq!(m.to_english(Keyword::Optional), "optional");
    assert_eq!(m.to_english(Keyword::Role), "role");
    assert_eq!(m.to_english(Keyword::Member), "member");
    assert_eq!(m.to_english(Keyword::Strategy), "strategy");
    assert_eq!(m.to_english(Keyword::Competitive), "competitive");
    assert_eq!(m.to_english(Keyword::Collaborative), "collaborative");
    assert_eq!(m.to_english(Keyword::Culture), "culture");
    assert_eq!(m.to_english(Keyword::Values), "values");
    assert_eq!(m.to_english(Keyword::Norms), "norms");
    assert_eq!(
        m.to_english(Keyword::CommunicationStyle),
        "communication_style"
    );
    assert_eq!(m.to_english(Keyword::Formal), "formal");
    assert_eq!(m.to_english(Keyword::Informal), "informal");
    assert_eq!(m.to_english(Keyword::Technical), "technical");
}
#[test]
fn kw_to_english_intent_goals() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Intent), "intent");
    assert_eq!(m.to_english(Keyword::Want), "want");
    assert_eq!(m.to_english(Keyword::Priority), "priority");
    assert_eq!(m.to_english(Keyword::Flow), "flow");
    assert_eq!(m.to_english(Keyword::DataFlow), "data_flow");
}
#[test]
fn kw_to_english_agent_roles() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Prosecutor), "prosecutor");
    assert_eq!(m.to_english(Keyword::Judge), "judge");
    assert_eq!(m.to_english(Keyword::Executor), "executor");
}
#[test]
fn kw_to_english_music() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Note), "note");
    assert_eq!(m.to_english(Keyword::Chord), "chord");
    assert_eq!(m.to_english(Keyword::Melody), "melody");
    assert_eq!(m.to_english(Keyword::Harmony), "harmony");
    assert_eq!(m.to_english(Keyword::Rhythm), "rhythm");
    assert_eq!(m.to_english(Keyword::Tempo), "tempo");
    assert_eq!(m.to_english(Keyword::Scale), "scale");
}
#[test]
fn kw_to_english_constraints_permissions() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::When), "when");
    assert_eq!(m.to_english(Keyword::On), "on");
    assert_eq!(m.to_english(Keyword::Trigger), "trigger");
    assert_eq!(m.to_english(Keyword::Allow), "allow");
    assert_eq!(m.to_english(Keyword::Deny), "deny");
}
#[test]
fn kw_to_english_orchestration() {
    let m = KeywordMap::new();
    assert_eq!(m.to_english(Keyword::Layer), "layer");
    assert_eq!(m.to_english(Keyword::Network), "network");
    assert_eq!(m.to_english(Keyword::DependsOn), "depends_on");
    assert_eq!(m.to_english(Keyword::Broadcast), "broadcast");
    assert_eq!(m.to_english(Keyword::Merge), "merge");
    assert_eq!(m.to_english(Keyword::Parallel), "parallel");
    assert_eq!(m.to_english(Keyword::Sequential), "sequential");
    assert_eq!(m.to_english(Keyword::Execute), "execute");
    assert_eq!(m.to_english(Keyword::Provider), "provider");
    assert_eq!(m.to_english(Keyword::Model), "model");
    assert_eq!(m.to_english(Keyword::Call), "call");
    assert_eq!(m.to_english(Keyword::Promise), "promise");
    assert_eq!(m.to_english(Keyword::Future), "future");
}

// ── MessageCatalog — from_map ─────────────────────────────────────────

#[test]
fn catalog_from_map() {
    let messages: std::collections::HashMap<String, String> = [
        ("a".to_string(), "1".to_string()),
        ("b".to_string(), "2".to_string()),
    ]
    .into_iter()
    .collect();
    let cat = MessageCatalog::from_map("de", messages);
    assert_eq!(cat.locale(), "de");
    assert_eq!(cat.len(), 2);
    assert_eq!(cat.get("a"), Some("1"));
}

// ── Plural rules — edge cases ─────────────────────────────────────────

#[test]
fn plural_russian_10_many() {
    assert_eq!(RussianPlural.select(10.0), PluralCategory::Many);
}
#[test]
fn plural_russian_12_many() {
    assert_eq!(RussianPlural.select(12.0), PluralCategory::Many);
}
#[test]
fn plural_russian_14_many() {
    assert_eq!(RussianPlural.select(14.0), PluralCategory::Many);
}
#[test]
fn plural_russian_102_few() {
    assert_eq!(RussianPlural.select(102.0), PluralCategory::Few);
}
#[test]
fn plural_russian_25_many() {
    assert_eq!(RussianPlural.select(25.0), PluralCategory::Many);
}

// ── Interpolation — edge cases ────────────────────────────────────────

#[test]
fn interpolate_plural_two_category() {
    let mut vars = HashMap::new();
    vars.insert("n".into(), "2".into());
    let t = "{n, plural, one{# item} two{# items (dual)} other{# items}}";
    assert_eq!(interpolate(t, &vars), "2 items (dual)");
}
#[test]
fn interpolate_plural_missing_variable_preserved() {
    let vars = HashMap::new();
    let t = "{count, plural, one{# item} other{# items}}";
    assert_eq!(interpolate(t, &vars), t);
}
#[test]
fn interpolate_plural_fractional_one_category() {
    let mut vars = HashMap::new();
    vars.insert("n".into(), "1.5".into());
    let t = "{n, plural, one{# item} other{# items}}";
    assert_eq!(interpolate(t, &vars), "1.5 item");
}

// ── Locale — edge cases ───────────────────────────────────────────────

#[test]
fn locale_detect_system_posix() {
    std::env::set_var("LANG", "C");
    std::env::remove_var("LC_ALL");
    std::env::remove_var("LC_MESSAGES");
    let l = Locale::detect_system_locale();
    assert_eq!(l.language, "en");
}
#[test]
fn locale_fallback_no_duplicate_lang() {
    let chain = FallbackChain::new(&Locale::parse("fr"));
    let tags: Vec<&str> = chain.iter().collect();
    assert_eq!(tags, vec!["fr", "en"]);
}
#[test]
fn locale_parse_script_only_two_parts() {
    let l = Locale::parse("zh-Hans");
    assert_eq!(l.language, "zh");
    assert_eq!(l.script, Some("Hans".into()));
    assert_eq!(l.region, None);
    assert_eq!(l.to_tag(), "zh-Hans");
}
#[test]
fn locale_display_with_script_and_region() {
    let l = Locale::parse("zh-Hans-CN");
    assert_eq!(format!("{}", l), "zh-Hans-CN");
}
#[test]
fn locale_parse_empty() {
    let l = Locale::parse("");
    assert_eq!(l.language, "");
    assert_eq!(l.to_tag(), "");
}
