//! VM provider system context: constitution/law formatting, agent-role
//! extraction, deduplication, and system-prompt composition order.

use hudhudscript_vm::vm::provider_system_context::{
    compose_system_prompt, format_active_constitution_system_context,
    format_agent_system_context,
};
use hudhudscript_vm::vm::VM;
use hudhudscript_bytecode::Value16;
use hudhudscript_governance::{Constitution, EnforcementLevel, Law};
use std::collections::HashMap;

#[test]
fn test_format_agent_system_context() {
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert(
        "role".to_string(),
        Value16::string("Sen yaratıcı bir içerik yazarısın."),
    );

    let result = format_agent_system_context(&obj).unwrap();
    assert!(result.contains("[Agent Role]"));
    assert!(result.contains("Sen yaratıcı bir içerik yazarısın."));
}

#[test]
fn test_compose_order() {
    let parts = vec![
        "[Constitution]".to_string(),
        "[Agent Role]".to_string(),
        "Call system text".to_string(),
    ];

    let result = compose_system_prompt(parts).unwrap();
    assert_eq!(result, "[Constitution]\n\n[Agent Role]\n\nCall system text");
}

#[test]
fn test_no_duplicate_parts() {
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("role".to_string(), Value16::string("Same role"));
    obj.insert("system".to_string(), Value16::string("Same role"));

    let result = format_agent_system_context(&obj).unwrap();
    assert_eq!(result, "[Agent Role]\nSame role");
}

#[test]
fn test_empty_context() {
    let obj = hudhudscript_bytecode::ObjMap::default();
    assert!(format_agent_system_context(&obj).is_none());
    assert!(compose_system_prompt(vec![]).is_none());
    assert!(compose_system_prompt(vec!["  ".to_string()]).is_none());
}

#[test]
fn test_format_active_constitution() {
    let mut vm = VM::new();
    vm.active_constitution = Some("SafeAI".to_string());

    let mut laws = HashMap::new();
    laws.insert(
        "NoLie".to_string(),
        Law {
            id: "NoLie".to_string(),
            constitution_id: "cons.SafeAI".to_string(),
            name: "NoLie".to_string(),
            description: "Yalan söyleme.".to_string(),
            enforcement_level: EnforcementLevel::Mandatory,
            conditions: vec![],
        },
    );

    let cons = Constitution {
        id: "cons.SafeAI".to_string(),
        name: "SafeAI".to_string(),
        description: Some("Dürüst ve güvenli cevap ver.".to_string()),
        laws,
        created_at: hudhudscript_governance::Utc::now(),
        version: 1,
    };

    vm.constitutions.insert("SafeAI".to_string(), cons);

    let result = format_active_constitution_system_context(&vm).unwrap();
    assert!(result.contains("[Constitution]"));
    assert!(result.contains("SafeAI"));
    assert!(result.contains("Dürüst ve güvenli cevap ver."));
    assert!(result.contains("[Law]"));
    assert!(result.contains("NoLie"));
}
