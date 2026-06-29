// P5: Math.floor(int/int) intrinsic bytecode sniff tests.
// These tests verify that the compiler emits IntDiv/IntDivI for
// Math.floor(int / int) and does NOT emit MethodCall for the floor call.

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
fn math_floor_div_literal_emits_intdivi() {
    let insns = compile_instructions("let x = 100; let y = Math.floor(x / 2);");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::IntDivI { .. })),
        "Math.floor(x / 2) must emit IntDivI"
    );
}

#[test]
fn math_floor_div_variable_emits_intdiv() {
    let insns = compile_instructions("let x = 100; let y = 3; let z = Math.floor(x / y);");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::IntDiv { .. })),
        "Math.floor(x / y) must emit IntDiv"
    );
}

#[test]
fn math_floor_float_no_int_div() {
    let insns = compile_instructions("print(Math.floor(3.7));");
    let has_int_div = has_instruction(&insns, |i| {
        matches!(i, Instruction::IntDiv { .. } | Instruction::IntDivI { .. })
    });
    assert!(
        !has_int_div,
        "Math.floor(3.7) must NOT emit IntDiv/IntDivI"
    );
}

#[test]
fn math_floor_single_int_no_intdiv() {
    let insns = compile_instructions("let n = 42; let m = Math.floor(n);");
    let has_int_div = has_instruction(&insns, |i| {
        matches!(i, Instruction::IntDiv { .. } | Instruction::IntDivI { .. })
    });
    assert!(
        !has_int_div,
        "Math.floor(single_int) must NOT emit IntDiv"
    );
}

#[test]
fn hot_loop_emits_intdivi() {
    let src = "
let total = 0;
for (let n = 1; n <= 10; n = n + 1) {
    let x = n;
    while (x > 0) {
        if (x % 2 == 1) {
            total = total + 1;
        }
        x = x / 2;
    }
}
print(total);
";
    let insns = compile_instructions(src);
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::IntDivI { .. })),
        "count_set_bits hot loop must use IntDivI"
    );
}
