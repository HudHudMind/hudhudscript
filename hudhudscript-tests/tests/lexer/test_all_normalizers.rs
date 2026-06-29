//! Comprehensive keyword normalizer tests for all 24+ supported languages.
//! Tests actual keyword mappings from the normalizer using normalize_keywords().
//!
//! Kural 8: Non-Latin script languages use their native writing system (NO romanization).

use hudhudscript_lexer::normalize_keywords;

/// Helper: verifies that normalizing `src` produces output containing `expected_english`.
fn assert_normalizes(src: &str, expected: &str) {
    let result = normalize_keywords(src);
    assert!(
        result.contains(expected),
        "normalize_keywords({src:?}) expected to contain '{expected}', got: '{result}'"
    );
}

/// Helper: verifies that `keyword` normalizes to `expected`.
fn assert_kw_normalizes(keyword: &str, expected: &str) {
    // Build a simple statement: "var x = <keyword>" -> "var x = <english>"
    let src = format!("değişken x = {}", keyword);
    let result = normalize_keywords(&src);
    assert!(
        result.contains(expected),
        "normalize_keywords({src:?}) expected to contain '{expected}', got: '{result}'"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// TR — Türkçe
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn tr_if() {
    assert_normalizes("eğer x > 0 { }", "if");
}
#[test]
fn tr_while() {
    assert_normalizes("iken x < 10 { }", "while");
}
#[test]
fn tr_function() {
    assert_normalizes("işlev topla() { }", "function");
}
#[test]
fn tr_class() {
    assert_normalizes("sınıf Nokta { }", "class");
}
#[test]
fn tr_let() {
    assert_normalizes("değişken x = 1", "let");
}
#[test]
fn tr_const() {
    assert_normalizes("sabit PI = 3.14", "const");
}
#[test]
fn tr_return() {
    assert_normalizes("dön 42", "return");
}
#[test]
fn tr_async() {
    assert_normalizes("eşzamansız işlev f() { }", "async");
}
#[test]
fn tr_await() {
    assert_normalizes("değişken r = bekle f()", "await");
}
#[test]
fn tr_import() {
    assert_normalizes("içe_aktar { x } den 'mod'", "import");
}
#[test]
fn tr_export() {
    assert_normalizes("dışa_aktar işlev f() { }", "export");
}
#[test]
fn tr_try() {
    assert_normalizes("dene { }", "try");
}
#[test]
fn tr_catch() {
    assert_normalizes("yakala e { }", "catch");
}
#[test]
fn tr_throw() {
    assert_normalizes("fırlat Hata('!')", "throw");
}
#[test]
fn tr_break() {
    assert_normalizes("iken doğru { kır }", "break");
}
#[test]
fn tr_continue() {
    assert_normalizes("iken doğru { devam }", "continue");
}
#[test]
fn tr_true() {
    assert_kw_normalizes("doğru", "true");
}
#[test]
fn tr_false() {
    assert_kw_normalizes("yanlış", "false");
}
#[test]
fn tr_switch() {
    assert_normalizes("seç x { }", "switch");
}
#[test]
fn tr_for() {
    assert_normalizes("döngü i 0..5 { }", "for");
}
#[test]
fn tr_new() {
    assert_normalizes("değişken n = yeni Nokta()", "new");
}
#[test]
fn tr_rule() {
    assert_normalizes("kural Ad { }", "rule");
}
#[test]
fn tr_agent() {
    assert_normalizes("ajan a = yeni ajan()", "agent");
}
#[test]
fn tr_protocol() {
    assert_normalizes("protokol P { }", "protocol");
}
#[test]
fn tr_event() {
    assert_normalizes("olay basildi { }", "event");
}
#[test]
fn tr_role() {
    assert_normalizes("rol r = yönetici", "role");
}
#[test]
fn tr_use() {
    assert_normalizes("kullan std", "use");
}
#[test]
fn tr_provider() {
    assert_normalizes("sağlayıcı p = yeni sağlayıcı()", "provider");
}
#[test]
fn tr_entity() {
    assert_normalizes("varlık V { }", "entity");
}
#[test]
fn tr_swarm() {
    assert_normalizes("sürü S { }", "swarm");
}
#[test]
fn tr_resource() {
    assert_normalizes("kaynak R { }", "resource");
}
// MANTIKSAL OPERATÖRLER
#[test]
fn tr_and() {
    assert_eq!(normalize_keywords("a ve b"), "a && b");
}
#[test]
fn tr_or() {
    assert_eq!(normalize_keywords("a veya b"), "a || b");
}
#[test]
fn tr_not() {
    assert_eq!(normalize_keywords("değil x"), "! x");
}
#[test]
fn tr_not2() {
    assert_eq!(normalize_keywords("değildir x"), "! x");
}
#[test]
fn tr_eq() {
    assert_eq!(normalize_keywords("a eşit b"), "a == b");
}
#[test]
fn tr_eq2() {
    assert_eq!(normalize_keywords("a eşittir b"), "a == b");
}
#[test]
fn tr_neq() {
    assert_eq!(normalize_keywords("a eşit değil b"), "a != b");
}
#[test]
fn tr_neq2() {
    assert_eq!(normalize_keywords("a eşit değildir b"), "a != b");
}
#[test]
fn tr_boundary() {
    assert_eq!(normalize_keywords("seviye"), "seviye");
}

// ═══════════════════════════════════════════════════════════════════════════
// JA — 日本語 (Kural 8: Romanize YASAK)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ja_if() {
    assert_normalizes("もし x > 0 { }", "if");
}
#[test]
fn ja_while() {
    assert_normalizes("間 x < 10 { }", "while");
}
#[test]
fn ja_function() {
    assert_normalizes("関数 合計() { }", "function");
}
#[test]
fn ja_class() {
    assert_normalizes("クラス 点 { }", "class");
}
#[test]
fn ja_let() {
    assert_normalizes("変数 x = 1", "let");
}
#[test]
fn ja_const() {
    assert_normalizes("定数 PI = 3.14", "const");
}
#[test]
fn ja_return() {
    assert_normalizes("戻る 42", "return");
}
#[test]
fn ja_async() {
    assert_normalizes("非同期 関数 f() { }", "async");
}
#[test]
fn ja_await() {
    assert_normalizes("変数 r = 待つ f()", "await");
}
#[test]
fn ja_import() {
    assert_normalizes("輸入 { x } から 'mod'", "import");
}
#[test]
fn ja_export() {
    assert_normalizes("輸出 関数 f() { }", "export");
}
#[test]
fn ja_try() {
    assert_normalizes("試みる { }", "try");
}
#[test]
fn ja_catch() {
    assert_normalizes("捕まえる e { }", "catch");
}
#[test]
fn ja_throw() {
    assert_normalizes("投げる エラー('!')", "throw");
}
#[test]
fn ja_break() {
    assert_normalizes("間 新しい { 破る }", "break");
}
#[test]
fn ja_continue() {
    assert_normalizes("間 新しい { 続ける }", "continue");
}
#[test]
fn ja_new() {
    assert_normalizes("変数 n = 新しい 点()", "new");
}
#[test]
fn ja_switch() {
    assert_normalizes("切り替え x { }", "switch");
}
#[test]
fn ja_agent() {
    assert_normalizes("エージェント a { }", "agent");
}
#[test]
fn ja_rule() {
    assert_normalizes("ルール 名 { }", "rule");
}
#[test]
fn ja_protocol() {
    assert_normalizes("プロトコル P { }", "protocol");
}
#[test]
fn ja_event() {
    assert_normalizes("イベント e { }", "event");
}
#[test]
fn ja_server() {
    assert_normalizes("サーバー s { }", "server");
}
#[test]
fn ja_use() {
    assert_normalizes("使う std", "use");
}
#[test]
fn ja_provider() {
    assert_normalizes("プロバイダー p { }", "provider");
}

// ═══════════════════════════════════════════════════════════════════════════
// AR — العربية (RTL, Kural 8: Romanize YASAK)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ar_if() {
    assert_normalizes("إذا x > 0 { }", "if");
}
#[test]
fn ar_while() {
    assert_normalizes("بينما x < 10 { }", "while");
}
#[test]
fn ar_function() {
    assert_normalizes("دالة مجموع() { }", "function");
}
#[test]
fn ar_class() {
    assert_normalizes("فئة نقطة { }", "class");
}
#[test]
fn ar_let() {
    assert_normalizes("متغير x = 1", "let");
}
#[test]
fn ar_const() {
    assert_normalizes("ثابت PI = 3.14", "const");
}
#[test]
fn ar_return() {
    assert_normalizes("ارجع 42", "return");
}
#[test]
fn ar_async() {
    assert_normalizes("غير_متزامن دالة f() { }", "async");
}
#[test]
fn ar_await() {
    assert_normalizes("متغير r = انتظر f()", "await");
}
#[test]
fn ar_import() {
    assert_normalizes("استورد { x } من 'mod'", "import");
}
#[test]
fn ar_export() {
    assert_normalizes("صدّر دالة f() { }", "export");
}
#[test]
fn ar_try() {
    assert_normalizes("حاول { }", "try");
}
#[test]
fn ar_catch() {
    assert_normalizes("امسك e { }", "catch");
}
#[test]
fn ar_throw() {
    assert_normalizes("ارمي خطأ('!')", "throw");
}
#[test]
fn ar_break() {
    assert_normalizes("طالما صحيح { اكسر }", "break");
}
#[test]
fn ar_continue() {
    assert_normalizes("طالما صحيح { استمر }", "continue");
}
#[test]
fn ar_new() {
    assert_normalizes("متغير n = جديد نقطة()", "new");
}
#[test]
fn ar_switch() {
    assert_normalizes("انتخاب x { }", "switch");
}
#[test]
fn ar_agent() {
    assert_normalizes("وكيل a { }", "agent");
}
#[test]
fn ar_protocol() {
    assert_normalizes("بروتوكول P { }", "protocol");
}
#[test]
fn ar_event() {
    assert_normalizes("حدث e { }", "event");
}
#[test]
fn ar_use() {
    assert_normalizes("استخدم std", "use");
}

