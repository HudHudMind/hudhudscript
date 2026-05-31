pub(crate) fn is_arabic_digit(ch: char) -> bool {
    matches!(ch, '\u{0660}'..='\u{0669}')
}

/// Convert Arabic-Indic digit to ASCII digit
pub(crate) fn arabic_to_ascii_digit(ch: char) -> char {
    match ch {
        '\u{0660}' => '0', // ٠
        '\u{0661}' => '1', // ١
        '\u{0662}' => '2', // ٢
        '\u{0663}' => '3', // ٣
        '\u{0664}' => '4', // ٤
        '\u{0665}' => '5', // ٥
        '\u{0666}' => '6', // ٦
        '\u{0667}' => '7', // ٧
        '\u{0668}' => '8', // ٨
        '\u{0669}' => '9', // ٩
        _ => ch,
    }
}

/// Check if a character is a Japanese kanji numeral
pub(crate) fn is_japanese_numeral(ch: char) -> bool {
    matches!(
        ch,
        '〇' | '零' |  // 0
        '一' |         // 1
        '二' |         // 2
        '三' |         // 3
        '四' |         // 4
        '五' |         // 5
        '六' |         // 6
        '七' |         // 7
        '八' |         // 8
        '九' |         // 9
        '十' |         // 10
        '百' |         // 100
        '千' |         // 1000
        '万' |         // 10000
        '億' // 100000000
    )
}

/// Convert a single Japanese kanji numeral character to number.
/// Used internally by the lexer during single-character tokenization.
pub(crate) fn japanese_numeral_char_to_number(ch: char) -> Option<f64> {
    match ch {
        '〇' | '零' => Some(0.0),
        '一' => Some(1.0),
        '二' => Some(2.0),
        '三' => Some(3.0),
        '四' => Some(4.0),
        '五' => Some(5.0),
        '六' => Some(6.0),
        '七' => Some(7.0),
        '八' => Some(8.0),
        '九' => Some(9.0),
        '十' => Some(10.0),
        '百' => Some(100.0),
        '千' => Some(1000.0),
        '万' => Some(10000.0),
        '億' => Some(100000000.0),
        _ => None,
    }
}

/// Parse Japanese/Chinese Kanji numeral string to number (supports any size).
///
/// This is the canonical implementation used by both lexer and parser.
/// Examples: "百五十三" = 153, "三千二百四十五" = 3245, "一万二千三百四十五" = 12345
pub fn japanese_numeral_to_number(s: &str) -> Option<f64> {
    // First try the lookup table for common numbers
    if let Some(num) = japanese_numeral_lookup(s) {
        return Some(num);
    }
    // If not in lookup, parse compositionally
    parse_kanji_number(s)
}

