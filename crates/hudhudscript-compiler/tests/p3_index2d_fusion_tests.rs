// P3: 2D index fusion tests — verify Index2D/IndexAssign2D emission
// for IndexArray/IndexAssignArray patterns introduced in P1.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_bytecode::Instruction;

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

#[test]
fn index2d_from_indexarray_reads() {
    let insns = compile_instructions("fn f() { let m = [[1,2],[3,4]]; let i = 0; let j = 1; return m[i][j]; } let x = f();");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::Index2D { .. })),
        "IndexArray+IndexArray must fuse to Index2D in function body, got none"
    );
}

#[test]
fn index2d_from_generic_index_reads() {
    let insns = compile_instructions("fn f(m, i, j) { return m[i][j]; } let x = f([[1,2],[3,4]], 0, 1);");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::Index2D { .. })),
        "Index+Index must fuse to Index2D, got none"
    );
}

#[test]
fn index_assign2d_from_mixed_indexarray_assign() {
    let insns = compile_instructions(
        "fn f() { let dp = [[0,0],[0,0]]; let i = 0; let j = 1; dp[i][j] = 42; } f();"
    );
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::IndexAssign2D { .. })),
        "IndexArray+IndexAssign must fuse to IndexAssign2D in function body, got none"
    );
}

#[test]
fn no_fusion_for_single_level() {
    let insns = compile_instructions("fn f() { let a = [1,2,3]; return a[1]; } let x = f();");
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::Index2D { .. })),
        "single-level index must NOT produce Index2D"
    );
}