// ═══════════════════════════════════════════════════════════════════════════
// RU — Русский (Кириллица, Kural 8: Romanize YASAK)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ru_if() {
    assert_normalizes("если x > 0 { }", "if");
}
#[test]
fn ru_while() {
    assert_normalizes("пока x < 10 { }", "while");
}
#[test]
fn ru_function() {
    assert_normalizes("функция сумма() { }", "function");
}
#[test]
fn ru_class() {
    assert_normalizes("класс Точка { }", "class");
}
#[test]
fn ru_let() {
    assert_normalizes("переменная x = 1", "let");
}
#[test]
fn ru_const() {
    assert_normalizes("константа PI = 3.14", "const");
}
#[test]
fn ru_return() {
    assert_normalizes("вернуть 42", "return");
}
#[test]
fn ru_async() {
    assert_normalizes("асинхронный функция f() { }", "async");
}
#[test]
fn ru_await() {
    assert_normalizes("переменная r = ждать f()", "await");
}
#[test]
fn ru_import() {
    assert_normalizes("импорт { x } из 'mod'", "import");
}
#[test]
fn ru_try() {
    assert_normalizes("попробовать { }", "try");
}
#[test]
fn ru_catch() {
    assert_normalizes("поймать e { }", "catch");
}
#[test]
fn ru_throw() {
    assert_normalizes("бросить ошибка('!')", "throw");
}
#[test]
fn ru_break() {
    assert_normalizes("пока истина { прервать }", "break");
}
#[test]
fn ru_true() {
    assert_kw_normalizes("истина", "true");
}
#[test]
fn ru_false() {
    assert_kw_normalizes("ложь", "false");
}
#[test]
fn ru_agent() {
    assert_normalizes("агент a { }", "agent");
}
#[test]
fn ru_law() {
    assert_normalizes("закон L { }", "law");
}
#[test]
fn ru_for() {
    assert_normalizes("для i 0..5 { }", "for");
}

