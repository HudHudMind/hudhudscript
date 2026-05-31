use hudhudscript_parser::{parse_lang_directive, strip_lang_directive};

#[test]
fn test_parse_lang_directive_english() {
    assert_eq!(
        parse_lang_directive("#!lang=en\nlet x = 1;"),
        Some("en".to_string())
    );
}

#[test]
fn test_parse_lang_directive_turkish() {
    assert_eq!(
        parse_lang_directive("#!dil=tr\nlet x = 1;"),
        Some("tr".to_string())
    );
}

#[test]
fn test_parse_lang_directive_french() {
    assert_eq!(
        parse_lang_directive("#!langue=fr\nlet x = 1;"),
        Some("fr".to_string())
    );
}

#[test]
fn test_parse_lang_directive_japanese() {
    assert_eq!(
        parse_lang_directive("#!言語=ja\nlet x = 1;"),
        Some("ja".to_string())
    );
    assert_eq!(
        parse_lang_directive("#!gengo=ja\nlet x = 1;"),
        Some("ja".to_string())
    );
}

#[test]
fn test_parse_lang_directive_arabic() {
    assert_eq!(
        parse_lang_directive("#!لغة=ar\nlet x = 1;"),
        Some("ar".to_string())
    );
    assert_eq!(
        parse_lang_directive("#!lugha=ar\nlet x = 1;"),
        Some("ar".to_string())
    );
}

#[test]
fn test_parse_lang_directive_spanish() {
    assert_eq!(
        parse_lang_directive("#!idioma=es\nlet x = 1;"),
        Some("es".to_string())
    );
}

#[test]
fn test_parse_lang_directive_german() {
    assert_eq!(
        parse_lang_directive("#!sprache=de\nlet x = 1;"),
        Some("de".to_string())
    );
}

#[test]
fn test_parse_lang_directive_russian() {
    assert_eq!(
        parse_lang_directive("#!язык=ru\nlet x = 1;"),
        Some("ru".to_string())
    );
    assert_eq!(
        parse_lang_directive("#!yazyk=ru\nlet x = 1;"),
        Some("ru".to_string())
    );
}

#[test]
fn test_parse_lang_directive_chinese() {
    assert_eq!(
        parse_lang_directive("#!语言=zh\nlet x = 1;"),
        Some("zh".to_string())
    );
    assert_eq!(
        parse_lang_directive("#!yuyan=zh\nlet x = 1;"),
        Some("zh".to_string())
    );
}

#[test]
fn test_parse_lang_directive_italian() {
    assert_eq!(
        parse_lang_directive("#!lingua=it\nlet x = 1;"),
        Some("it".to_string())
    );
}

#[test]
fn test_parse_lang_directive_korean() {
    assert_eq!(
        parse_lang_directive("#!언어=ko\nlet x = 1;"),
        Some("ko".to_string())
    );
    assert_eq!(
        parse_lang_directive("#!eoneo=ko\nlet x = 1;"),
        Some("ko".to_string())
    );
}

#[test]
fn test_parse_lang_directive_portuguese() {
    assert_eq!(
        parse_lang_directive("#!idioma=pt\nlet x = 1;"),
        Some("pt".to_string())
    );
}

#[test]
fn test_no_directive() {
    assert_eq!(parse_lang_directive("let x = 1;"), None);
    assert_eq!(parse_lang_directive("// comment\nlet x = 1;"), None);
}

#[test]
fn test_invalid_directive() {
    // Missing value
    assert_eq!(parse_lang_directive("#!lang="), None);
    // Unknown key
    assert_eq!(parse_lang_directive("#!foo=tr"), None);
}

#[test]
fn test_directive_with_spaces() {
    assert_eq!(
        parse_lang_directive("#!lang = tr\nlet x = 1;"),
        Some("tr".to_string())
    );
}

#[test]
fn test_strip_lang_directive() {
    let source = "#!lang=tr\nlet x = 1;\nlet y = 2;";
    let stripped = strip_lang_directive(source);
    assert_eq!(stripped, "\nlet x = 1;\nlet y = 2;");
}

#[test]
fn test_strip_no_directive() {
    let source = "let x = 1;\nlet y = 2;";
    let stripped = strip_lang_directive(source);
    assert_eq!(stripped, source);
}

#[test]
fn test_strip_directive_only() {
    let source = "#!lang=tr";
    let stripped = strip_lang_directive(source);
    assert_eq!(stripped, "");
}
