//! Tests for Issue #107: Dual-Phase Compilation — ADT/Match IR lowering
//! Verifies that AST → Bytecode lowering works for enum declarations and match statements.

use hudhudscript_compiler::{Compiler, VM};
use hudhudscript_parser::parse;

fn compile_and_run(src: &str) {
    let stmts = parse(src).expect("parse error");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts).expect("compile error");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("vm error");
}

fn compile_only(src: &str) -> hudhudscript_compiler::Bytecode {
    let stmts = parse(src).expect("parse error");
    let mut compiler = Compiler::new();
    compiler.compile(&stmts).expect("compile error")
}

// ── Enum declaration lowers to EnumDecl instruction ──────────────────────────

#[test]
fn test_enum_decl_compiles() {
    compile_and_run("enum Direction { North, South, East, West }");
}

#[test]
fn test_enum_with_fields_compiles() {
    compile_and_run("enum Shape { Circle(radius), Rectangle(width, height), Point }");
}

#[test]
fn test_enum_decl_emits_instruction() {
    use hudhudscript_compiler::Instruction;
    let bc = compile_only("enum Color { Red, Green, Blue }");
    let has_enum_decl = bc
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::EnumDecl(_)));
    assert!(
        has_enum_decl,
        "Expected EnumDecl instruction, got: {:?}",
        bc.instructions
    );
}

// ── Match with wildcard ───────────────────────────────────────────────────────

#[test]
fn test_match_wildcard_compiles() {
    compile_and_run(
        r#"
        let x = 1;
        match x {
            _ => { let result = 0; }
}
    "#,
    );
}

// ── Match with literal pattern ────────────────────────────────────────────────

#[test]
fn test_match_literal_compiles() {
    compile_and_run(
        r#"
        let code = 42;
        match code {
            42 => { let found = true; }
            _ => { let found = false; }
}
    "#,
    );
}

#[test]
fn test_match_literal_emits_eq_instruction() {
    use hudhudscript_compiler::Instruction;
    let bc = compile_only(
        r#"
        let x = 1;
        match x {
            1 => { let a = 1; }
            _ => { let a = 0; }
}
    "#,
    );
    eprintln!("match instructions: {:?}", bc.instructions);
    let has_eq = bc.instructions.iter().any(|i| {
        matches!(
            i,
            Instruction::IntCmp { op: 4, .. }
            | Instruction::IntCmpRRJumpIfFalse { op: 4, .. }
        )
    });
    assert!(
        has_eq,
        "Expected IntCmp/IntCmpRRJumpIfFalse eq instruction for literal pattern"
    );
}

// ── Match with identifier binding ─────────────────────────────────────────────

#[test]
fn test_match_identifier_compiles() {
    compile_and_run(
        r#"
        let val = 5;
        match val {
            x => { let bound = x; }
}
    "#,
    );
}

#[test]
fn test_match_identifier_emits_bind_var() {
    use hudhudscript_compiler::Instruction;
    let bc = compile_only(
        r#"
        let v = 1;
        match v {
            x => { let y = x; }
}
    "#,
    );
    let has_bind = bc
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::BindVar(_)));
    assert!(
        has_bind,
        "Expected BindVar instruction for identifier pattern"
    );
}

// ── Match with enum variant pattern ──────────────────────────────────────────

#[test]
fn test_match_enum_variant_compiles() {
    compile_and_run(
        r#"
        enum Status { Active, Inactive }
        let s = "Status::Active";
        match s {
            Status::Active => { let ok = true; }
            _ => { let ok = false; }
}
    "#,
    );
}

#[test]
fn test_match_enum_variant_emits_match_variant() {
    use hudhudscript_compiler::Instruction;
    let bc = compile_only(
        r#"
        match s {
            Shape::Circle => { let a = 1; }
            _ => { let a = 0; }
}
    "#,
    );
    let has_mv = bc
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MatchVariant(_)));
    assert!(has_mv, "Expected MatchVariant instruction");
}

// ── Multilingual source → same IR ────────────────────────────────────────────

#[test]
fn test_turkish_if_same_ir_as_english() {
    use hudhudscript_compiler::Instruction;

    let en_src = "if (true) { let x = 1; }";
    let tr_src = "eğer (true) { let x = 1; }";

    let en_stmts = parse(en_src).expect("en parse");
    let tr_stmts = parse(tr_src).expect("tr parse");

    let mut c1 = Compiler::new();
    let mut c2 = Compiler::new();
    let bc_en = c1.compile(&en_stmts).expect("en compile");
    let bc_tr = c2.compile(&tr_stmts).expect("tr compile");

    // Both should produce the same instruction sequence
    assert_eq!(
        bc_en.instructions.len(),
        bc_tr.instructions.len(),
        "EN and TR should produce same number of instructions"
    );

    // Both should have JumpIfFalseR (register-based control flow)
    let en_has_jif = bc_en
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::JumpIfFalse { .. }));
    let tr_has_jif = bc_tr
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::JumpIfFalse { .. }));
    assert!(en_has_jif);
    assert!(tr_has_jif);
}

#[test]
fn test_arabic_while_same_ir_as_english() {
    use hudhudscript_compiler::Instruction;

    let en_src = "let i = 0; while (i < 3) { i = i + 1; }";
    let ar_src = "let i = 0; بينما (i < 3) { i = i + 1; }";

    let en_stmts = parse(en_src).expect("en parse");
    let ar_stmts = parse(ar_src).expect("ar parse");

    let mut c1 = Compiler::new();
    let mut c2 = Compiler::new();
    let bc_en = c1.compile(&en_stmts).expect("en compile");
    let bc_ar = c2.compile(&ar_stmts).expect("ar compile");

    // Both should produce Jump and a conditional jump (JumpIfFalse or fused IntLe/LtRRJumpIfFalse)
    let en_has_jump = bc_en
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Jump(_)));
    let ar_has_jump = bc_ar
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Jump(_)));
    let en_has_jif = bc_en.instructions.iter().any(|i| {
        matches!(
            i,
            Instruction::JumpIfFalse { .. }
                | Instruction::IntLeRRJumpIfFalse { .. }
                | Instruction::IntLtRRJumpIfFalse { .. }
                | Instruction::IntCmpIJumpIfFalse { .. }
                | Instruction::IntCmpRRJumpIfFalse { .. }
        )
    });
    let ar_has_jif = bc_ar.instructions.iter().any(|i| {
        matches!(
            i,
            Instruction::JumpIfFalse { .. }
                | Instruction::IntLeRRJumpIfFalse { .. }
                | Instruction::IntLtRRJumpIfFalse { .. }
                | Instruction::IntCmpIJumpIfFalse { .. }
                | Instruction::IntCmpRRJumpIfFalse { .. }
        )
    });

    assert!(en_has_jump, "EN: expected Jump");
    assert!(
        ar_has_jump,
        "AR: expected Jump — got: {:?}",
        bc_ar.instructions
    );
    assert!(en_has_jif, "EN: expected conditional jump");
    assert!(ar_has_jif, "AR: expected conditional jump");
}