// ═══════════════════════════════════════════════════════════════════════════
// ZH — 中文 (Kural 8: Romanize YASAK)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn zh_if() {
    assert_normalizes("如果 x > 0 { }", "if");
}
#[test]
fn zh_while() {
    assert_normalizes("当 x < 10 { }", "while");
}
#[test]
fn zh_function() {
    assert_normalizes("函数 求和() { }", "function");
}
#[test]
fn zh_class() {
    assert_normalizes("类 点 { }", "class");
}
#[test]
fn zh_let() {
    assert_normalizes("变量 x = 1", "let");
}
#[test]
fn zh_const() {
    assert_normalizes("常量 PI = 3.14", "const");
}
#[test]
fn zh_return() {
    assert_normalizes("返回 42", "return");
}
#[test]
fn zh_async() {
    assert_normalizes("异步 函数 f() { }", "async");
}
#[test]
fn zh_import() {
    assert_normalizes("导入 { x } 从 'mod'", "import");
}
#[test]
fn zh_export() {
    assert_normalizes("导出 函数 f() { }", "export");
}
#[test]
fn zh_else() {
    assert_normalizes("如果 x > 0 { } 否则 { }", "else");
}
#[test]
fn zh_true() {
    assert_kw_normalizes("真", "true");
}
#[test]
fn zh_false() {
    assert_kw_normalizes("假", "false");
}
#[test]
fn zh_agent() {
    assert_normalizes("代理 a { }", "agent");
}
#[test]
fn zh_event() {
    assert_normalizes("事件 e { }", "event");
}
#[test]
fn zh_switch() {
    assert_normalizes("切换 x { }", "switch");
}
#[test]
fn zh_break() {
    assert_normalizes("当 真 { 中断 }", "break");
}
#[test]
fn zh_protocol() {
    assert_normalizes("协议 P { }", "protocol");
}

