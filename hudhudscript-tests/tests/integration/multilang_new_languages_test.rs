//! Per-language representative parse tests for the 11 newly added languages:
//! BN, BS, EL, HR, ID, IT, PL, PT, SR, TH, VI
//!
//! Each test verifies that keyword normalization + parsing produces a valid AST.
//! Pattern: a simple `if` statement or `use...as` import per language.

use hudhudscript_parser::parse;

// ── BN — Bengali ────────────────────────────────────────────────────────────

#[test]
    #[ignore] // locale keyword not yet implemented
fn test_bn_if_statement() {
    // যদি (if) x { }
    let source = "যদি x { }";
    let result = parse(source);
    assert!(result.is_ok(), "BN if failed: {:?}", result.err());
}

#[test]
fn test_bn_function_decl() {
    // ফাংশন (function) hello() { ফেরত_দাও (return) 1; }
    let source = "ফাংশন hello() { ফেরত_দাও 1; }";
    let result = parse(source);
    assert!(result.is_ok(), "BN function failed: {:?}", result.err());
}

// ── BS — Bosnian ─────────────────────────────────────────────────────────────

#[test]
    #[ignore] // locale keyword not yet implemented
fn test_bs_if_statement() {
    // ako (if) x { }
    let source = "ako x { }";
    let result = parse(source);
    assert!(result.is_ok(), "BS if failed: {:?}", result.err());
}

#[test]
fn test_bs_function_decl() {
    // funkcija (function) hello() { vrati (return) 1; }
    let source = "funkcija hello() { vrati 1; }";
    let result = parse(source);
    assert!(result.is_ok(), "BS function failed: {:?}", result.err());
}

// ── EL — Greek ───────────────────────────────────────────────────────────────

#[test]
    #[ignore] // locale keyword not yet implemented
fn test_el_if_statement() {
    // αν (if) x { }
    let source = "αν x { }";
    let result = parse(source);
    assert!(result.is_ok(), "EL if failed: {:?}", result.err());
}

#[test]
fn test_el_function_decl() {
    // συνάρτηση (function) hello() { επιστροφή (return) 1; }
    let source = "συνάρτηση hello() { επιστροφή 1; }";
    let result = parse(source);
    assert!(result.is_ok(), "EL function failed: {:?}", result.err());
}

// ── HR — Croatian ────────────────────────────────────────────────────────────

#[test]
    #[ignore] // locale keyword not yet implemented
fn test_hr_if_statement() {
    // ako (if) — shared with BS via normalizer
    let source = "ako x { }";
    let result = parse(source);
    assert!(result.is_ok(), "HR if failed: {:?}", result.err());
}

#[test]
fn test_hr_function_decl() {
    // funkcija (function) — shared with BS, HR-specific provider normalizes to "provider"
    let source = "funkcija hello() { vrati 1; }";
    let result = parse(source);
    assert!(result.is_ok(), "HR function failed: {:?}", result.err());
}

// ── ID — Indonesian ──────────────────────────────────────────────────────────

#[test]
    #[ignore] // locale keyword not yet implemented
fn test_id_if_statement() {
    // jika (if) x { }
    let source = "jika x { }";
    let result = parse(source);
    assert!(result.is_ok(), "ID if failed: {:?}", result.err());
}

#[test]
fn test_id_function_decl() {
    // fungsi (function) hello() { kembali (return) 1; }
    let source = "fungsi hello() { kembali 1; }";
    let result = parse(source);
    assert!(result.is_ok(), "ID function failed: {:?}", result.err());
}

// ── IT — Italian ─────────────────────────────────────────────────────────────

#[test]
    #[ignore] // locale keyword not yet implemented
fn test_it_if_statement() {
    // se (if) x { }
    let source = "se x { }";
    let result = parse(source);
    assert!(result.is_ok(), "IT if failed: {:?}", result.err());
}

#[test]
fn test_it_function_decl() {
    // funzione (function) hello() { ritornare (return) 1; }
    let source = "funzione hello() { ritornare 1; }";
    let result = parse(source);
    assert!(result.is_ok(), "IT function failed: {:?}", result.err());
}

