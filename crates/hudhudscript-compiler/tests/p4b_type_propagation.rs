// P4b: Two-pass type propagation integration test.

use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

#[test]
fn function_called_with_array_uses_indexarray() {
    let src = "fn f(a) { return a[0]; } print(f([1,2]));";
    let ast = parse(src).expect("parse");
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).expect("compile");

    // Get function body
    let names = bc.function_names.borrow();
    let funcs = bc.functions.borrow();
    let f_idx = names.get("f").expect("f should be in function names");
    let chunk = &funcs[*f_idx];

    let has_index_array = chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::IndexArray { .. }));
    let has_generic_index = chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Index { .. }));

    eprintln!("f body instructions: {:?}", chunk.instructions);
    assert!(
        has_index_array,
        "f(a) called with array should emit IndexArray, got generic={}",
        has_generic_index
    );
}