// ═══════════════════════════════════════════════════════════════════════════
// KO — 한국어 (한글, Kural 8: Romanize YASAK)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ko_if() {
    assert_normalizes("만약 x > 0 { }", "if");
}
#[test]
fn ko_while() {
    assert_normalizes("동안 x < 10 { }", "while");
}
#[test]
fn ko_function() {
    assert_normalizes("함수 합계() { }", "function");
}
#[test]
fn ko_class() {
    assert_normalizes("클래스 점 { }", "class");
}
#[test]
fn ko_let() {
    assert_normalizes("변수 x = 1", "let");
}
#[test]
fn ko_const() {
    assert_normalizes("상수 PI = 3.14", "const");
}
#[test]
fn ko_return() {
    assert_normalizes("반환 42", "return");
}
#[test]
fn ko_async() {
    assert_normalizes("비동기 함수 f() { }", "async");
}
#[test]
fn ko_await() {
    assert_normalizes("변수 r = 기다리기 f()", "await");
}
#[test]
fn ko_import() {
    assert_normalizes("가져오기 { x } 에서 'mod'", "import");
}
#[test]
fn ko_try() {
    assert_normalizes("시도 { }", "try");
}
#[test]
fn ko_catch() {
    assert_normalizes("잡기 e { }", "catch");
}
#[test]
fn ko_throw() {
    assert_normalizes("던지기 오류('!')", "throw");
}
#[test]
fn ko_break() {
    assert_normalizes("동안 참 { 중단 }", "break");
}
#[test]
fn ko_continue() {
    assert_normalizes("동안 참 { 계속 }", "continue");
}
#[test]
fn ko_true() {
    assert_kw_normalizes("참", "true");
}
#[test]
fn ko_false() {
    assert_kw_normalizes("거짓", "false");
}
#[test]
fn ko_new() {
    assert_normalizes("변수 n = 새 점()", "new");
}
#[test]
fn ko_agent() {
    assert_normalizes("에이전트 a { }", "agent");
}

// ═══════════════════════════════════════════════════════════════════════════
// HI — हिन्दी (Devanagari, Kural 8: Romanize YASAK)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn hi_if() {
    assert_normalizes("अगर x > 0 { }", "if");
}
#[test]
fn hi_while() {
    assert_normalizes("जब_तक x < 10 { }", "while");
}
#[test]
fn hi_function() {
    assert_normalizes("कार्यविधि योग() { }", "function");
}
#[test]
fn hi_let() {
    assert_normalizes("चर x = 1", "let");
}
#[test]
fn hi_const() {
    assert_normalizes("स्थिरांक PI = 3.14", "const");
}
#[test]
fn hi_return() {
    assert_normalizes("वापस_करो 42", "return");
}
#[test]
fn hi_true() {
    assert_kw_normalizes("सच", "true");
}
#[test]
fn hi_false() {
    assert_kw_normalizes("झूठ", "false");
}
#[test]
fn hi_agent() {
    assert_normalizes("एजेंट a { }", "agent");
}
#[test]
fn hi_event() {
    assert_normalizes("घटना e { }", "event");
}
#[test]
fn hi_try() {
    assert_normalizes("कोशिश_करो { }", "try");
}

