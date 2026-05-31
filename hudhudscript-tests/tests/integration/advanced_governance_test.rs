//! Advanced governance parsing tests
//! Tests for complex expressions, IN operator, AND/OR logic, and parentheses

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_parser::parse;

#[test]
fn test_in_operator_parsing() {
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
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_and_operator_parsing() {
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
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_or_operator_parsing() {
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
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_parentheses_parsing() {
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
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_nested_parentheses_parsing() {
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
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_in_operator_with_numbers() {
    let source = r#"
        constitution NumericGovernance {
            description: "Numeric IN operator",
            laws: [
                {
                    name: "Valid Codes",
                    description: "Status code must be valid",
                    enforcement: mandatory,
                    rules: ["status_code in [200, 201, 204]"]
                }
            ]
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_in_operator_with_booleans() {
    let source = r#"
        constitution BooleanGovernance {
            description: "Boolean IN operator",
            laws: [
                {
                    name: "Valid Flags",
                    description: "Flag must be valid",
                    enforcement: mandatory,
                    rules: ["enabled in [true, false]"]
                }
            ]
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_multiple_and_conditions() {
    let source = r#"
        constitution MultiConditionGovernance {
            description: "Multiple AND conditions",
            laws: [
                {
                    name: "All Conditions",
                    description: "All conditions must be met",
                    enforcement: mandatory,
                    rules: ["temperature > 0 AND temperature < 1 AND priority > 3 AND status == active"]
                }
            ]
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_multiple_or_conditions() {
    let source = r#"
        constitution MultiOrGovernance {
            description: "Multiple OR conditions",
            laws: [
                {
                    name: "Any Condition",
                    description: "Any condition can be met",
                    enforcement: mandatory,
                    rules: ["role == admin OR role == moderator OR role == superuser OR priority > 9"]
                }
            ]
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}

#[test]
fn test_mixed_and_or_precedence() {
    let source = r#"
        constitution PrecedenceGovernance {
            description: "AND has higher precedence than OR",
            laws: [
                {
                    name: "Precedence Rule",
                    description: "Test operator precedence",
                    enforcement: mandatory,
                    rules: ["role == admin AND status == active OR priority > 8"]
                }
            ]
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());
}
