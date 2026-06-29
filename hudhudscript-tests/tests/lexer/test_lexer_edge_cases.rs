//! Edge case and full coverage tests for hudhudscript-lexer
//! Covers: japanese_numeral_to_number, is_ident_start/continue, Lexer, Token/TokenKind

use hudhudscript_lexer::*;

// ═══════════════════════════════════════════════════════════════════════════
// japanese_numeral_to_number — lookup table + compositional parsing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn jp_numeral_single_zero_kanji() {
    assert_eq!(japanese_numeral_to_number("零"), Some(0.0));
    assert_eq!(japanese_numeral_to_number("〇"), Some(0.0));
}

#[test]
fn jp_numeral_single_digits() {
    assert_eq!(japanese_numeral_to_number("一"), Some(1.0));
    assert_eq!(japanese_numeral_to_number("二"), Some(2.0));
    assert_eq!(japanese_numeral_to_number("三"), Some(3.0));
    assert_eq!(japanese_numeral_to_number("四"), Some(4.0));
    assert_eq!(japanese_numeral_to_number("五"), Some(5.0));
    assert_eq!(japanese_numeral_to_number("六"), Some(6.0));
    assert_eq!(japanese_numeral_to_number("七"), Some(7.0));
    assert_eq!(japanese_numeral_to_number("八"), Some(8.0));
    assert_eq!(japanese_numeral_to_number("九"), Some(9.0));
}

#[test]
fn jp_numeral_powers_of_ten() {
    assert_eq!(japanese_numeral_to_number("十"), Some(10.0));
    assert_eq!(japanese_numeral_to_number("百"), Some(100.0));
    assert_eq!(japanese_numeral_to_number("千"), Some(1000.0));
    assert_eq!(japanese_numeral_to_number("万"), Some(10000.0));
    assert_eq!(japanese_numeral_to_number("億"), Some(100000000.0));
}

#[test]
fn jp_numeral_teens_lookup() {
    assert_eq!(japanese_numeral_to_number("十一"), Some(11.0));
    assert_eq!(japanese_numeral_to_number("十五"), Some(15.0));
    assert_eq!(japanese_numeral_to_number("十九"), Some(19.0));
}

#[test]
fn jp_numeral_twenties_lookup() {
    assert_eq!(japanese_numeral_to_number("二十"), Some(20.0));
    assert_eq!(japanese_numeral_to_number("二十一"), Some(21.0));
    assert_eq!(japanese_numeral_to_number("二十五"), Some(25.0));
    assert_eq!(japanese_numeral_to_number("二十九"), Some(29.0));
}

#[test]
fn jp_numeral_thirties_lookup() {
    assert_eq!(japanese_numeral_to_number("三十"), Some(30.0));
    assert_eq!(japanese_numeral_to_number("三十五"), Some(35.0));
}

#[test]
fn jp_numeral_forties_lookup() {
    assert_eq!(japanese_numeral_to_number("四十"), Some(40.0));
    assert_eq!(japanese_numeral_to_number("四十二"), Some(42.0));
}

#[test]
fn jp_numeral_fifties_lookup() {
    assert_eq!(japanese_numeral_to_number("五十"), Some(50.0));
    assert_eq!(japanese_numeral_to_number("五十三"), Some(53.0));
}

#[test]
fn jp_numeral_sixties_lookup() {
    assert_eq!(japanese_numeral_to_number("六十"), Some(60.0));
    assert_eq!(japanese_numeral_to_number("六十四"), Some(64.0));
}

#[test]
fn jp_numeral_seventies_lookup() {
    assert_eq!(japanese_numeral_to_number("七十"), Some(70.0));
    assert_eq!(japanese_numeral_to_number("七十五"), Some(75.0));
}

#[test]
fn jp_numeral_eighties_lookup() {
    assert_eq!(japanese_numeral_to_number("八十"), Some(80.0));
    assert_eq!(japanese_numeral_to_number("八十六"), Some(86.0));
}

#[test]
fn jp_numeral_nineties_lookup() {
    assert_eq!(japanese_numeral_to_number("九十"), Some(90.0));
    assert_eq!(japanese_numeral_to_number("九十九"), Some(99.0));
}

#[test]
fn jp_numeral_hundreds_lookup() {
    assert_eq!(japanese_numeral_to_number("二百"), Some(200.0));
    assert_eq!(japanese_numeral_to_number("三百"), Some(300.0));
    assert_eq!(japanese_numeral_to_number("五百"), Some(500.0));
    assert_eq!(japanese_numeral_to_number("九百"), Some(900.0));
}

// Compositional parsing tests (not in lookup table)

#[test]
fn jp_numeral_compositional_99() {
    // 九九 = 99 (special shorthand)
    assert_eq!(japanese_numeral_to_number("九九"), Some(99.0));
}

