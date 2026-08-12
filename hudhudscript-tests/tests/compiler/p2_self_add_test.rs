//! P2-B1-FIX2: Self-add-int regression tests.
//! Lock: IntAddI self-update (dst==src) for numeric locals, NO self-update for strings.

use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::Compiler;

fn compile(src: &str) -> hudhudscript_bytecode::Bytecode {
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::new();
    compiler.compile(&ast).unwrap()
}

fn run_get_variable(src: &str, name: &str) -> hudhudscript_bytecode::Value16 {
    let bc = compile(src);
    let mut vm = hudhudscript_vm::VM::new();
    vm.execute(&bc).expect("execute failed");
    vm.get_variable(name)
        .cloned()
        .unwrap_or(hudhudscript_bytecode::Value16::null())
}

fn get_func_instrs(bc: &hudhudscript_bytecode::Bytecode, name: &str) -> Vec<Instruction> {
    bc.get_function(name).unwrap().instructions.clone()
}

// ── 1. Numeric self-add semantic ─────────────────────────────

#[test]
fn p2_self_add_numeric_semantic() {
    let src = "let i = 0; i = i + 1; i = i + 1; let out = i;";
    let val = run_get_variable(src, "out");
    assert_eq!(
        val.as_int(),
        Some(2),
        "i=0; i=i+1; i=i+1 → out=2, got {:?}",
        val
    );
}

// ── 2. Commutative numeric self-add semantic ──────────────────

#[test]
fn p2_self_add_commutative_numeric_semantic() {
    let src = "let i = 0; i = 1 + i; i = 2 + i; let out = i;";
    let val = run_get_variable(src, "out");
    assert_eq!(
        val.as_int(),
        Some(3),
        "i=0; i=1+i; i=2+i → out=3, got {:?}",
        val
    );
}

// ── 3. Numeric bytecode: IntAddI dst==src, no trailing Move ────

#[test]
fn p2_self_add_numeric_bytecode_has_no_trailing_move() {
    let bc = compile("fn p2_inc() { let i = 0; i = i + 1; return i; } let out = p2_inc();");
    let instrs = get_func_instrs(&bc, "p2_inc");

    let mut found = false;
    for (ix, instr) in instrs.iter().enumerate() {
        if let Instruction::IntAddI { dst, src, imm } = instr {
            if *imm == 1 {
                assert_eq!(
                    dst, src,
                    "IntAddI must be self-update (dst==src), got dst={} src={}",
                    dst, src
                );
                // Check next instruction is NOT a Move to the same register
                if ix + 1 < instrs.len() {
                    if let Instruction::Move { dst: m_dst, src: _ } = &instrs[ix + 1] {
                        assert_ne!(
                            *m_dst, *dst,
                            "trailing Move to same register after IntAddI self-update at ip={}",
                            ix
                        );
                    }
                }
                found = true;
            }
        }
    }
    assert!(found, "IntAddI self-update not found in p2_inc function");
}

// ── 4. String concat: s = s + 1 semantic ──────────────────────

#[test]
fn p2_self_add_string_concat_left_semantic() {
    let src = "let s = \"a\"; s = s + 1; let out = s;";
    let val = run_get_variable(src, "out");
    assert_eq!(
        val.as_string(),
        Some("a1".to_string()),
        "s='a'; s=s+1 → 'a1', got {:?}",
        val
    );
}

// ── 5. String concat: s = 1 + s semantic ──────────────────────

#[test]
fn p2_self_add_string_concat_right_semantic() {
    let src = "let s = \"a\"; s = 1 + s; let out = s;";
    let val = run_get_variable(src, "out");
    assert_eq!(
        val.as_string(),
        Some("1a".to_string()),
        "s='a'; s=1+s → '1a', got {:?}",
        val
    );
}

// ── 6. String concat bytecode guard: NO IntAddI self-update ────

#[test]
fn p2_self_add_string_concat_does_not_emit_intaddi_self_update() {
    let src = "fn p2_str_left() { let s = \"a\"; s = s + 1; return s; } fn p2_str_right() { let s = \"a\"; s = 1 + s; return s; }";
    let bc = compile(src);

    for func_name in &["p2_str_left", "p2_str_right"] {
        let instrs = get_func_instrs(&bc, func_name);
        for instr in &instrs {
            if let Instruction::IntAddI { dst, src, .. } = instr {
                // dst == src means self-update IntAddI — must NOT happen for strings
                assert_ne!(dst, src,
                    "IntAddI self-update (dst==src) found in {func_name} — string concat must use generic path");
            }
        }
    }
}
