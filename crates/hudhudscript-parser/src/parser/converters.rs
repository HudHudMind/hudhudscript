//! Number conversion utilities for multi-language support

/// Convert Arabic-Indic digits to ASCII digits
pub fn arabic_to_ascii(s: &str) -> String {
    s.chars()
        .map(|ch| match ch {
            // Arabic-Indic digits: ٠-٩
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
            // Bengali-Indic digits: ০-৯
            '\u{09E6}' => '0', // ০
            '\u{09E7}' => '1', // ১
            '\u{09E8}' => '2', // ২
            '\u{09E9}' => '3', // ৩
            '\u{09EA}' => '4', // ৪
            '\u{09EB}' => '5', // ৫
            '\u{09EC}' => '6', // ৬
            '\u{09ED}' => '7', // ৭
            '\u{09EE}' => '8', // ৮
            '\u{09EF}' => '9', // ৯
            // Full-width digits: ０-９
            '\u{FF10}' => '0', // ０
            '\u{FF11}' => '1', // １
            '\u{FF12}' => '2', // ２
            '\u{FF13}' => '3', // ３
            '\u{FF14}' => '4', // ４
            '\u{FF15}' => '5', // ５
            '\u{FF16}' => '6', // ６
            '\u{FF17}' => '7', // ７
            '\u{FF18}' => '8', // ８
            '\u{FF19}' => '9', // ９
            // Japanese/Chinese Kanji digits
            '〇' | '零' => '0',
            '一' => '1',
            '二' => '2',
            '三' => '3',
            '四' => '4',
            '五' => '5',
            '六' => '6',
            '七' => '7',
            '八' => '8',
            '九' => '9',
            _ => ch,
        })
        .collect()
}

/// Parse Japanese/Chinese Kanji numeral to number — re-exported from lexer.
///
/// The canonical implementation lives in `hudhudscript_lexer` (issue #961).
pub use hudhudscript_lexer::japanese_numeral_to_number;
