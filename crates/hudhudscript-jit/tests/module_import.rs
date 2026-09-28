//! M4 (modül/import, v0.9.37) regresyon testleri.
//!
//! Named importlar AST-düzeyinde birleştirilir: hedef dosyanın istenen
//! fonksiyonları ana moda girer; çözümlenemeyen durum dürüst hatadır.

use hudhudscript_jit::JitRuntime;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn named_import_links_and_runs() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    rt.set_base_dir(fixture_dir());
    let out = rt
        .run("import { topla } from \"./lib_math.hud\"\nfn main() { return topla(20, 22) }")
        .expect("import birleştirilmeli");
    assert_eq!(out.exit_status, 0);
    assert_eq!(out.return_value, 42);
}

#[test]
fn missing_export_rejected_cleanly() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    rt.set_base_dir(fixture_dir());
    assert!(rt
        .run("import { fogsgon } from \"./lib_math.hud\"\nfn main() { return 1 }")
        .is_err());
}

#[test]
fn import_without_base_dir_honest_error() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    assert!(rt
        .run("import { topla } from \"./lib_math.hud\"\nfn main() { return 1 }")
        .is_err());
}

#[test]
fn assert_builtins_lower_and_pass() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    let out = rt
        .run("fn test_topla() { assert_eq(2 + 3, 5) }\nfn main() { test_topla(); return 0 }")
        .expect("assert lowering");
    assert_eq!(out.exit_status, 0);
}
