//! Advanced governance enforcement tests — restored, VM-backed.
//! Verifies that advanced conditions (IN / AND / OR / grouped) parse and
//! land in the VM's typed constitution store (surfaced via
//! `has_active_constitution`).

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_parser::parse;

#[test]
fn test_in_operator_enforcement() {
    let source = r#"
        constitution RoleGovernance {
            description: "Role-based access control",
            laws: [
                {
                    name: "Admin Only",
                    description: "Only admins and moderators can access",
                    enforcement: mandatory,
                    rules: ["role in [admin, moderator]"]
                }
            ]
        }

        let x = 1;
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
    assert!(
        interpreter.has_active_constitution(),
        "Constitution should be active"
    );
}

#[test]
fn test_and_operator_enforcement() {
    let source = r#"
        constitution RangeGovernance {
            description: "Temperature range validation",
            laws: [
                {
                    name: "Valid Range",
                    description: "Temperature must be between 0 and 1",
                    enforcement: mandatory,
                    rules: ["temperature >= 0 AND temperature <= 1"]
                }
            ]
        }

        let x = 1;
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
    assert!(interpreter.has_active_constitution());
}

#[test]
fn test_or_operator_enforcement() {
    let source = r#"
        constitution AccessGovernance {
            description: "Access control with multiple conditions",
            laws: [
                {
                    name: "Admin or High Priority",
                    description: "Allow if admin or high priority",
                    enforcement: mandatory,
                    rules: ["role == admin OR priority > 5"]
                }
            ]
        }

        let x = 1;
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
    assert!(interpreter.has_active_constitution());
}

#[test]
fn test_complex_expression_enforcement() {
    let source = r#"
        constitution ComplexGovernance {
            description: "Complex logic with parentheses",
            laws: [
                {
                    name: "Complex Rule",
                    description: "Complex condition with grouping",
                    enforcement: mandatory,
                    rules: ["(temperature > 0.5 AND temperature < 1.0) OR priority == high"]
                }
            ]
        }

        let x = 1;
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
    assert!(interpreter.has_active_constitution());
}

#[test]
fn test_nested_expression_enforcement() {
    let source = r#"
        constitution NestedGovernance {
            description: "Deeply nested logic",
            laws: [
                {
                    name: "Nested Rule",
                    description: "Nested conditions",
                    enforcement: mandatory,
                    rules: ["((role == admin OR role == moderator) AND status == active) OR priority > 8"]
                }
            ]
        }

        let x = 1;
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
    assert!(interpreter.has_active_constitution());
}
