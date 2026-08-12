//! FIRE tests for P3/P4 superinstruction fusion opcodes.
//! Verifies that compiler actually emits the fused opcodes in bytecode.
use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

/// Helper: compile source, return bytecode.
fn compile(src: &str) -> hudhudscript_bytecode::Bytecode {
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    compiler.compile(&ast).unwrap()
}

/// Helper: check if an instruction variant exists in top-level + function bodies.
fn bytecode_has<F: Fn(&Instruction) -> bool>(
    bc: &hudhudscript_bytecode::Bytecode,
    pred: F,
) -> bool {
    for instr in &bc.instructions {
        if pred(instr) {
            return true;
        }
    }
    let funcs = bc.functions.borrow();
    for chunk in funcs.iter() {
        for instr in &chunk.instructions {
            if pred(instr) {
                return true;
            }
        }
    }
    false
}

#[test]
fn index2d_fires_in_bytecode() {
    let src = "let m = [[1,2,3],[4,5,6]]; let i = 1; let j = 2; let x = m[i][j];";
    let bc = compile(src);
    assert!(
        bytecode_has(&bc, |instr| matches!(instr, Instruction::Index2D { .. })),
        "Index2D should fire for matrix[i][j]"
    );
}

#[test]
fn index_assign_2d_fires_in_bytecode() {
    let src = "let m = [[1,2],[3,4]]; let i = 0; let j = 1; m[i][j] = 9;";
    let bc = compile(src);
    assert!(
        bytecode_has(&bc, |instr| matches!(
            instr,
            Instruction::IndexAssign2D { .. }
        )),
        "IndexAssign2D should fire for m[i][j] = val"
    );
}

#[test]
fn int_mul_add_assign_fires_in_bytecode() {
    let src = "let acc = 0; let a = 3; let b = 4; acc = acc + a * b;";
    let bc = compile(src);
    assert!(
        bytecode_has(&bc, |instr| matches!(
            instr,
            Instruction::IntMulAddAssign { .. }
        )),
        "IntMulAddAssign should fire for acc = acc + a * b"
    );
}

#[test]
fn strcat3_fires_in_bytecode() {
    let src = "let p = \"a\"; let q = \"b\"; let r = \"c\"; let s = p + q + r;";
    let bc = compile(src);
    assert!(
        bytecode_has(&bc, |instr| matches!(instr, Instruction::StrCat3 { .. })),
        "StrCat3 should fire for a + b + c"
    );
}

// ── CORRECTNESS tests ──────────────────────────────────────

fn run_and_get(src: &str, var: &str) -> i64 {
    let bc = compile(src);
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    vm.get_variable(var)
        .and_then(|v| v.as_int())
        .unwrap_or(-999)
}

fn run_and_get_str(src: &str, var: &str) -> String {
    let bc = compile(src);
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    vm.get_variable(var)
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

#[test]
fn index2d_correct_result() {
    let src = "let m = [[1,2,3],[4,5,6]]; let i = 1; let j = 2; let x = m[i][j];";
    assert_eq!(run_and_get(src, "x"), 6);
}

#[test]
fn index_assign_2d_correct_result() {
    let src = "let m = [[1,2],[3,4]]; let i = 0; let j = 1; m[i][j] = 9; let r = m[i][j];";
    assert_eq!(run_and_get(src, "r"), 9);
}

#[test]
fn int_mul_add_assign_correct_result() {
    let src = "let acc = 0; let a = 3; let b = 4; acc = acc + a * b;";
    assert_eq!(run_and_get(src, "acc"), 12);
}

#[test]
fn strcat3_correct_result() {
    let src = "let p = \"a\"; let q = \"b\"; let r = \"c\"; let s = p + q + r;";
    assert_eq!(run_and_get_str(src, "s"), "abc");
}

#[test]
fn strcat4_correct_result() {
    let src = "let p = \"a\"; let q = \"b\"; let r = \"c\"; let t = \"d\"; let s = p + q + r + t;";
    assert_eq!(run_and_get_str(src, "s"), "abcd");
}

// ── ArrayPush tests ─────────────────────────────────────────

#[test]
fn array_push_int_const_fires_in_bytecode() {
    let src = "let arr = []; arr.push(5);";
    let bc = compile(src);
    assert!(
        bytecode_has(&bc, |instr| matches!(
            instr,
            Instruction::ArrayPushIntConst { .. }
        )),
        "ArrayPushIntConst should fire for arr.push(5)"
    );
}

#[test]
fn array_push_int_const_correct_result() {
    let src = "let arr = []; arr.push(5); let r = arr[0];";
    assert_eq!(run_and_get(src, "r"), 5);
}

#[test]
fn array_push_const_fires_in_bytecode() {
    let src = "let arr = []; arr.push(\"x\");";
    let bc = compile(src);
    assert!(
        bytecode_has(&bc, |instr| matches!(
            instr,
            Instruction::ArrayPushConst { .. }
        )),
        "ArrayPushConst should fire for arr.push(\"x\")"
    );
}

#[test]
fn array_push_const_correct_result() {
    let src = "let arr = []; arr.push(\"x\"); let r = arr[0];";
    let bc = compile(src);
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    let v = vm
        .get_variable("r")
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    assert_eq!(v, "x");
}
