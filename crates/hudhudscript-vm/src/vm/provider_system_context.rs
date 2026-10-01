use crate::vm::VM;
use hudhudscript_bytecode::ObjMap;

/// Format the active constitution and its laws/rules into a system context block.
pub fn format_active_constitution_system_context(vm: &VM) -> Option<String> {
    let active_name = vm.active_constitution.as_ref()?;
    let constitution = vm.constitutions.get(active_name)?;

    let decl_key = format!("constitution:{}", active_name);
    let raw_cons_obj = vm.declarations.get(&decl_key).and_then(|v| v.as_object());

    let mut out = String::new();
    out.push_str("[Constitution]\n");
    if !constitution.id.is_empty() {
        out.push_str(&format!("Name: {}\n", active_name));
        out.push_str(&format!("Id: {}\n", constitution.id));
    } else {
        out.push_str(&format!("Name: {}\n", active_name));
    }

    if let Some(desc) = &constitution.description {
        out.push_str(&format!("Description: {}\n", desc));
    }

    for law in constitution.laws.values() {
        out.push_str("\n[Law]\n");
        out.push_str(&format!("Name: {}\n", law.name));
        out.push_str(&format!("Description: {}\n", law.description));
        out.push_str(&format!("Enforcement: {:?}\n", law.enforcement_level));

        let mut raw_rules = Vec::new();
        if let Some(cons_obj) = &raw_cons_obj {
            if let Some(laws_val) = cons_obj.get("laws") {
                if let Some(laws_arr) = laws_val.as_array() {
                    for law_item in laws_arr {
                        if let Some(law_obj) = law_item.as_object() {
                            if let Some(name_val) = law_obj.get("name") {
                                if name_val.as_str() == Some(law.name.as_str()) {
                                    if let Some(rules_val) = law_obj.get("rules") {
                                        if let Some(rules_arr) = rules_val.as_array() {
                                            for r in rules_arr {
                                                if let Some(s) = r.as_string() {
                                                    raw_rules.push(s.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !raw_rules.is_empty() {
            out.push_str("Rules:\n");
            for r in &raw_rules {
                out.push_str(&format!("- {}\n", r));
            }
        } else if !law.conditions.is_empty() {
            out.push_str("Rules:\n");
            for cond in &law.conditions {
                out.push_str(&format!("- {:?}\n", cond));
            }
        }
    }

    Some(out.trim_end().to_string())
}

/// Extract agent role/system/instruction fields from the agent object.
pub fn format_agent_system_context(agent_obj: &hudhudscript_bytecode::ObjMap) -> Option<String> {
    let fields = [
        "role",
        "instruction",
        "instructions",
        "system",
        "system_prompt",
        "persona",
        "policy",
        "constraints",
    ];

    let mut found_texts = Vec::new();

    for field in fields {
        if let Some(val) = agent_obj.get(field) {
            if let Some(s) = val.as_string() {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    found_texts.push(trimmed.to_string());
                }
            }
        }
    }

    if found_texts.is_empty() {
        return None;
    }

    let mut out = String::new();
    out.push_str("[Agent Role]\n");

    // Deduplicate strings in case user defines multiple fields with same text
    let mut unique_texts = Vec::new();
    for text in found_texts {
        if !unique_texts.contains(&text) {
            unique_texts.push(text);
        }
    }

    out.push_str(&unique_texts.join("\n\n"));
    Some(out)
}

pub fn format_provider_system_context(
    provider_obj: &hudhudscript_bytecode::ObjMap,
) -> Option<String> {
    let fields = [
        "system",
        "system_prompt",
        "instructions",
        "policy",
        "constraints",
    ];

    let mut found_texts = Vec::new();

    for field in fields {
        if let Some(val) = provider_obj.get(field) {
            if let Some(s) = val.as_string() {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    found_texts.push(trimmed.to_string());
                }
            }
        }
    }

    if found_texts.is_empty() {
        return None;
    }

    let mut out = String::new();
    out.push_str("[Provider System]\n");

    let mut unique_texts = Vec::new();
    for text in found_texts {
        if !unique_texts.contains(&text) {
            unique_texts.push(text);
        }
    }

    out.push_str(&unique_texts.join("\n\n"));
    Some(out)
}

/// Compose the final system prompt.
/// Order: 1. Constitution, 2. Provider System, 3. Agent Role, 4. Call-site System Prompt.
pub fn compose_system_prompt(parts: Vec<String>) -> Option<String> {
    let mut unique_parts = Vec::new();

    for part in parts {
        let trimmed = part.trim();
        if !trimmed.is_empty() && !unique_parts.contains(&trimmed.to_string()) {
            unique_parts.push(trimmed.to_string());
        }
    }

    if unique_parts.is_empty() {
        None
    } else {
        Some(unique_parts.join("\n\n"))
    }
}
