// P8: MakeArray2 fast path tests.

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

#[test]
fn two_element_array_emits_makearray2() {
    let insns = compile_instructions("let a = [1, 2];");
    assert!(
        has_instruction(&insns, |i| matches!(i, Instruction::MakeArray2 { .. })),
        "2-element array must emit MakeArray2"
    );
}

#[test]
fn three_element_array_does_not_emit_makearray2() {
    let insns = compile_instructions("let a = [1, 2, 3];");
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::MakeArray2 { .. })),
        "3-element array must NOT emit MakeArray2"
    );
}

#[test]
fn spread_array_does_not_emit_makearray2() {
    // [1, ...b] with spread goes through compile_complex.rs → MakeArray + SpreadIntoArray
    // NOT MakeArray2. Use f([5]) to avoid 2-element arg triggering MakeArray2 at call site.
    let insns = compile_instructions("fn f(b) { let a = [1, ...b]; } f([5]);");
    assert!(
        !has_instruction(&insns, |i| matches!(i, Instruction::MakeArray2 { .. })),
        "spread array must NOT emit MakeArray2"
    );
}

#[test]
fn makearray2_runtime_correct() {
    // Verify runtime correctness via compilation and execution
    let src = "let a = [10, 20]; print(a[0] + a[1]);";
    let ast = parse(src).expect("parse");
    let mut c = Compiler::new();
    let bc = c.compile(&ast).expect("compile");
    let mut vm = hudhudscript_vm::VM::with_locale(hudhudscript_vm::VM::detect_locale(src));
    let result = vm.execute(&bc);
    assert!(
        result.is_ok(),
        "MakeArray2 runtime must work: {:?}",
        result.err()
    );
}
