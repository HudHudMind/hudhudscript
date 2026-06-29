//! Multilingual print() builtin — her dilin alias'i calismali, Kural 8 ihlalleri reddedilmeli.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run(source: &str) -> Result<(), String> {
    let ast = parse(source).map_err(|e| format!("parse: {}", e))?;
    let mut compiler = Compiler::new();
    let bytecode = compiler
        .compile(&ast)
        .map_err(|e| format!("compile: {}", e))?;
    let mut vm = VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bytecode)
        .map_err(|e| format!("execute: {}", e))?;
    Ok(())
}

// === Calismasi gerekenler (her dilin native print) ===
#[test]
fn print_english() {
    run(r#"print("ok")"#).unwrap();
}
#[test]
fn print_turkish_yaz() {
    run(r#"yaz("ok")"#).unwrap();
}
#[test]
fn print_turkish_yazdir() {
    run(r#"yazdır("ok")"#).unwrap();
}
#[test]
fn print_arabic() {
    run(r#"اطبع("ok")"#).unwrap();
}
#[test]
fn print_japanese_hyoji() {
    run(r#"表示("ok")"#).unwrap();
}
#[test]
fn print_japanese_kaku() {
    run(r#"書く("ok")"#).unwrap();
}
#[test]
fn print_korean() {
    run(r#"출력("ok")"#).unwrap();
}
#[test]
fn print_german() {
    run(r#"drucken("ok")"#).unwrap();
}
#[test]
fn print_french() {
    run(r#"imprimer("ok")"#).unwrap();
}
#[test]
fn print_spanish_portuguese() {
    run(r#"imprimir("ok")"#).unwrap();
}
#[test]
fn print_italian() {
    run(r#"stampare("ok")"#).unwrap();
}
#[test]
fn print_polish() {
    run(r#"drukuj("ok")"#).unwrap();
}
#[test]
fn print_indonesian() {
    run(r#"cetak("ok")"#).unwrap();
}
#[test]
fn print_russian() {
    run(r#"печать("ok")"#).unwrap();
}
#[test]
fn print_greek() {
    run(r#"εκτύπωση("ok")"#).unwrap();
}
#[test]
fn print_persian() {
    run(r#"چاپ("ok")"#).unwrap();
}
#[test]
fn print_hindi_print() {
    run(r#"प्रिंट("ok")"#).unwrap();
}
#[test]
fn print_hindi_chhap() {
    run(r#"छाप("ok")"#).unwrap();
}
#[test]
fn print_bengali() {
    run(r#"প্রিন্ট("ok")"#).unwrap();
}
#[test]
fn print_thai() {
    run(r#"พิมพ์("ok")"#).unwrap();
}
#[test]
fn print_chinese() {
    run(r#"打印("ok")"#).unwrap();
}
#[test]
fn print_kurdish() {
    run(r#"çap_bike("ok")"#).unwrap();
}
#[test]
fn print_serbo_croatian() {
    run(r#"ispiši("ok")"#).unwrap();
}
#[test]
fn print_vietnamese() {
    run(r#"in_ra("ok")"#).unwrap();
}

// === Kural 8: ROMANIZE YASAK — reddedilmeli ===
#[test]
fn print_romanized_japanese_rejected() {
    assert!(
        run(r#"kaku("ok")"#).is_err(),
        "kaku must be rejected (Kural 8)"
    );
    assert!(
        run(r#"hyoji("ok")"#).is_err(),
        "hyoji must be rejected (Kural 8)"
    );
}

// === Vietnamca in keyword cakisma testi ===
#[test]
fn vietnamese_in_keyword_conflict() {
    assert!(run(r#"in_ra("ok")"#).is_ok(), "in_ra should work");
    assert!(
        run(r#"in("ok")"#).is_err(),
        "'in' must fail (for-in reserved)"
    );
}

// === Kurtce iki kelime syntax kontrolu ===
#[test]
fn print_kurdish_two_word_rejected() {
    assert!(run(r#"çap bike("ok")"#).is_err(), "two-word must fail");
}
