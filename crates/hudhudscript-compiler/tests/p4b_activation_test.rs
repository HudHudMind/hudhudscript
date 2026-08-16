// P4b activation debug: verify call_site_param_types is populated and used.

use hudhudscript_bytecode::Instruction;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

#[test]
fn recorded_type_present_in_map() {
    let src = "fn f(a) { return a[0]; } print(f([1,2]));";
    let ast = parse(src).expect("parse");
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).expect("compile");

    let names = bc.function_names.borrow();
    let funcs = bc.functions.borrow();
    let f_idx = names.get("f").expect("f should exist");
    let chunk = &funcs[*f_idx];

    eprintln!("f body: {:?}", chunk.instructions);
    eprintln!("f params: {:?}", chunk.params);
    eprintln!("f locals: {:?}", chunk.local_names);

    // f(a) called with [1,2] should produce IndexArray, not generic Index
    let has_index_array = chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::IndexArray { .. }));

    if !has_index_array {
        eprintln!("FAIL: f body still has generic Index instead of IndexArray");
        eprintln!("Instructions: {:?}", chunk.instructions);
    }

    assert!(
        has_index_array,
        "f(a) with array arg should emit IndexArray. Body: {:?}",
        chunk.instructions
    );
}