// ═══════════════════════════════════════════════════════════════════════════
// BN — বাংলা (Bangla script, Kural 8: Romanize YASAK)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn bn_if() {
    assert_normalizes("যদি x > 0 { }", "if");
}
#[test]
fn bn_while() {
    assert_normalizes("যতক্ষণ x < 10 { }", "while");
}
#[test]
fn bn_function() {
    assert_normalizes("ফাংশন যোগ() { }", "function");
}
#[test]
fn bn_let() {
    assert_normalizes("চলক x = 1", "let");
}
#[test]
fn bn_return() {
    assert_normalizes("ফেরত_দাও 42", "return");
}
#[test]
fn bn_true() {
    assert_kw_normalizes("সত্য", "true");
}
#[test]
fn bn_false() {
    assert_kw_normalizes("মিথ্যা", "false");
}

// ═══════════════════════════════════════════════════════════════════════════
// FA — فارسی (RTL, Kural 8: Romanize YASAK)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fa_if() {
    assert_normalizes("اگر x > 0 { }", "if");
}
#[test]
fn fa_while() {
    assert_normalizes("تا_وقتی x < 10 { }", "while");
}
#[test]
fn fa_function() {
    assert_normalizes("تابع جمع() { }", "function");
}
#[test]
fn fa_let() {
    assert_normalizes("متغیر x = 1", "let");
}
#[test]
fn fa_const() {
    assert_normalizes("ثابت PI = 3.14", "const");
}
#[test]
fn fa_return() {
    assert_normalizes("برگردان 42", "return");
}

// ═══════════════════════════════════════════════════════════════════════════
// ES — Español (Latin script)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn es_if() {
    assert_normalizes("si x > 0 { }", "if");
}
#[test]
fn es_while() {
    assert_normalizes("mientras x < 10 { }", "while");
}
#[test]
fn es_function() {
    assert_normalizes("función suma() { }", "function");
}
#[test]
fn es_class() {
    assert_normalizes("clase Punto { }", "class");
}
#[test]
fn es_let() {
    assert_normalizes("variable x = 1", "let");
}
#[test]
fn es_const() {
    assert_normalizes("constante PI = 3.14", "const");
}
#[test]
fn es_return() {
    assert_normalizes("retornar 42", "return");
}
#[test]
fn es_import() {
    assert_normalizes("importar { x } de 'mod'", "import");
}
#[test]
fn es_export() {
    assert_normalizes("exportar función f() { }", "export");
}
#[test]
fn es_try() {
    assert_normalizes("intentar { }", "try");
}
#[test]
fn es_catch() {
    assert_normalizes("capturar e { }", "catch");
}
#[test]
fn es_throw() {
    assert_normalizes("lanzar Error('!')", "throw");
}
#[test]
fn es_true() {
    assert_kw_normalizes("verdadero", "true");
}
#[test]
fn es_false() {
    assert_kw_normalizes("falso", "false");
}
#[test]
fn es_agent() {
    assert_normalizes("agente a { }", "agent");
}
#[test]
fn es_break() {
    assert_normalizes("mientras verdadero { romper }", "break");
}

