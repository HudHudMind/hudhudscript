//! Coverage tests for hudhudscript-vm crate - FIXED VERSION
//!
//! Real assertions testing actual VM behavior, not just checking for panics.

use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

/// Helper to run source code through parser → compiler → VM
fn run_vm(source: &str) -> CompileResult<VM> {
    let stmts = parse(source).map_err(|e| {
        hudhudscript_bytecode::error::compile_codes::runtime_error(format!("Parse error: {:?}", e))
    })?;
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts)?;
    let mut vm = VM::new();
    vm.execute(&bytecode)?;
    Ok(vm)
}

/// Helper to get a variable from VM and assert it exists
fn get_var(vm: &VM, name: &str) -> Value16 {
    vm.get_variable(name)
        .cloned()
        .map(|v| v)
        .unwrap_or_else(|| panic!("Variable '{}' not found in VM", name))
}

// ------------------------------------------------------------
// Basic instructions - with REAL assertions
// ------------------------------------------------------------

#[test]
fn test_vm_load_const_number() {
    let vm = run_vm("let x = 42;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(n) = x.as_number() {
        assert!((n - 42.0).abs() < 1e-10, "Expected 42, got {}", n)
    } else {
        panic!("Expected Number, got {:?}", x)
    }
}

#[test]
fn test_vm_load_const_string() {
    let vm = run_vm(r#"let x = "hello";"#).expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(s) = x.as_str() {
        assert_eq!(s, "hello", "Expected 'hello', got '{}'", s)
    } else {
        panic!("Expected String, got {:?}", x)
    }
}

