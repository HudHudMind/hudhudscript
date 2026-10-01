// B1 (filo raporu): harita-tabanlı zincirli erişim (`m["k"][i]`) genel
// `Index` çiftlerinin Index2D'ye eritilmesiyle çöküyordu (`[E0038] Index2D:
// obj not array`). Index2D yalnızca dizi-içinden-dizi okur; eritme artık
// YALNIZCA tabanı derleyici kanıtıyla dizi olan (`IndexArray` başlangıçlı)
// çiftlere uygulanır. Bu dosya kanıtsız tabanların ERİTİLMEDİĞİNİ doğrular;
// dizi-kanıtlı hız yolunun korunduğu p3_index2d_fusion_tests.rs'tedir.

use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

fn compile_instructions(src: &str) -> Vec<Instruction> {
    let ast = parse(src).expect("parse failed");
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).expect("compile failed");
    let mut all = bc.instructions.clone();
    for chunk in bc.functions.borrow().iter() {
        all.extend_from_slice(&chunk.instructions);
    }
    all
}

fn has_instruction<F>(insns: &[Instruction], pred: F) -> bool
where
    F: Fn(&Instruction) -> bool,
{
    insns.iter().any(pred)
}

/// `m["d"][i]` — sabit dizge anahtarı + değişken indeks: taban harita,
/// kanıt yok → Index2D ÜRETİLMEMELİ (eski davranış: erir + E0038).
#[test]
fn no_index2d_for_map_base_read_chain() {
    let insns = compile_instructions(
        "fn f(m, i) { return m[\"d\"][i]; } let x = f({\"d\": [1,2,3]}, 1);",
    );
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::Index2D { .. })),
        "map-base chain (m[\"d\"][i]) must NOT fuse to Index2D"
    );
}

/// `m[k][i]` — değişken dizge anahtarı: taban tipi bilinmiyor → eritme yok.
#[test]
fn no_index2d_for_unproven_base_chain() {
    let insns =
        compile_instructions("fn f(m, k, i) { return m[k][i]; } let x = f({\"d\": [1,2,3]}, \"d\", 1);");
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::Index2D { .. })),
        "unproven-base chain (m[k][i]) must NOT fuse to Index2D"
    );
}

/// `m["d"][i] = v` — yazma tarafı: IndexAssign2D ÜRETİLMEMELİ.
#[test]
fn no_index_assign2d_for_map_base_write_chain() {
    let insns = compile_instructions(
        "fn f(m, i) { m[\"d\"][i] = 9; return m[\"d\"][i]; } let x = f({\"d\": [1,2,3]}, 1);",
    );
    assert!(
        !has_instruction(
            &insns,
            |i| matches!(i, Instruction::IndexAssign2D { .. })
        ),
        "map-base write chain (m[\"d\"][i] = v) must NOT fuse to IndexAssign2D"
    );
}

/// Hız yolu koruması: dizi-kanıtlı taban (çağrı-yeri tipi Array) eritmeye
/// DEVAM ETMELİ — matris `a[i][k]` deseni.
#[test]
fn index2d_kept_for_array_proven_base() {
    let insns =
        compile_instructions("fn f(a, i, j) { return a[i][j]; } let x = f([[1,2],[3,4]], 1, 0);");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::Index2D { .. })),
        "array-proven base (a[i][j]) must keep fusing to Index2D"
    );
}