// ═══════════════════════════════════════════════════════════════════════════
// DE — Deutsch (Latin script)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn de_if() {
    assert_normalizes("wenn x > 0 { }", "if");
}
#[test]
fn de_while() {
    assert_normalizes("während x < 10 { }", "while");
}
#[test]
fn de_function() {
    assert_normalizes("funktion summe() { }", "function");
}
#[test]
fn de_class() {
    assert_normalizes("klasse Punkt { }", "class");
}
#[test]
fn de_let() {
    assert_normalizes("variable x = 1", "let");
}
#[test]
fn de_const() {
    assert_normalizes("konstante PI = 3.14", "const");
}
#[test]
fn de_return() {
    assert_normalizes("zurück 42", "return");
}
#[test]
fn de_import() {
    assert_normalizes("importieren { x } von 'mod'", "import");
}
#[test]
fn de_export() {
    assert_normalizes("exportieren funktion f() { }", "export");
}
#[test]
fn de_try() {
    assert_normalizes("versuchen { }", "try");
}
#[test]
fn de_catch() {
    assert_normalizes("fangen e { }", "catch");
}
#[test]
fn de_throw() {
    assert_normalizes("werfen Fehler('!')", "throw");
}
#[test]
fn de_break() {
    assert_normalizes("solange wahr { abbrechen }", "break");
}
#[test]
fn de_continue() {
    assert_normalizes("solange wahr { fortsetzen }", "continue");
}
#[test]
fn de_true() {
    assert_kw_normalizes("wahr", "true");
}
#[test]
fn de_false() {
    assert_kw_normalizes("falsch", "false");
}

// ═══════════════════════════════════════════════════════════════════════════
// FR — Français (Latin script)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fr_if() {
    assert_normalizes("si x > 0 { }", "if");
}
#[test]
fn fr_while() {
    assert_normalizes("pendant x < 10 { }", "while");
}
#[test]
fn fr_function() {
    assert_normalizes("fonction somme() { }", "function");
}
#[test]
fn fr_class() {
    assert_normalizes("classe Point { }", "class");
}
#[test]
fn fr_let() {
    assert_normalizes("variable x = 1", "let");
}
#[test]
fn fr_const() {
    assert_normalizes("constante PI = 3.14", "const");
}
#[test]
fn fr_return() {
    assert_normalizes("retourner 42", "return");
}
#[test]
fn fr_import() {
    assert_normalizes("importer { x } depuis 'mod'", "import");
}
#[test]
fn fr_export() {
    assert_normalizes("exporter fonction f() { }", "export");
}
#[test]
fn fr_try() {
    assert_normalizes("essayer { }", "try");
}
#[test]
fn fr_catch() {
    assert_normalizes("attraper e { }", "catch");
}
#[test]
fn fr_throw() {
    assert_normalizes("lancer Erreur('!')", "throw");
}
#[test]
fn fr_break() {
    assert_normalizes("tantque vrai { casser }", "break");
}
#[test]
fn fr_continue() {
    assert_normalizes("tantque vrai { continuer }", "continue");
}
#[test]
fn fr_true() {
    assert_kw_normalizes("vrai", "true");
}
#[test]
fn fr_false() {
    assert_kw_normalizes("faux", "false");
}

// ═══════════════════════════════════════════════════════════════════════════
// EL — Ελληνικά (Greek script, Kural 8: Romanize YASAK)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn el_if() {
    assert_normalizes("αν x > 0 { }", "if");
}
#[test]
fn el_while() {
    assert_normalizes("ενώ x < 10 { }", "while");
}
#[test]
fn el_function() {
    assert_normalizes("συνάρτηση άθροισμα() { }", "function");
}
#[test]
fn el_class() {
    assert_normalizes("κλάση Σημείο { }", "class");
}
#[test]
fn el_let() {
    assert_normalizes("μεταβλητή x = 1", "let");
}

// ═══════════════════════════════════════════════════════════════════════════
// SR — Српски (Cyrillic, Kural 8: Romanize YASAK)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn sr_if() {
    assert_normalizes("ако x > 0 { }", "if");
}
#[test]
fn sr_while() {
    assert_normalizes("док x < 10 { }", "while");
}
#[test]
fn sr_let() {
    assert_normalizes("променљива x = 1", "let");
}

// ═══════════════════════════════════════════════════════════════════════════
// IT — Italiano (Latin script)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn it_if() {
    assert_normalizes("se x > 0 { }", "if");
}
#[test]
fn it_while() {
    assert_normalizes("mentre x < 10 { }", "while");
}
#[test]
fn it_function() {
    assert_normalizes("funzione somma() { }", "function");
}
#[test]
fn it_class() {
    assert_normalizes("classe Punto { }", "class");
}
#[test]
fn it_let() {
    assert_normalizes("variabile x = 1", "let");
}

