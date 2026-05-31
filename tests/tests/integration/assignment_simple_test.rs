//! Simple test for assignment

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

#[test]
fn test_simple_array_assignment() {
    let code = r#"
        let arr = [1, 2, 3];
        arr[0] = 10;
    "#;

    let ast = parse(code).expect("Failed to parse");
    println!("AST: {:#?}", ast);

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_program(&ast);

    println!("Result: {:?}", result);

    let arr_val = interpreter.get_variable("arr");
    println!("arr from environment: {:?}", arr_val);

    assert!(result.is_ok());

    // Check if arr is actually an array
    if let Ok(val) = arr_val {
        if let Some(arr) = val.as_array() {
            println!("Array contents: {:?}", arr);
            assert_eq!(arr[0], hudhudscript_bytecode::Value16::number(10.0));
        } else {
            panic!("Expected array, got {:?}", val);
        }
    }
}
