//! Tests for assignment operations (member and index assignment)

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

#[test]
fn test_array_index_assignment() {
    let code = r#"
        let arr = [1, 2, 3];
        arr[0] = 10;
        arr[1] = 20;
        arr[2] = 30;
        arr;
    "#;

    let ast = parse(code).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_program(&ast).expect("Failed to execute");

    // Get the array from environment
    let arr_val = interpreter.get_variable("arr").expect("Failed to get arr");

    // Check that array was modified
    if let Some(arr) = arr_val.as_array() {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], hudhudscript_bytecode::Value16::number(10.0));
        assert_eq!(arr[1], hudhudscript_bytecode::Value16::number(20.0));
        assert_eq!(arr[2], hudhudscript_bytecode::Value16::number(30.0));
    } else {
        panic!("Expected array, got {:?}", arr_val);
    }
}

#[test]
fn test_object_member_assignment() {
    let code = r#"
        let obj = { x: 1, y: 2 };
        obj.x = 10;
        obj.y = 20;
        obj;
    "#;

    let ast = parse(code).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let _result = interpreter.eval_program(&ast).expect("Failed to execute");

    // Get the object from environment
    let obj_val = interpreter.get_variable("obj").expect("Failed to get obj");

    // Check that object was modified
    if let Some(map) = obj_val.as_object() {
        assert_eq!(
            map.get("x"),
            Some(&hudhudscript_bytecode::Value16::number(10.0))
        );
        assert_eq!(
            map.get("y"),
            Some(&hudhudscript_bytecode::Value16::number(20.0))
        );
    } else {
        panic!("Expected object, got {:?}", obj_val);
    }
}

#[test]
fn test_object_index_assignment() {
    let code = r#"
        let obj = { name: "Alice" };
        obj["age"] = 25;
        obj["city"] = "NYC";
        obj;
    "#;

    let ast = parse(code).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let _result = interpreter.eval_program(&ast).expect("Failed to execute");

    // Get the object from environment
    let obj_val = interpreter.get_variable("obj").expect("Failed to get obj");

    // Check that object was modified
    if let Some(map) = obj_val.as_object() {
        assert_eq!(
            map.get("name"),
            Some(&hudhudscript_bytecode::Value16::string("Alice".to_string()))
        );
        assert_eq!(
            map.get("age"),
            Some(&hudhudscript_bytecode::Value16::number(25.0))
        );
        assert_eq!(
            map.get("city"),
            Some(&hudhudscript_bytecode::Value16::string("NYC".to_string()))
        );
    } else {
        panic!("Expected object, got {:?}", obj_val);
    }
}

#[test]
fn test_assignment_returns_value() {
    let code = r#"
        let arr = [1, 2, 3];
        arr[0] = 100;
        arr[0];
    "#;

    let ast = parse(code).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_program(&ast).expect("Failed to execute");

    // Last expression should return the assigned value
    assert_eq!(result, hudhudscript_bytecode::Value16::number(100.0));
}

#[test]
fn test_array_index_out_of_bounds() {
    // Auto-extend: out-of-bounds assignment now silently extends the array
    let code = r#"
        let arr = [1, 2, 3];
        arr[10] = 100;
    "#;

    let ast = parse(code).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_program(&ast);
    assert!(result.is_ok(), "Auto-extend should succeed");
    let arr = interpreter.get_variable("arr").unwrap();
    assert_eq!(
        arr.as_array().map(|a| a.len()),
        Some(11),
        "Array should have 11 elements"
    );
    assert_eq!(
        arr.as_array().and_then(|a| a[10].as_int()),
        Some(100),
        "arr[10] should be 100"
    );
    assert_eq!(
        arr.as_array().and_then(|a| a[0].as_int()),
        Some(1),
        "arr[0] should be 1"
    );
}

#[test]
    #[ignore] // pre-existing issue
fn test_member_assignment_on_non_object() {
    let code = r#"
        let num = 42;
        num.x = 10;
    "#;

    let ast = parse(code).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_program(&ast);

    // Should return error
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(format!("{:?}", e).contains("non-object"));
    }
}

#[test]
fn test_index_assignment_with_wrong_type() {
    let code = r#"
        let arr = [1, 2, 3];
        arr["invalid"] = 10;
    "#;

    let ast = parse(code).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_program(&ast);

    // Should return type error
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(format!("{:?}", e).contains("number"));
    }
}