#[test]
fn jp_numeral_compositional_153() {
    // 百五十三 = 153
    assert_eq!(japanese_numeral_to_number("百五十三"), Some(153.0));
}

#[test]
fn jp_numeral_compositional_3245() {
    // 三千二百四十五 = 3245
    assert_eq!(japanese_numeral_to_number("三千二百四十五"), Some(3245.0));
}

#[test]
fn jp_numeral_compositional_12345() {
    // 一万二千三百四十五 = 12345
    assert_eq!(
        japanese_numeral_to_number("一万二千三百四十五"),
        Some(12345.0)
    );
}

#[test]
fn jp_numeral_compositional_large() {
    // 一億二千三百四十五万六千七百八十九 = 123456789
    let result = japanese_numeral_to_number("一億二千三百四十五万六千七百八十九");
    assert_eq!(result, Some(123456789.0));
}

#[test]
fn jp_numeral_compositional_just_wari() {
    // 万 alone should be processed
    let result = japanese_numeral_to_number("一万");
    assert_eq!(result, Some(10000.0));
}

#[test]
fn jp_numeral_compositional_just_oku() {
    let result = japanese_numeral_to_number("一億");
    assert_eq!(result, Some(100000000.0));
}

#[test]
fn jp_numeral_compositional_hundred() {
    // 二百三 = 203
    assert_eq!(japanese_numeral_to_number("二百三"), Some(203.0));
}

#[test]
fn jp_numeral_compositional_thousand() {
    // 二千五 = 2005
    assert_eq!(japanese_numeral_to_number("二千五"), Some(2005.0));
}

#[test]
fn jp_numeral_compositional_mixed() {
    // 千二十三 = 1023
    assert_eq!(japanese_numeral_to_number("千二十三"), Some(1023.0));
}

#[test]
fn jp_numeral_empty_string() {
    assert_eq!(japanese_numeral_to_number(""), None);
}

#[test]
fn jp_numeral_invalid_chars() {
    assert_eq!(japanese_numeral_to_number("abc"), None);
    assert_eq!(japanese_numeral_to_number("123"), None);
    assert_eq!(japanese_numeral_to_number("あいう"), None);
}

#[test]
fn jp_numeral_zero_with_unit() {
    // 零百 = 0 (digit zero followed by hundred)
    assert_eq!(japanese_numeral_to_number("零百"), Some(0.0));
}

// ═══════════════════════════════════════════════════════════════════════════
// is_ident_start / is_ident_continue — charclass coverage
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ident_start_ascii_lowercase() {
    assert!(is_ident_start('a'));
    assert!(is_ident_start('z'));
    assert!(is_ident_start('m'));
}

#[test]
fn ident_start_ascii_uppercase() {
    assert!(is_ident_start('A'));
    assert!(is_ident_start('Z'));
    assert!(is_ident_start('M'));
}

#[test]
fn ident_start_underscore() {
    assert!(is_ident_start('_'));
}

#[test]
fn ident_start_unicode_letter() {
    assert!(is_ident_start('ä')); // Latin small a with diaeresis
    assert!(is_ident_start('ğ')); // Turkish g with breve
    assert!(is_ident_start('ş')); // Turkish s with cedilla
}

#[test]
fn ident_start_cjk() {
    assert!(is_ident_start('変')); // Japanese kanji
    assert!(is_ident_start('数')); // Japanese kanji
    assert!(is_ident_start('如')); // Chinese character
}

#[test]
fn ident_start_digits_are_false() {
    assert!(!is_ident_start('0'));
    assert!(!is_ident_start('9'));
    assert!(!is_ident_start('5'));
}

#[test]
fn ident_start_punctuation_false() {
    assert!(!is_ident_start('!'));
    assert!(!is_ident_start('.'));
    assert!(!is_ident_start('+'));
}

#[test]
fn ident_start_whitespace_false() {
    assert!(!is_ident_start(' '));
    assert!(!is_ident_start('\n'));
    assert!(!is_ident_start('\t'));
}

#[test]
fn ident_continue_digits() {
    assert!(is_ident_continue('0'));
    assert!(is_ident_continue('9'));
}

#[test]
fn ident_continue_letters() {
    assert!(is_ident_continue('a'));
    assert!(is_ident_continue('Z'));
    assert!(is_ident_continue('_'));
}

#[test]
fn ident_continue_unicode() {
    assert!(is_ident_continue('ç')); // c-cedilla
    assert!(is_ident_continue('ı')); // dotless i
}

#[test]
fn ident_continue_cjk() {
    assert!(is_ident_continue('変'));
    assert!(is_ident_continue('数'));
}

#[test]
fn ident_continue_punctuation_false() {
    assert!(!is_ident_continue('?'));
    assert!(!is_ident_continue(','));
}

// ═══════════════════════════════════════════════════════════════════════════
