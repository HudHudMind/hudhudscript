//! G3 — fusion emit kalıcı testleri (sessiz çürüme koruması).
//!
//! Her YAŞAYAN fusion opcode'u için: minimal kaynak → derle → nihai komut
//! akışında (ana chunk + fonksiyon chunk'ları) fused opcode'un VARLIĞINI
//! doğrula. Telemetry feature'ına bağımlı DEĞİL — Instruction enum'u
//! doğrudan taranır; böylece bu testler default build'de de çalışır.
//!
//! Envanter kaynağı: 2026-08-11 iki tur empirik battery census'u
//! (docs/FUSION_TABLE.md). Bir desen ölürse buradaki test kırılır —
//! HANDOVER_G3'teki "sessiz çürüme" bir daha yaşanmaz.
use hudhudscript_bytecode::{Bytecode, Instruction};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn compile(src: &str) -> Bytecode {
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    compiler.compile(&ast).unwrap()
}

/// Ana chunk + tüm fonksiyon chunk'larında `pred`'i sağlayan komut sayısı.
fn count_instrs(bc: &Bytecode, pred: impl Fn(&Instruction) -> bool) -> usize {
    let mut n = bc.instructions.iter().filter(|i| pred(i)).count();
    for chunk in bc.functions.borrow().iter() {
        n += chunk.instructions.iter().filter(|i| pred(i)).count();
    }
    n
}

macro_rules! assert_emits {
    ($src:expr, $variant:pat, $name:expr) => {
        let bc = compile($src);
        let n = count_instrs(&bc, |i| matches!(i, $variant));
        assert!(
            n > 0,
            "{} EMİT EDİLMEDİ — fusion öldü (kaynak: {})",
            $name,
            $src
        );
    };
}

// ── immediate-katlama (fuse_slot_immediate) ──────────────────────────

#[test]
fn g3_emit_int_add_i_jump() {
    assert_emits!(
        "fn f(n) { var i = 0; var s = 0; while (i < n) { s = s + 1; i = i + 1 } return s }",
        Instruction::IntAddIJump { .. },
        "IntAddIJump"
    );
}

#[test]
fn g3_emit_int_sub_i_jump() {
    assert_emits!(
        "fn f(n) { var i = n; var s = 0; while (i > 0) { s = s + i; i = i - 1 } return s }",
        Instruction::IntSubIJump { .. },
        "IntSubIJump"
    );
}

#[test]
fn g3_emit_int_mul_i() {
    assert_emits!(
        "fn f(n) { var s = 0; var i = 0; while (i < n) { s = s + i * 2; i = i + 1 } return s }",
        Instruction::IntMulI { .. },
        "IntMulI"
    );
}

#[test]
fn g3_emit_int_cmp_i() {
    assert_emits!(
        "fn f(a) { var t = a > 5; return t }",
        Instruction::IntCmpI { .. },
        "IntCmpI"
    );
}

#[test]
fn g3_emit_int_mod_cmp_i() {
    assert_emits!(
        "fn f(a) { if (a % 3 == 0) { return 1 } return 0 }",
        Instruction::IntModCmpI { .. },
        "IntModCmpI"
    );
}

// ── cmp+branch / döngü kuyruğu ───────────────────────────────────────

#[test]
fn g3_emit_int_cmp_i_jump_if_false() {
    assert_emits!(
        "fn f(i) { if (i < 10) { return 1 } return 0 }",
        Instruction::IntCmpIJumpIfFalse { .. },
        "IntCmpIJumpIfFalse"
    );
}

#[test]
fn g3_emit_int_lt_rr_jump_if_false() {
    // NOT: `if (a < b)` şekli FÜZLENMEZ (IntCmp+JumpIfFalse kalır — G4
    // adayı); IntLtRRJumpIfFalse yalnız while-döngü başlığından çıkar.
    assert_emits!(
        "fn f(n) { var s = 0; var i = 0; while (i < n) { s = s + i; i = i + 1 } return s }",
        Instruction::IntLtRRJumpIfFalse { .. },
        "IntLtRRJumpIfFalse"
    );
}

#[test]
fn g3_emit_int_le_rr_jump_if_false() {
    assert_emits!(
        "fn f(n) { var s = 0; var i = 0; while (i <= n) { s = s + i; i = i + 1 } return s }",
        Instruction::IntLeRRJumpIfFalse { .. },
        "IntLeRRJumpIfFalse"
    );
}

#[test]
fn g3_emit_int_cmp_i_return() {
    assert_emits!(
        "fn f(a) { return a < 10 }",
        Instruction::IntCmpIReturn { .. },
        "IntCmpIReturn"
    );
}

// ── aritmetik compound ───────────────────────────────────────────────

#[test]
fn g3_emit_int_mul_add_assign_int_shape() {
    // acc = acc + a*b (int/generic yol: IntMul+IntAdd+Move → IntMulAddAssign)
    assert_emits!(
        "fn f(a, b, n) { var acc = 0; var i = 0; while (i < n) { acc = acc + a * b; i = i + 1 } return acc }",
        Instruction::IntMulAddAssign { .. },
        "IntMulAddAssign (int)"
    );
}

#[test]
fn g3_emit_int_mul_add_assign_num_shape() {
    // G3 diriltme: tip yayılımı float param'larda NumMul üretir; matcher
    // artık Num varyantını da kabul ediyor (fuse_super_extra int_mul_add_match).
    assert_emits!(
        "fn f(x, y, n) { var acc = 0.0; var i = 0; while (i < n) { acc = acc + x * y; i = i + 1 } return acc }\nvar s = [4]\nvar r = f(1.5, 2.0, s[0])",
        Instruction::IntMulAddAssign { .. },
        "IntMulAddAssign (num-param)"
    );
}

