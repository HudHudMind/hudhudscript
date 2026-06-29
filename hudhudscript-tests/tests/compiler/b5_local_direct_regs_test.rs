//! B5-TEST: Local identifier direct-reg bytecode verification.
use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::Compiler;

#[test]
fn b5_horner_direct_reg_bytecode() {
    let src = "fn horner_test(coeffs, x) { let result = coeffs[2]; let i = 1; while (i >= 0) { result = result * x + coeffs[i]; i = i - 1; } return result; }";
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let functions = bc.functions.borrow();
    let bc_ref = &bc;
    let instrs = &bc_ref.get_function("horner_test").unwrap().instructions;

    let (coeffs_reg, x_reg, i_reg) = (0u8, 1u8, 3u8);
    let mut has_fma = false;

    for instr in instrs {
        if let Instruction::NumMulAddIndexed { acc: _a, mul, arr, idx: _i } = instr {
            has_fma = true;
            assert_eq!(*mul, x_reg, "NumMulAddIndexed mul must be x register");
            assert_eq!(*arr, coeffs_reg, "NumMulAddIndexed arr must be coeffs register");
        }
    }
    assert!(has_fma, "NumMulAddIndexed not found");
}
