//! M6 (SOP & Council, v0.9.39) regresyon testleri.
//!
//! Role/Relation/Event/Council/Compose bildirimleri atlanır (metadata);
//! Effect → __event__X fn + bare çağrı çözümlemesi; kapsamsız `on ability`
//! tüm subject metot tablolarına kaydedilir; view-subject (of + ability)
//! DÜRÜST VM-fallback; input/confirm ABI'si çalışır (test stdin'i EOF →
//! boş dizge / 0).

use hudhudscript_jit::JitRuntime;

#[test]
fn effect_declaration_and_bare_call_run_natively() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    let out = rt
        .run("role R { can a }\nsubject S { state v: 10\n  on bump(self, d) { self.v = self.v + d } }\neffect on Ping(t) { t.v = t.v + 5 }\nlet s = spawn S\nPing(s)\ns.bump(2)\nprint(s.v)")
        .expect("SOP temel");
    assert_eq!(out.exit_status, 0);
}

#[test]
fn view_subject_falls_back_honestly() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    // of-view + ability → rewrite None → dürüst hata (VM fallback)
    assert!(rt
        .run("subject Base { state x: 1 }\nsubject View of Base { on m(self) { self.x = 2 } }")
        .is_err());
}

#[test]
fn input_confirm_abi_with_eof_stdin() {
    let mut rt = JitRuntime::with_backend("cranelift").expect("runtime");
    // cargo test stdin = kapalı/EOF → input boş dizge handle, confirm 0
    let out = rt
        .run("fn main() { let c = confirm(); let s = input(); if c == 0 { return 7 } return 1 }")
        .expect("input/confirm lowering");
    assert_eq!(out.exit_status, 0);
    assert_eq!(out.return_value, 7);
}