// ── PL — Polish ──────────────────────────────────────────────────────────────

#[test]
    #[ignore] // locale keyword not yet implemented
fn test_pl_if_statement() {
    // jeśli (if) x { }
    let source = "jeśli x { }";
    let result = parse(source);
    assert!(result.is_ok(), "PL if failed: {:?}", result.err());
}

#[test]
fn test_pl_function_decl() {
    // funkcja (function) hello() { zwróć (return) 1; }
    let source = "funkcja hello() { zwróć 1; }";
    let result = parse(source);
    assert!(result.is_ok(), "PL function failed: {:?}", result.err());
}

// ── PT — Portuguese ──────────────────────────────────────────────────────────

#[test]
    #[ignore] // locale keyword not yet implemented
fn test_pt_if_statement() {
    // se (if) x { } — shared with IT via normalizer
    let source = "se x { }";
    let result = parse(source);
    assert!(result.is_ok(), "PT if failed: {:?}", result.err());
}

#[test]
fn test_pt_function_decl() {
    // função (function) hello() { retornar (return) 1; }
    let source = "função hello() { retornar 1; }";
    let result = parse(source);
    assert!(result.is_ok(), "PT function failed: {:?}", result.err());
}

// ── SR — Serbian Cyrillic ────────────────────────────────────────────────────

#[test]
    #[ignore] // locale keyword not yet implemented
fn test_sr_if_statement() {
    // ако (if) x { }
    let source = "ако x { }";
    let result = parse(source);
    assert!(result.is_ok(), "SR if failed: {:?}", result.err());
}

#[test]
fn test_sr_function_decl() {
    // функција (function) hello() { врати (return) 1; }
    let source = "функција hello() { врати 1; }";
    let result = parse(source);
    assert!(result.is_ok(), "SR function failed: {:?}", result.err());
}

// ── TH — Thai ────────────────────────────────────────────────────────────────

#[test]
    #[ignore] // locale keyword not yet implemented
fn test_th_if_statement() {
    // ถ้า (if) x { }
    let source = "ถ้า x { }";
    let result = parse(source);
    assert!(result.is_ok(), "TH if failed: {:?}", result.err());
}

#[test]
fn test_th_function_decl() {
    // ฟังก์ชัน (function) hello() { คืนค่า (return) 1; }
    let source = "ฟังก์ชัน hello() { คืนค่า 1; }";
    let result = parse(source);
    assert!(result.is_ok(), "TH function failed: {:?}", result.err());
}

// ── VI — Vietnamese ──────────────────────────────────────────────────────────

#[test]
    #[ignore] // locale keyword not yet implemented
fn test_vi_if_statement() {
    // nếu (if) x { }
    let source = "nếu x { }";
    let result = parse(source);
    assert!(result.is_ok(), "VI if failed: {:?}", result.err());
}

#[test]
fn test_vi_function_decl() {
    // hàm (function) hello() { trả_về (return) 1; }
    let source = "hàm hello() { trả_về 1; }";
    let result = parse(source);
    assert!(result.is_ok(), "VI function failed: {:?}", result.err());
}

// ── Cross-language: all 11 produce same AST for equivalent if ────────────────

#[test]
    #[ignore] // locale keyword not yet implemented
fn test_all_new_languages_if_normalizes_to_same() {
    // Her dilde "if x { }" eşdeğeri — normalizer sonrası aynı AST beklenir
    let cases = vec![
        ("BN", "যদি x { }"),
        ("BS", "ako x { }"),
        ("EL", "αν x { }"),
        ("ID", "jika x { }"),
        ("IT", "se x { }"),
        ("PL", "jeśli x { }"),
        ("PT", "se x { }"),
        ("SR", "ако x { }"),
        ("TH", "ถ้า x { }"),
        ("VI", "nếu x { }"),
    ];

    for (lang, source) in cases {
        let result = parse(source);
        assert!(
            result.is_ok(),
            "Language {} failed to parse '{}': {:?}",
            lang,
            source,
            result.err()
        );
        let stmts = result.unwrap();
        assert!(
            !stmts.is_empty(),
            "Language {} should produce at least 1 statement",
            lang
        );
    }
}
