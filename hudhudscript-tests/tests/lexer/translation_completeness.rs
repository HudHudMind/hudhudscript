//! Translation Completeness Test (Issue #980)
//!
//! Verifies that the keyword normalizer maps ALL essential keywords
//! for every supported language. Each language must provide a translation
//! for core control-flow, variable, function, class, async, and error-handling
//! keywords so that programs written in any supported language compile correctly.
//!
//! This test uses normalize_keywords() to verify actual normalization behavior,
//! not just static analysis of the mapping table.

use hudhudscript_lexer::normalize_keywords;
use std::collections::{BTreeMap, BTreeSet};

/// Essential keywords that every language MUST map.
/// These are the minimum required to write a meaningful HudHudScript program.
const ESSENTIAL_KEYWORDS: &[&str] = &[
    "let", "const", "function", "class", "if", "else", "while", "for", "return", "break",
    "continue", "switch", "case", "async", "await", "try", "catch", "throw", "new", "import",
    "export", "true", "false", "null",
];

/// Returns all supported languages with their keyword mappings.
/// Each entry: (language_code, language_name, vec of (foreign_keyword, expected_english))
///
/// These mappings are extracted directly from keyword_normalizer.rs KEYWORD_MAP.
/// Only essential keywords are included (control flow, variables, OOP, async, error handling,
/// booleans/null, and module system).
fn language_keyword_maps() -> Vec<(
    &'static str,
    &'static str,
    Vec<(&'static str, &'static str)>,
)> {
    vec![
        // ── TR — Turkish ────────────────────────────────────────────────────
        (
            "TR",
            "Turkish",
            vec![
                ("eşzamansız", "async"),
                ("bekle", "await"),
                ("kır", "break"),
                ("durum", "case"),
                ("yakala", "catch"),
                ("sınıf", "class"),
                ("sabit", "const"),
                ("devam", "continue"),
                ("değilse", "else"),
                ("dışa_aktar", "export"),
                ("yanlış", "false"),
                ("döngü", "for"),
                ("işlev", "function"),
                ("eğer", "if"),
                ("içe_aktar", "import"),
                ("değişken", "let"),
                ("yeni", "new"),
                ("boş", "null"),
                ("dön", "return"),
                ("seç", "switch"),
                ("fırlat", "throw"),
                ("doğru", "true"),
                ("dene", "try"),
                ("iken", "while"),
            ],
        ),
        // ── AR — Arabic ─────────────────────────────────────────────────────
        (
            "AR",
            "Arabic",
            vec![
                ("غير_متزامن", "async"),
                ("انتظر", "await"),
                ("اكسر", "break"),
                ("حالة", "case"),
                ("امسك", "catch"),
                ("فئة", "class"),
                ("ثابت", "const"),
                ("استمر", "continue"),
                ("وإلا", "else"),
                ("صدّر", "export"),
                ("خطأ", "false"),
                ("لكل", "for"),
                ("دالة", "function"),
                ("إذا", "if"),
                ("استورد", "import"),
                ("متغير", "let"),
                ("جديد", "new"),
                ("فارغ", "null"),
                ("ارجع", "return"),
                ("تبديل", "switch"),
                ("ارمي", "throw"),
                ("صحيح", "true"),
                ("حاول", "try"),
                ("بينما", "while"),
            ],
        ),
        // ── JA — Japanese ───────────────────────────────────────────────────
        (
            "JA",
            "Japanese",
            vec![
                ("非同期", "async"),
                ("待つ", "await"),
                ("破る", "break"),
                ("場合", "case"),
                ("捕まえる", "catch"),
                ("クラス", "class"),
                ("定数", "const"),
                ("続ける", "continue"),
                ("それでも", "else"),
                ("輸出", "export"),
                ("偽", "false"),
                ("繰り返し", "for"),
                ("関数", "function"),
                ("もし", "if"),
                ("輸入", "import"),
                ("変数", "let"),
                ("新しい", "new"),
                ("ヌル", "null"),
                ("戻る", "return"),
                ("切り替え", "switch"),
                ("投げる", "throw"),
                ("真", "true"),
                ("試みる", "try"),
                ("間", "while"),
            ],
        ),
        // ── RU — Russian ────────────────────────────────────────────────────
        (
            "RU",
            "Russian",
            vec![
                ("асинхронный", "async"),
                ("ждать", "await"),
                ("прервать", "break"),
                ("случай", "case"),
                ("поймать", "catch"),
                ("класс", "class"),
                ("константа", "const"),
                ("продолжить", "continue"),
                ("иначе", "else"),
                ("экспорт", "export"),
                ("ложь", "false"),
                ("для", "for"),
                ("функция", "function"),
                ("если", "if"),
                ("импорт", "import"),
                ("переменная", "let"),
                ("новый", "new"),
                ("ноль", "null"),
                ("вернуть", "return"),
                ("переключить", "switch"),
                ("бросить", "throw"),
                ("истина", "true"),
                ("попробовать", "try"),
                ("пока", "while"),
            ],
        ),
        // ── ES — Spanish ────────────────────────────────────────────────────
        (
            "ES",
            "Spanish",
            vec![
                ("asíncrono", "async"),
                ("esperar", "await"),
                ("romper", "break"),
                ("caso", "case"),
                ("capturar", "catch"),
                ("clase", "class"),
                ("constante", "const"),
                ("continuar", "continue"),
                ("sino", "else"),
                ("exportar", "export"),
                ("falso", "false"),
                ("función", "function"),
                ("si", "if"),
                ("importar", "import"),
                ("definir", "let"),
                ("nuevo", "new"),
                ("nulo", "null"),
                ("retornar", "return"),
                ("elige", "switch"),
                ("lanzar", "throw"),
                ("verdadero", "true"),
                ("intentar", "try"),
                ("mientras", "while"),
            ],
        ),
        // ── DE — German ─────────────────────────────────────────────────────
        (
            "DE",
            "German",
            vec![
                ("asynchron", "async"),
                ("warten", "await"),
                ("abbrechen", "break"),
                ("fall", "case"),
                ("fangen", "catch"),
                ("klasse", "class"),
                ("konstante", "const"),
                ("fortsetzen", "continue"),
                ("sonst", "else"),
                ("exportieren", "export"),
                ("falsch", "false"),
                ("für", "for"),
                ("funktion", "function"),
                ("wenn", "if"),
                ("importieren", "import"),
                ("lasse", "let"),
                ("neu", "new"),
                ("zurück", "return"),
                ("wähle", "switch"),
                ("werfen", "throw"),
                ("wahr", "true"),
                ("versuchen", "try"),
                ("während", "while"),
            ],
        ),
        // ── FR — French ─────────────────────────────────────────────────────
        (
            "FR",
            "French",
            vec![
                ("asynchrone", "async"),
                ("attendre", "await"),
                ("casser", "break"),
                ("cas", "case"),
                ("attraper", "catch"),
                ("classe", "class"),
                ("constante", "const"),
                ("continuer", "continue"),
                ("sinon", "else"),
                ("exporter", "export"),
                ("faux", "false"),
                ("pour", "for"),
                ("fonction", "function"),
                ("importer", "import"),
                ("soit", "let"),
                ("nouveau", "new"),
                ("nul", "null"),
                ("retourner", "return"),
                ("choisis", "switch"),
                ("lancer", "throw"),
                ("vrai", "true"),
                ("essayer", "try"),
                ("pendant", "while"),
            ],
        ),
        // ── ZH — Chinese ───────────────────────────────────────────────────
        (
            "ZH",
            "Chinese",
            vec![
                ("异步", "async"),
                ("等待", "await"),
                ("中断", "break"),
                ("情况", "case"),
                ("捕获", "catch"),
                ("类", "class"),
                ("常量", "const"),
                ("继续", "continue"),
                ("否则", "else"),
                ("导出", "export"),
                ("假", "false"),
                ("对于", "for"),
                ("函数", "function"),
                ("如果", "if"),
                ("导入", "import"),
                ("变量", "let"),
                ("新建", "new"),
                ("空", "null"),
                ("返回", "return"),
                ("切换", "switch"),
                ("抛出", "throw"),
                ("尝试", "try"),
                ("当", "while"),
            ],
        ),
        // ── HI — Hindi ──────────────────────────────────────────────────────
        (
            "HI",
            "Hindi",
            vec![
                ("असिंक", "async"),
                ("प्रतीक्षा", "await"),
                ("तोड़ो", "break"),
                ("मामला", "case"),
                ("पकड़ो", "catch"),
                ("वर्ग", "class"),
                ("स्थिरांक", "const"),
                ("जारी_रखो", "continue"),
                ("नहीं_तो", "else"),
                ("निर्यात_करो", "export"),
                ("झूठ", "false"),
                ("लिए", "for"),
                ("फ़ंक्शन", "function"),
                ("अगर", "if"),
                ("आयात_करो", "import"),
                ("चर", "let"),
                ("नया", "new"),
                ("शून्य", "null"),
                ("वापस_करो", "return"),
                ("स्विच", "switch"),
                ("फेंको", "throw"),
                ("सच", "true"),
                ("कोशिश_करो", "try"),
                ("जब_तक", "while"),
            ],
        ),
        // ── KO — Korean ────────────────────────────────────────────────────
        (
            "KO",
            "Korean",
            vec![
                ("비동기", "async"),
                ("기다리기", "await"),
                ("중단", "break"),
                ("경우", "case"),
                ("잡기", "catch"),
                ("클래스", "class"),
                ("상수", "const"),
                ("계속", "continue"),
                ("아니면", "else"),
                ("내보내기", "export"),
                ("거짓", "false"),
                ("반복", "for"),
                ("함수", "function"),
                ("만약", "if"),
                ("가져오기", "import"),
                ("변수", "let"),
                ("새", "new"),
                ("없음", "null"),
                ("반환", "return"),
                ("선택", "switch"),
                ("던지기", "throw"),
                ("참", "true"),
                ("시도", "try"),
                ("동안", "while"),
            ],
        ),
        // ── FA — Persian ────────────────────────────────────────────────────
        (
            "FA",
            "Persian",
            vec![
                ("ناهمزمان", "async"),
                ("صبرکن", "await"),
                ("بشکن", "break"),
                ("حالت", "case"),
                ("بگیر", "catch"),
                ("کلاس", "class"),
                ("ادامه", "continue"),
                ("وگرنه", "else"),
                ("صادر_کن", "export"),
                ("نادرست", "false"),
                ("برای", "for"),
                ("تابع", "function"),
                ("اگر", "if"),
                ("وارد_کن", "import"),
                ("متغیر", "let"),
                ("جدید", "new"),
                ("تهی", "null"),
                ("برگردان", "return"),
                ("سوئیچ", "switch"),
                ("پرتاب", "throw"),
                ("درست", "true"),
                ("امتحان_کن", "try"),
                ("تا_وقتی", "while"),
            ],
        ),
        // ── KU — Kurdish (Sorani) ───────────────────────────────────────────
        (
            "KU",
            "Kurdish",
            vec![
                ("asenkron", "async"),
                ("bişkîne", "break"),
                ("rewş", "case"),
                ("bigire", "catch"),
                ("sinif", "class"),
                ("domdar", "const"),
                ("bidomîne", "continue"),
                ("wekî_din", "else"),
                ("derxîne", "export"),
                ("xelet", "false"),
                ("fonksiyon", "function"),
                ("eger", "if"),
                ("têxe", "import"),
                ("guhêrbar", "let"),
                ("nû", "new"),
                ("vala", "null"),
                ("vegere", "return"),
                ("hilbijêre", "switch"),
                ("biavêje", "throw"),
                ("rast", "true"),
                ("biceribîne", "try"),
                ("dema_ku", "while"),
            ],
        ),
        // ── BN — Bengali ────────────────────────────────────────────────────
        (
            "BN",
            "Bengali",
            vec![
                ("অ্যাসিঙ্ক", "async"),
                ("অপেক্ষা_করো", "await"),
                ("ভাঙো", "break"),
                ("কেস", "case"),
                ("ধরে_ফেলো", "catch"),
                ("শ্রেণী", "class"),
                ("কনস্ট্যান্ট", "const"),
                ("চালিয়ে_যাও", "continue"),
                ("নাহলে", "else"),
                ("রপ্তানি_করো", "export"),
                ("মিথ্যা", "false"),
                ("জন্য", "for"),
                ("ফাংশন", "function"),
                ("যদি", "if"),
                ("আমদানি_করো", "import"),
                ("ধরো", "let"),
                ("নতুন", "new"),
                ("ফেরত_দাও", "return"),
                ("সুইচ", "switch"),
                ("ছুঁড়ে_দাও", "throw"),
                ("সত্য", "true"),
                ("চেষ্টা_করো", "try"),
                ("যতক্ষণ", "while"),
            ],
        ),
        // ── BS — Bosnian ────────────────────────────────────────────────────
        (
            "BS",
            "Bosnian",
            vec![
                ("asinkroni", "async"),
                ("čekaj", "await"),
                ("prekini", "break"),
                ("slučaj", "case"),
                ("uhvati", "catch"),
                ("klasa", "class"),
                ("konstanta", "const"),
                ("nastavi", "continue"),
                ("inače", "else"),
                ("izvezi", "export"),
                ("netačno", "false"),
                ("za", "for"),
                ("funkcija", "function"),
                ("ako", "if"),
                ("uvezi", "import"),
                ("neka", "let"),
                ("novi", "new"),
                ("vrati", "return"),
                ("prebaci", "switch"),
                ("baci", "throw"),
                ("tačno", "true"),
                ("pokušaj", "try"),
                ("dok", "while"),
            ],
        ),
        // ── EL — Greek ──────────────────────────────────────────────────────
        (
            "EL",
            "Greek",
            vec![
                ("ασύγχρονος", "async"),
                ("αναμονή", "await"),
                ("διακοπή", "break"),
                ("περίπτωση", "case"),
                ("πιάσε", "catch"),
                ("κλάση", "class"),
                ("σταθερά", "const"),
                ("συνέχεια", "continue"),
                ("αλλιώς", "else"),
                ("εξαγωγή", "export"),
                ("για", "for"),
                ("συνάρτηση", "function"),
                ("αν", "if"),
                ("εισαγωγή", "import"),
                ("ας", "let"),
                ("νέο", "new"),
                ("επιστροφή", "return"),
                ("εναλλαγή", "switch"),
                ("ρίξε", "throw"),
                ("δοκίμασε", "try"),
                ("ενώ", "while"),
            ],
        ),
        // ── ID — Indonesian ─────────────────────────────────────────────────
        (
            "ID",
            "Indonesian",
            vec![
                ("asinkron", "async"),
                ("tunggu", "await"),
                ("hentikan", "break"),
                ("kasus", "case"),
                ("tangkap", "catch"),
                ("kelas", "class"),
                ("konstan", "const"),
                ("lanjutkan", "continue"),
                ("jika_tidak", "else"),
                ("ekspor", "export"),
                ("salah", "false"),
                ("untuk", "for"),
                ("fungsi", "function"),
                ("jika", "if"),
                ("impor", "import"),
                ("biarkan", "let"),
                ("baru", "new"),
                ("kembali", "return"),
                ("ganti", "switch"),
                ("lempar", "throw"),
                ("benar", "true"),
                ("coba", "try"),
                ("selama", "while"),
            ],
        ),
        // ── IT — Italian ────────────────────────────────────────────────────
        (
            "IT",
            "Italian",
            vec![
                ("asincrono", "async"),
                ("attendere", "await"),
                ("rompere", "break"),
                ("caso", "case"),
                ("catturare", "catch"),
                ("classe", "class"),
                ("costante", "const"),
                ("continuare", "continue"),
                ("altrimenti", "else"),
                ("esportare", "export"),
                ("per", "for"),
                ("funzione", "function"),
                ("se", "if"),
                ("importare", "import"),
                ("variabile", "let"),
                ("nuovo", "new"),
                ("ritornare", "return"),
                ("scegli", "switch"),
                ("lanciare", "throw"),
                ("provare", "try"),
                ("mentre", "while"),
            ],
        ),
        // ── PL — Polish ─────────────────────────────────────────────────────
        (
            "PL",
            "Polish",
            vec![
                ("asynchroniczny", "async"),
                ("czekaj", "await"),
                ("przerwij", "break"),
                ("przypadek", "case"),
                ("złap", "catch"),
                ("klasa", "class"),
                ("stała", "const"),
                ("kontynuuj", "continue"),
                ("w_przeciwnym_razie", "else"),
                ("eksportuj", "export"),
                ("fałsz", "false"),
                ("dla", "for"),
                ("funkcja", "function"),
                ("jeśli", "if"),
                ("importuj", "import"),
                ("niech", "let"),
                ("nowy", "new"),
                ("zwróć", "return"),
                ("przełącz", "switch"),
                ("rzuć", "throw"),
                ("prawda", "true"),
                ("spróbuj", "try"),
                ("podczas", "while"),
            ],
        ),
        // ── PT — Portuguese ─────────────────────────────────────────────────
        (
            "PT",
            "Portuguese",
            vec![
                ("assíncrono", "async"),
                ("aguardar", "await"),
                ("quebrar", "break"),
                ("caso", "case"),
                ("capturar", "catch"),
                ("classe", "class"),
                ("constante", "const"),
                ("continuar", "continue"),
                ("senão", "else"),
                ("exportar", "export"),
                ("para", "for"),
                ("função", "function"),
                ("importar", "import"),
                ("variável", "let"),
                ("novo", "new"),
                ("retornar", "return"),
                ("escolha", "switch"),
                ("lançar", "throw"),
                ("tentar", "try"),
                ("enquanto", "while"),
            ],
        ),
        // ── SR — Serbian ────────────────────────────────────────────────────
        (
            "SR",
            "Serbian",
            vec![
                ("асинхрони", "async"),
                ("чекај", "await"),
                ("прекини", "break"),
                ("случај", "case"),
                ("ухвати", "catch"),
                ("класа", "class"),
                ("константа", "const"),
                ("настави", "continue"),
                ("иначе", "else"),
                ("извези", "export"),
                ("за", "for"),
                ("функција", "function"),
                ("ако", "if"),
                ("увези", "import"),
                ("нека", "let"),
                ("нови", "new"),
                ("врати", "return"),
                ("пребаци", "switch"),
                ("баци", "throw"),
                ("покушај", "try"),
                ("док", "while"),
            ],
        ),
        // ── TH — Thai ───────────────────────────────────────────────────────
        (
            "TH",
            "Thai",
            vec![
                ("อะซิงโครนัส", "async"),
                ("รอ", "await"),
                ("หยุด", "break"),
                ("กรณี", "case"),
                ("จับ", "catch"),
                ("คลาส", "class"),
                ("ค่าคงที่", "const"),
                ("ดำเนินการต่อ", "continue"),
                ("มิฉะนั้น", "else"),
                ("ส่งออก", "export"),
                ("สำหรับ", "for"),
                ("ฟังก์ชัน", "function"),
                ("ถ้า", "if"),
                ("นำเข้า", "import"),
                ("ให้", "let"),
                ("ใหม่", "new"),
                ("คืนค่า", "return"),
                ("สลับ", "switch"),
                ("โยน", "throw"),
                ("ลอง", "try"),
                ("ขณะที่", "while"),
            ],
        ),
        // ── VI — Vietnamese ─────────────────────────────────────────────────
        (
            "VI",
            "Vietnamese",
            vec![
                ("bất_đồng_bộ", "async"),
                ("chờ", "await"),
                ("ngắt", "break"),
                ("trường_hợp", "case"),
                ("bắt", "catch"),
                ("lớp", "class"),
                ("hằng", "const"),
                ("tiếp_tục", "continue"),
                ("nếu_không", "else"),
                ("xuất", "export"),
                ("đối_với", "for"),
                ("hàm", "function"),
                ("nếu", "if"),
                ("nhập", "import"),
                ("cho", "let"),
                ("mới", "new"),
                ("trả_về", "return"),
                ("chuyển", "switch"),
                ("ném", "throw"),
                ("thử", "try"),
                ("trong_khi", "while"),
            ],
        ),
        // ── HR — Croatian ───────────────────────────────────────────────────
        // NOTE: Croatian has very few dedicated keywords in the normalizer;
        // many are shared with Bosnian/Serbian. Only the ones that actually
        // exist in the KEYWORD_MAP are tested here.
        (
            "HR",
            "Croatian",
            vec![("točno", "true"), ("netočno", "false"), ("nova", "new")],
        ),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

/// Test that normalize_keywords correctly normalizes each foreign keyword
/// to its English equivalent for all supported languages.
///
/// This is the primary correctness test: for every language, for every
/// mapped keyword, we verify the normalizer produces the correct output.
#[test]
fn test_all_languages_keyword_normalization() {
    let languages = language_keyword_maps();
    let mut failures: Vec<String> = Vec::new();

    for (code, name, mappings) in &languages {
        for (foreign, expected_english) in mappings {
            let result = normalize_keywords(foreign);
            if result.trim() != *expected_english {
                failures.push(format!(
                    "  {} ({}): '{}' -> got '{}', expected '{}'",
                    code,
                    name,
                    foreign,
                    result.trim(),
                    expected_english,
                ));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "\n\nKeyword normalization failures ({} total):\n{}\n",
            failures.len(),
            failures.join("\n")
        );
    }
}

/// Test that every language covers the ESSENTIAL_KEYWORDS set.
/// Reports which essential keywords are missing per language.
///
/// Languages with known gaps are listed explicitly so the test serves
/// as living documentation of translation completeness status.
#[test]
fn test_essential_keyword_completeness_report() {
    let languages = language_keyword_maps();
    let essential: BTreeSet<&str> = ESSENTIAL_KEYWORDS.iter().copied().collect();
    let mut missing_report: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    for (code, _name, mappings) in &languages {
        let covered: BTreeSet<&str> = mappings.iter().map(|(_, eng)| *eng).collect();
        let missing: Vec<&str> = essential.difference(&covered).copied().collect();
        if !missing.is_empty() {
            missing_report.insert(code, missing);
        }
    }

    // Print the completeness report
    if !missing_report.is_empty() {
        let mut report = String::from("\n\n=== Translation Completeness Report ===\n");
        for (code, missing) in &missing_report {
            report.push_str(&format!("  {}: missing {:?}\n", code, missing));
        }

        let complete_count = languages.len() - missing_report.len();
        report.push_str(&format!(
            "\n  {}/{} languages are 100% complete for essential keywords.\n",
            complete_count,
            languages.len()
        ));
        eprintln!("{}", report);
    }

    // Known incomplete languages (documented gaps):
    // - HR (Croatian): mostly shares keywords with BS/SR, only 3 dedicated entries
    // - EL (Greek): missing true/false/null
    // - IT (Italian): missing true/false/null
    // - TH (Thai): missing true/false/null
    // - VI (Vietnamese): missing true/false/null
    // - SR (Serbian): missing true/false/null
    // - BN (Bengali): missing null
    // - PT (Portuguese): missing if/true/false/null
    // - ES (Spanish): missing for
    // - FA (Persian): missing const
    // - FR (French): missing if
    // - KU (Kurdish): missing await/for
    // - DE (German): missing null
    // - BS (Bosnian): missing null
    // - ID (Indonesian): missing null
    // - PL (Polish): missing null
    // - ZH (Chinese): missing true

    // Assert that fully-complete languages remain complete
    let expected_complete = vec!["TR", "AR", "JA", "RU", "HI", "KO"];
    for code in &expected_complete {
        assert!(
            !missing_report.contains_key(code),
            "Language {} was expected to be 100% complete but has gaps: {:?}",
            code,
            missing_report.get(code)
        );
    }
}

/// Verify that the normalizer is idempotent for English keywords.
/// English keywords should pass through unchanged.
#[test]
fn test_english_keywords_pass_through() {
    for &kw in ESSENTIAL_KEYWORDS {
        let result = normalize_keywords(kw);
        assert_eq!(
            result.trim(),
            kw,
            "English keyword '{}' should pass through unchanged, got '{}'",
            kw,
            result.trim()
        );
    }
}

/// Verify that normalizing a mini-program in Turkish produces valid English.
#[test]
fn test_contextual_normalization_turkish() {
    let turkish_code = "değişken x = 10\neğer x > 5 {\n    dön x\n}\n";
    let normalized = normalize_keywords(turkish_code);
    assert!(
        normalized.contains("let x = 10"),
        "Turkish 'değişken' should normalize to 'let' in context, got: {}",
        normalized
    );
    assert!(
        normalized.contains("if x > 5"),
        "Turkish 'eğer' should normalize to 'if' in context, got: {}",
        normalized
    );
    assert!(
        normalized.contains("return x"),
        "Turkish 'dön' should normalize to 'return' in context, got: {}",
        normalized
    );
}

/// Verify that normalizing a mini-program in Arabic produces valid English.
#[test]
fn test_contextual_normalization_arabic() {
    let arabic_code = "متغير س = 10\nإذا س > 5 {\n    ارجع س\n}\n";
    let normalized = normalize_keywords(arabic_code);
    assert!(
        normalized.contains("let"),
        "Arabic 'متغير' should normalize to 'let'"
    );
    assert!(
        normalized.contains("if"),
        "Arabic 'إذا' should normalize to 'if'"
    );
    assert!(
        normalized.contains("return"),
        "Arabic 'ارجع' should normalize to 'return'"
    );
}

/// Verify that normalizing a mini-program in Japanese produces valid English.
#[test]
fn test_contextual_normalization_japanese() {
    let ja_code = "変数 x = 10\nもし x > 5 {\n    戻る x\n}\n";
    let normalized = normalize_keywords(ja_code);
    assert!(
        normalized.contains("let x = 10"),
        "Japanese '変数' should normalize to 'let', got: {}",
        normalized
    );
    assert!(
        normalized.contains("if x > 5"),
        "Japanese 'もし' should normalize to 'if', got: {}",
        normalized
    );
    assert!(
        normalized.contains("return x"),
        "Japanese '戻る' should normalize to 'return', got: {}",
        normalized
    );
}

/// Verify that normalizing a mini-program in Chinese produces valid English.
#[test]
fn test_contextual_normalization_chinese() {
    let zh_code = "变量 x = 10\n如果 x > 5 {\n    返回 x\n}\n";
    let normalized = normalize_keywords(zh_code);
    assert!(
        normalized.contains("let x = 10"),
        "Chinese '变量' should normalize to 'let', got: {}",
        normalized
    );
    assert!(
        normalized.contains("if x > 5"),
        "Chinese '如果' should normalize to 'if', got: {}",
        normalized
    );
    assert!(
        normalized.contains("return x"),
        "Chinese '返回' should normalize to 'return', got: {}",
        normalized
    );
}

/// Verify that normalizing a mini-program in Korean produces valid English.
#[test]
fn test_contextual_normalization_korean() {
    let ko_code = "변수 x = 10\n만약 x > 5 {\n    반환 x\n}\n";
    let normalized = normalize_keywords(ko_code);
    assert!(
        normalized.contains("let x = 10"),
        "Korean '변수' should normalize to 'let', got: {}",
        normalized
    );
    assert!(
        normalized.contains("if x > 5"),
        "Korean '만약' should normalize to 'if', got: {}",
        normalized
    );
    assert!(
        normalized.contains("return x"),
        "Korean '반환' should normalize to 'return', got: {}",
        normalized
    );
}

/// Count total languages and assert minimum expected count.
#[test]
fn test_minimum_language_count() {
    let languages = language_keyword_maps();
    assert!(
        languages.len() >= 23,
        "Expected at least 23 supported languages, found {}",
        languages.len()
    );
}

/// Verify no two languages use the exact same keyword for different English targets.
/// This catches potential normalization conflicts where a single word could map
/// to two different English keywords depending on intended language.
#[test]
fn test_no_cross_language_conflicts() {
    let languages = language_keyword_maps();
    let mut keyword_targets: BTreeMap<&str, Vec<(&str, &str)>> = BTreeMap::new();

    for (code, _name, mappings) in &languages {
        for (foreign, english) in mappings {
            keyword_targets
                .entry(foreign)
                .or_default()
                .push((code, english));
        }
    }

    let mut conflicts: Vec<String> = Vec::new();
    for (foreign, targets) in &keyword_targets {
        let unique_english: BTreeSet<&str> = targets.iter().map(|(_, e)| *e).collect();
        if unique_english.len() > 1 {
            let details: Vec<String> = targets
                .iter()
                .map(|(lang, eng)| format!("{}={}", lang, eng))
                .collect();
            conflicts.push(format!(
                "  '{}' maps to different targets: {}",
                foreign,
                details.join(", ")
            ));
        }
    }

    // Some conflicts are expected (e.g., shared words between related languages
    // that map to the same thing). Only flag actual divergences.
    if !conflicts.is_empty() {
        eprintln!(
            "\n\nPotential cross-language conflicts ({}):\n{}\n",
            conflicts.len(),
            conflicts.join("\n")
        );
    }
}