#[test]
fn g3_emit_num_mul_add_assign() {
    // acc = acc*b + c şekli (NumMul+NumAdd in-place)
    assert_emits!(
        "fn f(b, c, n) { var acc = 1.0; var i = 0; while (i < n) { acc = acc * b + c; i = i + 1 } return acc }\nvar s = [3]\nvar r = f(2.0, 0.5, s[0])",
        Instruction::NumMulAddAssign { .. },
        "NumMulAddAssign"
    );
}

#[test]
fn g3_emit_int_mul_mod() {
    assert_emits!(
        "fn f(a, b, m) { return (a * b) % m }",
        Instruction::IntMulMod { .. },
        "IntMulMod"
    );
}

#[test]
fn g3_emit_int_mul_mod_i() {
    assert_emits!(
        "fn f(a, b) { return (a * b) % 10 }",
        Instruction::IntMulModI { .. },
        "IntMulModI"
    );
}

#[test]
fn g3_emit_property_sub_assign() {
    assert_emits!(
        "fn f(y) { var o = {x: 10}; o.x = o.x - y; return o.x }",
        Instruction::PropertySubAssign { .. },
        "PropertySubAssign"
    );
}

// ── arith+return ─────────────────────────────────────────────────────

#[test]
fn g3_emit_arith_return_family() {
    assert_emits!(
        "fn f(a, b) { return a + b }",
        Instruction::IntAddReturn { .. },
        "IntAddReturn"
    );
    assert_emits!(
        "fn f(a, b) { return a - b }",
        Instruction::IntSubReturn { .. },
        "IntSubReturn"
    );
    assert_emits!(
        "fn f(a, b) { return a * b }",
        Instruction::IntMulReturn { .. },
        "IntMulReturn"
    );
    assert_emits!(
        "fn f(a, b) { return a / b }",
        Instruction::IntDivReturn { .. },
        "IntDivReturn"
    );
}

// ── const-katlama ────────────────────────────────────────────────────

#[test]
fn g3_emit_return_const() {
    // NOT: `return 42` (int) FÜZLENMEZ — LoadIntConst `int_constants`
    // havuzundan, ReturnConst `constants` havuzundan okur (G4 adayı:
    // ReturnIntConst). String sabiti LoadConst kullanır → füzlenir.
    assert_emits!(
        "fn f() { return \"sabit\" }\nprint(f())",
        Instruction::ReturnConst { .. },
        "ReturnConst"
    );
}

#[test]
fn g3_emit_store_global_const() {
    // NOT: kullanılmayan/yalnız-main'de-okunan top-level değişken main-local
    // register'a iner (PERF-B1) — StoreGlobalConst yalnız fonksiyondan
    // erişilen (paylaşılan) global'de çıkar.
    assert_emits!(
        "var g = 42\nfn f() { return g }\nprint(f())",
        Instruction::StoreGlobalConst { .. },
        "StoreGlobalConst"
    );
}

#[test]
fn g3_emit_array_push_int_const() {
    assert_emits!(
        "var a = []\na.push(42)",
        Instruction::ArrayPushIntConst { .. },
        "ArrayPushIntConst"
    );
}

#[test]
fn g3_emit_array_push_const() {
    assert_emits!(
        "var a = []\na.push(\"s\")",
        Instruction::ArrayPushConst { .. },
        "ArrayPushConst"
    );
}

// ── string ───────────────────────────────────────────────────────────

#[test]
fn g3_emit_strcat3() {
    // NOT: tip yayılımı çağrı-yerinden string kanıtını `print(f(...))`
    // bağlamında üretiyor; `var r = f(...)` bağlamında ÜRETMİYOR (generic
    // IntAdd kalır). Bu testin kaynağı o yüzden print'li.
    assert_emits!(
        "fn f(a, b, c) { return a + b + c }\nprint(f(\"x\", \"y\", \"z\"))",
        Instruction::StrCat3 { .. },
        "StrCat3"
    );
}

// ── G3 doğruluk regresyonu: PropertySubAssign null-clobber fix'i ─────

#[test]
fn g3_property_sub_assign_local_object_correct() {
    // Fix öncesi: füzyon SetProperty.dst'yi obj'a geri taşıyan Move'u
    // bırakıyordu; hiç yazılmayan register obj'u null'la eziyordu →
    // "Property 'x' not found on null". v0.8.183'ten beri kırıktı.
    let bc = compile("fn p(y) { var o = {x: 10}; o.x = o.x - y; return o.x }\nvar sonuc = p(3)");
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute PropertySubAssign repro");
    let got = vm
        .get_variable("sonuc")
        .and_then(|v| v.as_int())
        .unwrap_or(-999);
    assert_eq!(got, 7, "o.x = 10 - 3 = 7 olmalı");
}

#[test]
fn g3_property_sub_assign_result_used_later() {
    // Objenin füzyon SONRASI da canlı kaldığını doğrula (iki ardışık düşüm).
    let bc = compile(
        "fn p(y) { var o = {x: 100}; o.x = o.x - y; o.x = o.x - y; return o.x }\nvar sonuc = p(30)",
    );
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute double PropertySubAssign");
    let got = vm
        .get_variable("sonuc")
        .and_then(|v| v.as_int())
        .unwrap_or(-999);
    assert_eq!(got, 40, "100 - 30 - 30 = 40 olmalı");
}
