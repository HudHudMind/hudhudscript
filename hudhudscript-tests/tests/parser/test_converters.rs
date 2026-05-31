//! External tests for `hudhudscript_parser::parser::converters` module.

use hudhudscript_parser::{arabic_to_ascii, japanese_numeral_to_number};

// ── arabic_to_ascii ────────────────────────────────────────────────

#[test]
fn test_arabic_to_ascii_empty() {
    assert_eq!(arabic_to_ascii(""), "");
}

#[test]
fn test_arabic_to_ascii_passthrough() {
    assert_eq!(arabic_to_ascii("12345"), "12345");
}

#[test]
fn test_arabic_indic_digits() {
    assert_eq!(arabic_to_ascii("٠١٢٣٤٥٦٧٨٩"), "0123456789");
}

#[test]
fn test_bengali_digits() {
    assert_eq!(arabic_to_ascii("০১২৩৪৫৬৭৮৯"), "0123456789");
}

#[test]
fn test_fullwidth_digits() {
    assert_eq!(arabic_to_ascii("０１２３４５６７８９"), "0123456789");
}

#[test]
fn test_kanji_digit_chars() {
    assert_eq!(arabic_to_ascii("〇一二三四五六七八九"), "0123456789");
}

#[test]
fn test_zero_kanji() {
    assert_eq!(arabic_to_ascii("零"), "0");
}

#[test]
fn test_mixed_arabic_and_ascii() {
    assert_eq!(arabic_to_ascii("١2٣"), "123");
}

// ── japanese_numeral_to_number lookup ──────────────────────────────

#[test]
fn test_single_digit_kanji() {
    assert_eq!(japanese_numeral_to_number("一"), Some(1.0));
    assert_eq!(japanese_numeral_to_number("九"), Some(9.0));
    assert_eq!(japanese_numeral_to_number("零"), Some(0.0));
    assert_eq!(japanese_numeral_to_number("〇"), Some(0.0));
}

#[test]
fn test_ten() {
    assert_eq!(japanese_numeral_to_number("十"), Some(10.0));
}

#[test]
fn test_teens() {
    assert_eq!(japanese_numeral_to_number("十一"), Some(11.0));
    assert_eq!(japanese_numeral_to_number("十九"), Some(19.0));
}

#[test]
fn test_multiples_of_ten() {
    assert_eq!(japanese_numeral_to_number("二十"), Some(20.0));
    assert_eq!(japanese_numeral_to_number("五十"), Some(50.0));
    assert_eq!(japanese_numeral_to_number("九十"), Some(90.0));
}

#[test]
fn test_two_digit_numbers() {
    assert_eq!(japanese_numeral_to_number("四十二"), Some(42.0));
    assert_eq!(japanese_numeral_to_number("九十九"), Some(99.0));
}

#[test]
fn test_hundreds() {
    assert_eq!(japanese_numeral_to_number("百"), Some(100.0));
    assert_eq!(japanese_numeral_to_number("二百"), Some(200.0));
    assert_eq!(japanese_numeral_to_number("九百"), Some(900.0));
}

#[test]
fn test_thousand() {
    assert_eq!(japanese_numeral_to_number("千"), Some(1000.0));
}

#[test]
fn test_ten_thousand() {
    assert_eq!(japanese_numeral_to_number("万"), Some(10000.0));
}

#[test]
fn test_hundred_million() {
    assert_eq!(japanese_numeral_to_number("億"), Some(100000000.0));
}

// ── Compositional parsing ──────────────────────────────────────────

#[test]
fn test_parse_kanji_153() {
    assert_eq!(japanese_numeral_to_number("百五十三"), Some(153.0));
}

#[test]
fn test_parse_kanji_3245() {
    assert_eq!(japanese_numeral_to_number("三千二百四十五"), Some(3245.0));
}

#[test]
fn test_parse_kanji_99_shorthand() {
    assert_eq!(japanese_numeral_to_number("九九"), Some(99.0));
}

#[test]
fn test_non_kanji_returns_none() {
    assert_eq!(japanese_numeral_to_number("hello"), None);
    assert_eq!(japanese_numeral_to_number("abc"), None);
}

#[test]
fn test_empty_returns_none() {
    assert_eq!(japanese_numeral_to_number(""), None);
}

#[test]
fn test_mixed_kanji_non_kanji_returns_none() {
    assert_eq!(japanese_numeral_to_number("三x"), None);
}

#[test]
fn test_man_unit() {
    // 二万 = 20,000
    assert_eq!(japanese_numeral_to_number("二万"), Some(20000.0));
}

#[test]
fn test_oku_unit() {
    // 二億 = 200,000,000
    assert_eq!(japanese_numeral_to_number("二億"), Some(200000000.0));
}

#[test]
fn test_complex_number_12345() {
    // 一万二千三百四十五 = 12345
    assert_eq!(
        japanese_numeral_to_number("一万二千三百四十五"),
        Some(12345.0)
    );
}

// ── Additional compositional parsing ──────────────────────────────

#[test]
fn test_parse_kanji_hundred_fifty() {
    // 百五十 = 150
    assert_eq!(japanese_numeral_to_number("百五十"), Some(150.0));
}

#[test]
fn test_parse_kanji_thousand_one() {
    // 千一 = 1001
    assert_eq!(japanese_numeral_to_number("千一"), Some(1001.0));
}

#[test]
fn test_parse_kanji_three_hundred() {
    // Compositional parsing: 三百 should be 300
    assert_eq!(japanese_numeral_to_number("三百"), Some(300.0));
}

#[test]
fn test_parse_kanji_five_thousand() {
    // 五千 = 5000
    assert_eq!(japanese_numeral_to_number("五千"), Some(5000.0));
}

#[test]
fn test_parse_kanji_zero() {
    assert_eq!(japanese_numeral_to_number("零"), Some(0.0));
    assert_eq!(japanese_numeral_to_number("〇"), Some(0.0));
}

#[test]
fn test_parse_kanji_oku_prefix() {
    // 三億 = 300,000,000
    assert_eq!(japanese_numeral_to_number("三億"), Some(300000000.0));
}

#[test]
fn test_parse_kanji_man_with_remainder() {
    // 三万五千 = 35,000
    assert_eq!(japanese_numeral_to_number("三万五千"), Some(35000.0));
}

#[test]
fn test_parse_kanji_hundred_unit_alone() {
    // 百 = 100 (from lookup)
    assert_eq!(japanese_numeral_to_number("百"), Some(100.0));
}

#[test]
fn test_parse_kanji_just_digit() {
    // 五 = 5
    assert_eq!(japanese_numeral_to_number("五"), Some(5.0));
}

// ── arabic_to_ascii edge cases ────────────────────────────────────

#[test]
fn test_arabic_mixed_with_spaces() {
    assert_eq!(arabic_to_ascii("٣ ٧"), "3 7");
}

#[test]
fn test_bengali_mixed_with_ascii() {
    assert_eq!(arabic_to_ascii("১0২"), "102");
}

#[test]
fn test_fullwidth_single_digit() {
    assert_eq!(arabic_to_ascii("５"), "5");
}