#[test]
fn test_vm_load_const_bool_true() {
    let vm = run_vm("let x = true;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(b, "Expected true, got false")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_load_const_bool_false() {
    let vm = run_vm("let x = false;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(!b, "Expected false, got true")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_load_const_null() {
    let vm = run_vm("let x = null;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if x.is_null() {
    } else {
        panic!("Expected Null, got {:?}", x);
    }
}

// ------------------------------------------------------------
// Arithmetic operations - with REAL assertions
// ------------------------------------------------------------

#[test]
fn test_vm_add_numbers() {
    let vm = run_vm("let x = 1 + 2;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(n) = x.as_number() {
        assert!((n - 3.0).abs() < 1e-10, "Expected 3, got {}", n)
    } else {
        panic!("Expected Number, got {:?}", x)
    }
}

#[test]
fn test_vm_subtract_numbers() {
    let vm = run_vm("let x = 5 - 3;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(n) = x.as_number() {
        assert!((n - 2.0).abs() < 1e-10, "Expected 2, got {}", n)
    } else {
        panic!("Expected Number, got {:?}", x)
    }
}

#[test]
fn test_vm_multiply_numbers() {
    let vm = run_vm("let x = 3 * 4;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(n) = x.as_number() {
        assert!((n - 12.0).abs() < 1e-10, "Expected 12, got {}", n)
    } else {
        panic!("Expected Number, got {:?}", x)
    }
}

#[test]
fn test_vm_divide_numbers() {
    let vm = run_vm("let x = 10 / 2;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(n) = x.as_number() {
        assert!((n - 5.0).abs() < 1e-10, "Expected 5, got {}", n)
    } else {
        panic!("Expected Number, got {:?}", x)
    }
}

#[test]
fn test_vm_modulo_numbers() {
    let vm = run_vm("let x = 10 % 3;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(n) = x.as_number() {
        assert!((n - 1.0).abs() < 1e-10, "Expected 1, got {}", n)
    } else {
        panic!("Expected Number, got {:?}", x)
    }
}

#[test]
fn test_vm_division_by_zero_returns_infinity_or_error() {
    let result = run_vm("let x = 1 / 0;");
    // Either returns Infinity or errors - both are acceptable
    // Just ensure it doesn't panic unexpectedly
    match result {
        Ok(vm) => {
            let x = get_var(&vm, "x");
            if let Some(n) = x.as_number() {
                assert!(
                    n.is_infinite() || n.is_nan(),
                    "1/0 should be Infinity or NaN, got {}",
                    n
                );
            }
        }
        Err(_) => {} // runtime error for division by zero is also acceptable
    }
}

// ------------------------------------------------------------
// Comparison operations - with REAL assertions
// ------------------------------------------------------------

#[test]
fn test_vm_equal_true() {
    let vm = run_vm("let x = 1 == 1;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(b, "Expected true for 1 == 1")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_equal_false() {
    let vm = run_vm("let x = 1 == 2;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(!b, "Expected false for 1 == 2")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_not_equal_true() {
    let vm = run_vm("let x = 1 != 2;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(b, "Expected true for 1 != 2")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_not_equal_false() {
    let vm = run_vm("let x = 1 != 1;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(!b, "Expected false for 1 != 1")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_less_than_true() {
    let vm = run_vm("let x = 1 < 2;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(b, "Expected true for 1 < 2")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_less_than_false() {
    let vm = run_vm("let x = 2 < 1;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(!b, "Expected false for 2 < 1")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_greater_than_true() {
    let vm = run_vm("let x = 2 > 1;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(b, "Expected true for 2 > 1")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_greater_than_false() {
    let vm = run_vm("let x = 1 > 2;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(!b, "Expected false for 1 > 2")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

// ------------------------------------------------------------
// Logical operations - with REAL assertions
// ------------------------------------------------------------

#[test]
fn test_vm_logical_and_true_true() {
    let vm = run_vm("let x = true && true;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(b, "Expected true for true && true")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_logical_and_true_false() {
    let vm = run_vm("let x = true && false;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(!b, "Expected false for true && false")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_logical_or_true_false() {
    let vm = run_vm("let x = true || false;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(b, "Expected true for true || false")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_logical_or_false_false() {
    let vm = run_vm("let x = false || false;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(!b, "Expected false for false || false")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_logical_not_true() {
    let vm = run_vm("let x = !true;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(!b, "Expected false for !true")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

#[test]
fn test_vm_logical_not_false() {
    let vm = run_vm("let x = !false;").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(b) = x.as_bool() {
        assert!(b, "Expected true for !false")
    } else {
        panic!("Expected Boolean, got {:?}", x)
    }
}

// ------------------------------------------------------------
// Control flow - with REAL assertions
// ------------------------------------------------------------

#[test]
fn test_vm_if_true_branch() {
    let vm = run_vm("let x = 0; if (true) { x = 1; }").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(n) = x.as_number() {
        assert!(
            (n - 1.0).abs() < 1e-10,
            "Expected 1 in true branch, got {}",
            n
        )
    } else {
        panic!("Expected Number, got {:?}", x)
    }
}

#[test]
fn test_vm_if_false_branch() {
    let vm = run_vm("let x = 0; if (false) { x = 1; } else { x = 2; }").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(n) = x.as_number() {
        assert!(
            (n - 2.0).abs() < 1e-10,
            "Expected 2 in else branch, got {}",
            n
        )
    } else {
        panic!("Expected Number, got {:?}", x)
    }
}

#[test]
fn test_vm_while_loop() {
    let vm = run_vm("let x = 0; while (x < 5) { x = x + 1; }").expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(n) = x.as_number() {
        assert!(
            (n - 5.0).abs() < 1e-10,
            "Expected 5 after while loop, got {}",
            n
        )
    } else {
        panic!("Expected Number, got {:?}", x)
    }
}

// ------------------------------------------------------------
// Functions - with REAL assertions
// ------------------------------------------------------------

#[test]
fn test_vm_function_call() {
    let vm = run_vm("function add(a, b) { return a + b; } let x = add(1, 2);")
        .expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(n) = x.as_number() {
        assert!(
            (n - 3.0).abs() < 1e-10,
            "Expected 3 from function call, got {}",
            n
        )
    } else {
        panic!("Expected Number, got {:?}", x)
    }
}

#[test]
fn test_vm_nested_function() {
    let vm = run_vm(
        "function outer() { function inner() { return 42; } return inner(); } let x = outer();",
    )
    .expect("VM should execute");
    let x = get_var(&vm, "x");
    if let Some(n) = x.as_number() {
        assert!(
            (n - 42.0).abs() < 1e-10,
            "Expected 42 from nested function, got {}",
            n
        )
    } else {
        panic!("Expected Number, got {:?}", x)
    }
}

// ------------------------------------------------------------
// Objects and Arrays - with REAL assertions
// ------------------------------------------------------------

#[test]
fn test_vm_array_literal() {
    let vm = run_vm("let arr = [1, 2, 3];").expect("VM should execute");
    let arr = get_var(&vm, "arr");
    if let Some(elements) = arr.as_array() {
        assert_eq!(elements.len(), 3, "Array should have 3 elements");
        if let (Some(n1), Some(n2), Some(n3)) = (
            elements[0].as_number(),
            elements[1].as_number(),
            elements[2].as_number(),
        ) {
            assert!((n1 - 1.0).abs() < 1e-10, "Expected 1, got {}", n1);
            assert!((n2 - 2.0).abs() < 1e-10, "Expected 2, got {}", n2);
            assert!((n3 - 3.0).abs() < 1e-10, "Expected 3, got {}", n3);
        } else {
            panic!("Array elements should be numbers")
        }
    } else {
        panic!("Expected Array, got {:?}", arr)
    }
}

#[test]
fn test_vm_object_literal() {
    let vm = run_vm(r#"let obj = { name: "test", value: 42 };"#).expect("VM should execute");
    let obj = get_var(&vm, "obj");
    if let Some(map) = obj.as_object() {
        assert_eq!(map.len(), 2, "Object should have 2 properties");
        // Check name property
        if let Some(name) = map.get("name").and_then(|v| v.as_str()) {
            assert_eq!(name, "test", "Expected name='test', got '{}'", name);
        } else {
            panic!("Object should have 'name' string property");
        }
        // Check value property
        if let Some(value) = map.get("value").and_then(|v| v.as_number()) {
            assert!(
                (value - 42.0).abs() < 1e-10,
                "Expected value=42, got {}",
                value
            );
        } else {
            panic!("Object should have 'value' number property");
        }
    } else {
        panic!("Expected Object, got {:?}", obj)
    }
}

// ------------------------------------------------------------
// Error handling - with REAL assertions
// ------------------------------------------------------------

#[test]
fn test_vm_throw_returns_error() {
    let result = run_vm(r#"throw "error";"#);
    assert!(result.is_err(), "Throw should return error");
}

#[test]
fn test_vm_undefined_variable_returns_error() {
    let result = run_vm("let x = y;");
    assert!(result.is_err(), "Undefined variable should return error");
}

// ------------------------------------------------------------
// Edge cases
// ------------------------------------------------------------

#[test]
fn test_vm_string_concatenation() {
    let vm = run_vm(r#"let s = "hello" + " " + "world";"#).expect("VM should execute");
    let s = get_var(&vm, "s");
    if let Some(str) = s.as_str() {
        assert_eq!(str, "hello world", "Expected 'hello world', got '{}'", str)
    } else {
        panic!("Expected String, got {:?}", s)
    }
}

#[test]
fn test_vm_chained_calls() {
    let vm = run_vm("function f(x) { return x + 1; } let y = f(f(1));").expect("VM should execute");
    let y = get_var(&vm, "y");
    if let Some(n) = y.as_number() {
        assert!(
            (n - 3.0).abs() < 1e-10,
            "Expected 3 from f(f(1)), got {}",
            n
        )
    } else {
        panic!("Expected Number, got {:?}", y)
    }
}

#[test]
fn test_vm_strcat_mut_optimization() {
    let vm = run_vm(r#"let s = "Hello"; s = s + " " + "World" + "!";"#).expect("VM should execute");
    let s = get_var(&vm, "s");
    if let Some(str) = s.as_str() {
        assert_eq!(str, "Hello World!", "Expected 'Hello World!', got '{}'", str)
    } else {
        panic!("Expected String, got {:?}", s)
    }
}