/// Lookup table for common Japanese/Chinese numerals
fn japanese_numeral_lookup(s: &str) -> Option<f64> {
    match s {
        // Two-digit numbers with units (11-99)
        "九十九" => Some(99.0),
        "九十八" => Some(98.0),
        "九十七" => Some(97.0),
        "九十六" => Some(96.0),
        "九十五" => Some(95.0),
        "九十四" => Some(94.0),
        "九十三" => Some(93.0),
        "九十二" => Some(92.0),
        "九十一" => Some(91.0),
        "九十" => Some(90.0),
        "八十九" => Some(89.0),
        "八十八" => Some(88.0),
        "八十七" => Some(87.0),
        "八十六" => Some(86.0),
        "八十五" => Some(85.0),
        "八十四" => Some(84.0),
        "八十三" => Some(83.0),
        "八十二" => Some(82.0),
        "八十一" => Some(81.0),
        "八十" => Some(80.0),
        "七十九" => Some(79.0),
        "七十八" => Some(78.0),
        "七十七" => Some(77.0),
        "七十六" => Some(76.0),
        "七十五" => Some(75.0),
        "七十四" => Some(74.0),
        "七十三" => Some(73.0),
        "七十二" => Some(72.0),
        "七十一" => Some(71.0),
        "七十" => Some(70.0),
        "六十九" => Some(69.0),
        "六十八" => Some(68.0),
        "六十七" => Some(67.0),
        "六十六" => Some(66.0),
        "六十五" => Some(65.0),
        "六十四" => Some(64.0),
        "六十三" => Some(63.0),
        "六十二" => Some(62.0),
        "六十一" => Some(61.0),
        "六十" => Some(60.0),
        "五十九" => Some(59.0),
        "五十八" => Some(58.0),
        "五十七" => Some(57.0),
        "五十六" => Some(56.0),
        "五十五" => Some(55.0),
        "五十四" => Some(54.0),
        "五十三" => Some(53.0),
        "五十二" => Some(52.0),
        "五十一" => Some(51.0),
        "五十" => Some(50.0),
        "四十九" => Some(49.0),
        "四十八" => Some(48.0),
        "四十七" => Some(47.0),
        "四十六" => Some(46.0),
        "四十五" => Some(45.0),
        "四十四" => Some(44.0),
        "四十三" => Some(43.0),
        "四十二" => Some(42.0),
        "四十一" => Some(41.0),
        "四十" => Some(40.0),
        "三十九" => Some(39.0),
        "三十八" => Some(38.0),
        "三十七" => Some(37.0),
        "三十六" => Some(36.0),
        "三十五" => Some(35.0),
        "三十四" => Some(34.0),
        "三十三" => Some(33.0),
        "三十二" => Some(32.0),
        "三十一" => Some(31.0),
        "三十" => Some(30.0),
        "二十九" => Some(29.0),
        "二十八" => Some(28.0),
        "二十七" => Some(27.0),
        "二十六" => Some(26.0),
        "二十五" => Some(25.0),
        "二十四" => Some(24.0),
        "二十三" => Some(23.0),
        "二十二" => Some(22.0),
        "二十一" => Some(21.0),
        "二十" => Some(20.0),
        "十九" => Some(19.0),
        "十八" => Some(18.0),
        "十七" => Some(17.0),
        "十六" => Some(16.0),
        "十五" => Some(15.0),
        "十四" => Some(14.0),
        "十三" => Some(13.0),
        "十二" => Some(12.0),
        "十一" => Some(11.0),
        // Hundreds (100-900)
        "九百" => Some(900.0),
        "八百" => Some(800.0),
        "七百" => Some(700.0),
        "六百" => Some(600.0),
        "五百" => Some(500.0),
        "四百" => Some(400.0),
        "三百" => Some(300.0),
        "二百" => Some(200.0),
        // Large numbers
        "億" => Some(100000000.0),
        "万" => Some(10000.0),
        "千" => Some(1000.0),
        "百" => Some(100.0),
        "十" => Some(10.0),
        // Single digits
        "九" => Some(9.0),
        "八" => Some(8.0),
        "七" => Some(7.0),
        "六" => Some(6.0),
        "五" => Some(5.0),
        "四" => Some(4.0),
        "三" => Some(3.0),
        "二" => Some(2.0),
        "一" => Some(1.0),
        "零" | "〇" => Some(0.0),
        _ => None,
    }
}

/// Parse Kanji number compositionally (supports any size)
/// Examples: 百五十三 = 153, 三千二百四十五 = 3245, 一万二千三百四十五 = 12345
/// Special: 九九 = 99 (common shorthand for 九十九)
fn parse_kanji_number(s: &str) -> Option<f64> {
    // Special case: 九九 (99) - common shorthand
    if s == "九九" {
        return Some(99.0);
    }

    let chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let mut result = 0.0;
    let mut current = 0.0;
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Get digit value
        let digit = match ch {
            '零' | '〇' => 0.0,
            '一' => 1.0,
            '二' => 2.0,
            '三' => 3.0,
            '四' => 4.0,
            '五' => 5.0,
            '六' => 6.0,
            '七' => 7.0,
            '八' => 8.0,
            '九' => 9.0,
            '十' => {
                if current == 0.0 {
                    current = 10.0;
                } else {
                    current *= 10.0;
                }
                i += 1;
                continue;
            }
            '百' => {
                if current == 0.0 {
                    current = 100.0;
                } else {
                    current *= 100.0;
                }
                i += 1;
                continue;
            }
            '千' => {
                if current == 0.0 {
                    current = 1000.0;
                } else {
                    current *= 1000.0;
                }
                i += 1;
                continue;
            }
            '万' => {
                if current == 0.0 {
                    current = 1.0;
                }
                result += current * 10000.0;
                current = 0.0;
                i += 1;
                continue;
            }
            '億' => {
                if current == 0.0 {
                    current = 1.0;
                }
                result += current * 100000000.0;
                current = 0.0;
                i += 1;
                continue;
            }
            _ => return None,
        };

        // Check if next char is a multiplier
        if i + 1 < chars.len() {
            let next = chars[i + 1];
            match next {
                '十' => {
                    current += digit * 10.0;
                    i += 2;
                    continue;
                }
                '百' => {
                    current += digit * 100.0;
                    i += 2;
                    continue;
                }
                '千' => {
                    current += digit * 1000.0;
                    i += 2;
                    continue;
                }
                '万' => {
                    result += (current + digit) * 10000.0;
                    current = 0.0;
                    i += 2;
                    continue;
                }
                '億' => {
                    result += (current + digit) * 100000000.0;
                    current = 0.0;
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }

        // Just a digit
        current += digit;
        i += 1;
    }

    result += current;
    Some(result)
}
