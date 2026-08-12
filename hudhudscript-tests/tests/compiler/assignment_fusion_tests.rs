//! Assignment-fusion helper public API tests.
//!
//! Moved from `hudhudscript-compiler/src/compiler/stmt_shared/assignment_fusions.rs`
//! inline test module as part of I2-A5 (private inline test consolidation).
//! The private helper unit tests remain in the main repo; this file only covers
//! the public `Compiler` Horner / NumMulAddAssign integration test.

use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::Compiler;

#[test]
fn test_horner_fma_emitted_with_distinct_operands() {
    // coeffs = [1,2,3], x = 10, expected: 321
    let src = "fn horner_test(coeffs, x) { let result = coeffs[2]; let i = 1; while (i >= 0) { result = result * x + coeffs[i]; i = i - 1; } return result; } horner_test([1,2,3], 10);";
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();

    let horner = bc
        .get_function("horner_test")
        .expect("horner_test function not found");

    let mut has_fma = false;
    for instr in &horner.instructions {
        if let Instruction::NumMulAddIndexed {
            acc: _a,
            mul,
            arr,
            idx: _i,
        } = instr
        {
            has_fma = true;
            assert_ne!(
                mul, arr,
                "NumMulAddIndexed mul and arr must be different registers (both {mul})"
            );
        }
    }
    assert!(has_fma, "NumMulAddIndexed not emitted for horner pattern");
}
