//! Integration tests for ID generator and validator.

use hudhudscript_governance::id_generator::IdGenerator;
use hudhudscript_governance::id_validator::{
    validate_constitution_id, validate_law_id, validate_rule_id,
};

#[test]
fn test_generated_constitution_ids_are_valid() {
    let generator = IdGenerator::new();

    // Generate 100 constitution IDs and verify they all pass validation
    for _ in 0..100 {
        let id = generator.next_constitution_id();
        assert!(
            validate_constitution_id(&id),
            "Generated constitution ID '{}' failed validation",
            id
        );
    }
}

#[test]
fn test_generated_law_ids_are_valid() {
    let generator = IdGenerator::new();

    // Generate law IDs for different constitutions and verify they all pass validation
    for i in 1..=10 {
        let constitution_id = format!("cons.{}", i);
        for _ in 0..10 {
            let id = generator.next_law_id(&constitution_id);
            assert!(
                validate_law_id(&id),
                "Generated law ID '{}' failed validation",
                id
            );
        }
    }
}

#[test]
fn test_generated_rule_ids_are_valid() {
    let generator = IdGenerator::new();

    // Generate 100 rule IDs and verify they all pass validation
    for _ in 0..100 {
        let id = generator.next_rule_id();
        assert!(
            validate_rule_id(&id),
            "Generated rule ID '{}' failed validation",
            id
        );
    }
}

#[test]
fn test_generator_with_custom_start_values() {
    let generator = IdGenerator::with_start_values(1000, 2000, 3000, 4000, 5000, 6000);

    // Verify constitution IDs
    let const_id = generator.next_constitution_id();
    assert_eq!(const_id, "cons.1000");
    assert!(validate_constitution_id(&const_id));

    // Verify law IDs
    let law_id = generator.next_law_id("cons.1000");
    assert_eq!(law_id, "cons1000.law2000");
    assert!(validate_law_id(&law_id));

    // Verify rule IDs
    let rule_id = generator.next_rule_id();
    assert_eq!(rule_id, "rule.3000");
    assert!(validate_rule_id(&rule_id));
}

#[test]
fn test_law_id_hierarchy_consistency() {
    let generator = IdGenerator::new();

    // Generate a constitution ID
    let const_id = generator.next_constitution_id();
    assert_eq!(const_id, "cons.1");

    // Generate law IDs for this constitution
    let law1 = generator.next_law_id(&const_id);
    let law2 = generator.next_law_id(&const_id);
    let law3 = generator.next_law_id(&const_id);

    // Verify all law IDs are valid
    assert!(validate_law_id(&law1));
    assert!(validate_law_id(&law2));
    assert!(validate_law_id(&law3));

    // Verify law IDs contain the parent constitution number
    assert_eq!(law1, "cons1.law1");
    assert_eq!(law2, "cons1.law2");
    assert_eq!(law3, "cons1.law3");
}

#[test]
fn test_concurrent_generation_produces_valid_ids() {
    use std::sync::Arc;
    use std::thread;

    let generator = Arc::new(IdGenerator::new());
    let mut handles = vec![];

    // Spawn 5 threads, each generating different types of IDs
    for _ in 0..5 {
        let gen = Arc::clone(&generator);
        let handle = thread::spawn(move || {
            let mut results = Vec::new();

            for _ in 0..20 {
                let const_id = gen.next_constitution_id();
                results.push((
                    "constitution",
                    const_id.clone(),
                    validate_constitution_id(&const_id),
                ));

                let law_id = gen.next_law_id("cons.1");
                results.push(("law", law_id.clone(), validate_law_id(&law_id)));

                let rule_id = gen.next_rule_id();
                results.push(("rule", rule_id.clone(), validate_rule_id(&rule_id)));
            }

            results
        });
        handles.push(handle);
    }

    // Collect and verify all results
    for handle in handles {
        let results = handle.join().unwrap();
        for (id_type, id, is_valid) in results {
            assert!(
                is_valid,
                "Generated {} ID '{}' failed validation",
                id_type, id
            );
        }
    }
}