// ═══════════════════════════════════════════════════════════════════════════
// PT — Português (Latin script)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn pt_if() {
    assert_normalizes("se x > 0 { }", "if");
}
#[test]
fn pt_while() {
    assert_normalizes("enquanto x < 10 { }", "while");
}
#[test]
fn pt_function() {
    assert_normalizes("função soma() { }", "function");
}
#[test]
fn pt_class() {
    assert_normalizes("classe Ponto { }", "class");
}
#[test]
fn pt_let() {
    assert_normalizes("variável x = 1", "let");
}

// ═══════════════════════════════════════════════════════════════════════════
// PL — Polski (Latin script)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn pl_if() {
    assert_normalizes("jeśli x > 0 { }", "if");
}
#[test]
fn pl_while() {
    assert_normalizes("dopóki x < 10 { }", "while");
}
#[test]
fn pl_function() {
    assert_normalizes("funkcja suma() { }", "function");
}
#[test]
fn pl_class() {
    assert_normalizes("klasa Punkt { }", "class");
}
#[test]
fn pl_let() {
    assert_normalizes("zmienna x = 1", "let");
}
#[test]
fn pl_true() {
    assert_kw_normalizes("prawda", "true");
}
#[test]
fn pl_false() {
    assert_kw_normalizes("fałsz", "false");
}

// ═══════════════════════════════════════════════════════════════════════════
// ID — Bahasa Indonesia (Latin script)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn id_if() {
    assert_normalizes("jika x > 0 { }", "if");
}
#[test]
fn id_while() {
    assert_normalizes("selama x < 10 { }", "while");
}
#[test]
fn id_function() {
    assert_normalizes("fungsi jumlah() { }", "function");
}
#[test]
fn id_class() {
    assert_normalizes("kelas Titik { }", "class");
}
#[test]
fn id_let() {
    assert_normalizes("variabel x = 1", "let");
}
#[test]
fn id_true() {
    assert_kw_normalizes("benar", "true");
}
#[test]
fn id_false() {
    assert_kw_normalizes("salah", "false");
}

// ═══════════════════════════════════════════════════════════════════════════
// VI — Tiếng Việt (Latin script)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn vi_if() {
    assert_normalizes("nếu x > 0 { }", "if");
}
#[test]
fn vi_while() {
    assert_normalizes("trong_khi x < 10 { }", "while");
}
#[test]
fn vi_function() {
    assert_normalizes("hàm tổng() { }", "function");
}
#[test]
fn vi_class() {
    assert_normalizes("lớp Điểm { }", "class");
}
#[test]
fn vi_let() {
    assert_normalizes("biến x = 1", "let");
}

// ═══════════════════════════════════════════════════════════════════════════
// TH — ไทย (Thai script, Kural 8: Romanize YASAK)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn th_if() {
    assert_normalizes("ถ้า x > 0 { }", "if");
}
#[test]
fn th_while() {
    assert_normalizes("ขณะที่ x < 10 { }", "while");
}
#[test]
fn th_function() {
    assert_normalizes("ฟังก์ชัน ผลรวม() { }", "function");
}
#[test]
fn th_let() {
    assert_normalizes("ตัวแปร x = 1", "let");
}

// ═══════════════════════════════════════════════════════════════════════════
// KU — Kurdî (Latin script for Kurmanji, Kural 8)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ku_if() {
    assert_normalizes("eger x > 0 { }", "if");
}
#[test]
fn ku_while() {
    assert_normalizes("dema_ku x < 10 { }", "while");
}
#[test]
fn ku_function() {
    assert_normalizes("fonksiyon kom() { }", "function");
}
#[test]
fn ku_class() {
    assert_normalizes("sinif Xal { }", "class");
}

// ═══════════════════════════════════════════════════════════════════════════
// BS — Bosanski (Latin script)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn bs_if() {
    assert_normalizes("ako x > 0 { }", "if");
}

// ═══════════════════════════════════════════════════════════════════════════
// HR — Hrvatski (Latin script)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn hr_if() {
    assert_normalizes("ako x > 0 { }", "if");
}
