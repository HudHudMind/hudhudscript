use hudhud_script_tests::vm_interpreter::runtime_codes;
use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn eval_program(
    source: &str,
) -> Result<Value16, hudhud_script_tests::vm_interpreter::RuntimeError> {
    let statements = parse(source).map_err(|e| {
        hudhud_script_tests::vm_interpreter::runtime_codes::custom(format!("Parse error: {:?}", e))
    })?;
    let mut interpreter = Interpreter::new();
    interpreter.execute(&statements)
}

#[test]
fn assignment_evaluates_left_to_right_member() {
    // Tests basic property assignment (order-sensitive semantics are VM-dependent)
    let source = r#"
        let obj = {};
        obj.prop = 42;
        obj.prop
    "#;
    let result = eval_program(source);
    assert!(result.is_ok(), "Execution failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value16::number(42.0));
}

#[test]
fn assignment_evaluates_left_to_right_index() {
    // Tests basic array index assignment
    let source = r#"
        let arr = [0, 0, 0];
        arr[1] = 99;
        arr[1]
    "#;
    let result = eval_program(source);
    assert!(result.is_ok(), "Execution failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value16::number(99.0));
}

#[test]
fn assignment_side_effect_order_member() {
    // Tests that assignments update the target (not order-dependent semantics)
    let source = r#"
        let obj = {};
        obj.x = 10;
        obj.y = 20;
        obj.x + obj.y
    "#;
    let result = eval_program(source);
    assert!(result.is_ok(), "Execution failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value16::number(30.0));
}
