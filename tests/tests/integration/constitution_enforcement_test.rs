//! Constitution enforcement tests — restored, VM-backed.
//!
//! All routes go through the `vm_interpreter::Interpreter` shim, which
//! forwards `has_active_constitution`, `get_active_constitution`, and
//! `check_constitution_compliance` straight at the VM's typed
//! constitution store.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_parser::parse;

#[test]
fn test_constitution_creation_with_conditions() {
    let source = r#"
        constitution DataGovernance {
            description: "Data governance rules",
            laws: [
                {
                    name: "Size Limit",
                    description: "Data must be under 1000",
                    enforcement: mandatory,
                    rules: ["data_size < 1000"]
                },
                {
                    name: "Priority Check",
                    description: "Priority must be high",
                    enforcement: advisory,
                    rules: ["priority == high"]
                }
            ]
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();

    let result = interpreter.execute(&statements);
    assert!(result.is_ok(), "Failed to execute: {:?}", result.err());

    // Constitution also surfaces as a declaration variable on the VM
    // side (via `Instruction::DeclStore`) — pre-existing behaviour.

    let active = interpreter.get_active_constitution();
    assert!(active.is_some(), "No active constitution");

    let active_const = active.unwrap();
    assert_eq!(active_const.name, "DataGovernance");
    assert_eq!(active_const.laws.len(), 2);

    // Verify conditions were parsed (non-empty for both laws).
    for law in active_const.laws.values() {
        assert!(
            !law.conditions.is_empty(),
            "Law {} has no conditions",
            law.name
        );
    }
}

#[test]
fn test_enforcement_check_compliant() {
    let source = r#"
        constitution TestConst {
            description: "Test",
            laws: [
                {
                    name: "Size Limit",
                    description: "Data must be under 1000",
                    enforcement: mandatory,
                    rules: ["data_size < 1000"]
                }
            ]
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    interpreter.execute(&statements).expect("Failed to execute");

    use hudhudscript_governance::enforcement::EvaluationContext;
    use serde_json::json;

    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(500));

    let result = interpreter.check_constitution_compliance(&context);
    assert!(
        result.is_ok(),
        "Compliant action was blocked: {:?}",
        result.err()
    );
}

#[test]
fn test_enforcement_check_violation() {
    let source = r#"
        constitution TestConst {
            description: "Test",
            laws: [
                {
                    name: "Size Limit",
                    description: "Data must be under 1000",
                    enforcement: mandatory,
                    rules: ["data_size < 1000"]
                }
            ]
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    interpreter.execute(&statements).expect("Failed to execute");

    use hudhudscript_governance::enforcement::EvaluationContext;
    use serde_json::json;

    let mut context = EvaluationContext::new();
    context.insert("data_size".to_string(), json!(1500));

    let result = interpreter.check_constitution_compliance(&context);
    assert!(result.is_err(), "Non-compliant action was allowed");

    let err = result.err().unwrap();
    let violation_count = err.context.iter().filter(|(k, _)| k == "violation").count();
    assert!(
        violation_count > 0,
        "No violations reported in error context: {:?}",
        err
    );
}

#[test]
fn test_multiple_conditions_all_must_pass() {
    let source = r#"
        constitution TestConst {
            description: "Test",
            laws: [
                {
                    name: "Range Check",
                    description: "Value16 must be between 0 and 100",
                    enforcement: mandatory,
                    rules: ["value >= 0", "value <= 100"]
                }
            ]
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    interpreter.execute(&statements).expect("Failed to execute");

    use hudhudscript_governance::enforcement::EvaluationContext;
    use serde_json::json;

    let mut ctx1 = EvaluationContext::new();
    ctx1.insert("value".to_string(), json!(50));
    assert!(interpreter.check_constitution_compliance(&ctx1).is_ok());

    let mut ctx2 = EvaluationContext::new();
    ctx2.insert("value".to_string(), json!(-10));
    assert!(interpreter.check_constitution_compliance(&ctx2).is_err());

    let mut ctx3 = EvaluationContext::new();
    ctx3.insert("value".to_string(), json!(150));
    assert!(interpreter.check_constitution_compliance(&ctx3).is_err());
}

#[test]
fn test_advisory_laws_dont_block() {
    let source = r#"
        constitution TestConst {
            description: "Test",
            laws: [
                {
                    name: "Advisory Rule",
                    description: "Recommended but not required",
                    enforcement: advisory,
                    rules: ["priority == high"]
                }
            ]
        }
    "#;

    let statements = parse(source).expect("Failed to parse");
    let mut interpreter = Interpreter::new();
    interpreter.execute(&statements).expect("Failed to execute");

    use hudhudscript_governance::enforcement::EvaluationContext;
    use serde_json::json;

    let mut context = EvaluationContext::new();
    context.insert("priority".to_string(), json!("low"));

    let result = interpreter.check_constitution_compliance(&context);
    assert!(result.is_ok(), "Advisory violation blocked action");
}
