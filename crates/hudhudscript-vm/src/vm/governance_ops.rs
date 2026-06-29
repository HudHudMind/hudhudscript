//! Shared governance operations (Kural 7).
//!
//! Previously the VM carried a subset copy of the interpreter's
//! rule parser (missing OR/AND/IN/parentheses) plus its own
//! `value_to_constitution` / `value_to_eval_context` /
//! `value_to_serde_json` helpers, and the interpreter kept the full rule
//! parser inside `hudhudscript_interpreter::integration::governance`.
//! Two parsers, two value-conversion paths, identical goals. This module
//! is the single source of truth for:
//!
//! - [`parse_rule_to_condition`] — full AND/OR/IN/parentheses-aware string
//!   rule parser (ported from the interpreter's version, which was the
//!   canonical one; the VM's simpler variant is now superseded).
//! - [`value_to_constitution`] — build a
//!   [`hudhudscript_governance::Constitution`] from any script-level
//!   constitution `Value::Object`. Value16-based so both
//!   runtimes call the same code.
//! - [`value_to_eval_context`] — build a
//!   [`hudhudscript_governance::enforcement::EvaluationContext`] from a
//!   script-level action `Value::Object`.
//! - [`value_to_serde_json`] — recursive conversion helper used by the
//!   two builders above; also useful anywhere a shared builtin needs to
//!   hand a script `Value` over to a JSON-based library.
//!
//! AST-level governance helpers (e.g. the interpreter's `expr_to_condition`
//! that converts a typed AST expression tree into a `Condition`) stay in
//! `hudhudscript_interpreter::integration::governance` because
//! shared-builtins cannot depend on the AST crate.

use std::collections::HashMap;

use hudhudscript_governance::{
    enforcement::EvaluationContext, Condition, Constitution, EnforcementLevel, Law, Utc,
};
use serde_json::json;

use hudhudscript_bytecode::Value16;

// ── Rule-string parser ─────────────────────────────────────────────────────

/// Parse a string rule into a [`Condition`].
///
/// Supported grammar (case-insensitive logical operators):
///
/// ```text
/// rule        := or_expr
/// or_expr     := and_expr ( " OR " and_expr )*
/// and_expr    := atom ( " AND " atom )*
/// atom        := "(" rule ")"
///              | field " IN " "[" value ("," value)* "]"
///              | field op value
/// op          := "<" | "<=" | ">" | ">=" | "==" | "!="
/// ```
///
/// Boolean / numeric / string literals are accepted on the value side;
/// quoted strings (`"admin"`) are preserved literally, unquoted words are
/// passed through as JSON strings. Returns `None` on any unrecognised
/// shape so callers can fall through to a default / fail-closed branch.
pub fn parse_rule_to_condition(rule: &str) -> Option<Condition> {
    let rule = rule.trim();
    let rule_upper = rule.to_uppercase();

    // OR has the lowest precedence — split first.
    if let Some(or_pos) = find_logical_operator(&rule_upper, " OR ") {
        let left = &rule[..or_pos];
        let right = &rule[or_pos + 4..];
        if let (Some(l), Some(r)) = (
            parse_rule_to_condition(left),
            parse_rule_to_condition(right),
        ) {
            return Some(Condition::Or(vec![l, r]));
        }
    }

    // AND next.
    if let Some(and_pos) = find_logical_operator(&rule_upper, " AND ") {
        let left = &rule[..and_pos];
        let right = &rule[and_pos + 5..];
        if let (Some(l), Some(r)) = (
            parse_rule_to_condition(left),
            parse_rule_to_condition(right),
        ) {
            return Some(Condition::And(vec![l, r]));
        }
    }

    // Parenthesised atom.
    if rule.starts_with('(') && rule.ends_with(')') {
        let inner = &rule[1..rule.len() - 1];
        return parse_rule_to_condition(inner);
    }

    // IN operator.
    if let Some(in_pos) = rule_upper.find(" IN ") {
        let field = rule[..in_pos].trim().to_string();
        let values_str = rule[in_pos + 4..].trim();
        if values_str.starts_with('[') && values_str.ends_with(']') {
            let inner = &values_str[1..values_str.len() - 1];
            let values: Vec<serde_json::Value> = inner
                .split(',')
                .filter_map(|s| {
                    let s = s.trim();
                    if s.starts_with('"') && s.ends_with('"') {
                        Some(json!(s.trim_matches('"')))
                    } else if let Ok(num) = s.parse::<f64>() {
                        Some(json!(num))
                    } else if s == "true" {
                        Some(json!(true))
                    } else if s == "false" {
                        Some(json!(false))
                    } else if !s.is_empty() {
                        Some(json!(s))
                    } else {
                        None
                    }
                })
                .collect();
            if !values.is_empty() {
                return Some(Condition::In { field, values });
            }
        }
    }

    // Comparison operators. Order matters: <= / >= / != / == before the
    // single-char variants so we don't accidentally split at the first `<`.
    if let Some(pos) = rule.find("<=") {
        let field = rule[..pos].trim().to_string();
        let value_str = rule[pos + 2..].trim();
        if let Ok(value) = value_str.parse::<f64>() {
            return Some(Condition::Not(Box::new(Condition::GreaterThan {
                field,
                value,
            })));
        }
    } else if let Some(pos) = rule.find(">=") {
        let field = rule[..pos].trim().to_string();
        let value_str = rule[pos + 2..].trim();
        if let Ok(value) = value_str.parse::<f64>() {
            return Some(Condition::Not(Box::new(Condition::LessThan {
                field,
                value,
            })));
        }
    } else if let Some(pos) = rule.find("!=") {
        let field = rule[..pos].trim().to_string();
        let value_str = rule[pos + 2..].trim();
        return Some(Condition::NotEquals {
            field,
            value: parse_rule_value(value_str),
        });
    } else if let Some(pos) = rule.find("==") {
        let field = rule[..pos].trim().to_string();
        let value_str = rule[pos + 2..].trim();
        return Some(Condition::Equals {
            field,
            value: parse_rule_value(value_str),
        });
    } else if let Some(pos) = rule.find('<') {
        let field = rule[..pos].trim().to_string();
        let value_str = rule[pos + 1..].trim();
        if let Ok(value) = value_str.parse::<f64>() {
            return Some(Condition::LessThan { field, value });
        }
    } else if let Some(pos) = rule.find('>') {
        let field = rule[..pos].trim().to_string();
        let value_str = rule[pos + 1..].trim();
        if let Ok(value) = value_str.parse::<f64>() {
            return Some(Condition::GreaterThan { field, value });
        }
    }

    None
}

/// Find a logical operator substring that sits outside any parentheses.
fn find_logical_operator(rule: &str, operator: &str) -> Option<usize> {
    let mut depth: i32 = 0;
    let rule_bytes = rule.as_bytes();
    let op_bytes = operator.as_bytes();
    let mut i = 0;
    while i < rule_bytes.len() {
        let b = rule_bytes[i];
        if b == b'(' {
            depth += 1;
        } else if b == b')' {
            depth -= 1;
        } else if depth == 0
            && i + op_bytes.len() <= rule_bytes.len()
            && &rule_bytes[i..i + op_bytes.len()] == op_bytes
        {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Parse a right-hand-side value token from a comparison rule into JSON.
/// Quoted strings keep their content; numeric tokens become JSON numbers;
/// `true` / `false` become JSON booleans; everything else stays a string.
pub fn parse_rule_value(value_str: &str) -> serde_json::Value {
    if value_str.starts_with('"') && value_str.ends_with('"') {
        json!(value_str.trim_matches('"'))
    } else if let Ok(num) = value_str.parse::<f64>() {
        json!(num)
    } else if value_str == "true" {
        json!(true)
    } else if value_str == "false" {
        json!(false)
    } else {
        json!(value_str)
    }
}

// ── Value → governance-type converters ────────────────────────────────────

/// Recursively convert a script `Value16` into a `serde_json::Value`.
///
/// Set/Map variants are unfolded to their array/object equivalents so the
/// governance engine (which only speaks plain JSON) can evaluate rules
/// against them uniformly. Unsupported variants (functions, promises, etc.)
/// collapse to `null` to keep rule evaluation fail-safe.
pub fn value_to_serde_json(val: &Value16) -> serde_json::Value {
    if val.is_null() {
        return serde_json::Value::Null;
    }
    if let Some(b) = val.as_bool() {
        return json!(b);
    }
    if let Some(n) = val.as_number() {
        return json!(n);
    }
    if let Some(s) = val.as_str() {
        return json!(s);
    }
    if let Some(arr) = val.as_array() {
        return serde_json::Value::Array(arr.iter().map(value_to_serde_json).collect());
    }
    if let Some(obj) = val.as_object() {
        let map: serde_json::Map<String, serde_json::Value> = obj
            .iter()
            .map(|(k, v)| (k.clone(), value_to_serde_json(v)))
            .map(|(k, v)| (k.to_string(), v)).collect();
        return serde_json::Value::Object(map);
    }
    if let Some(items) = val.as_set() {
        return serde_json::Value::Array(items.iter().map(value_to_serde_json).collect());
    }
    if let Some(pairs) = val.as_map_pairs() {
        let map: serde_json::Map<String, serde_json::Value> = pairs
            .iter()
            .map(|(k, v)| (k.display_string(), value_to_serde_json(v)))
            .map(|(k, v)| (k.to_string(), v)).collect();
        return serde_json::Value::Object(map);
    }
    serde_json::Value::Null
}

/// Build a governance [`EvaluationContext`] from a script-level action
/// `Value::Object`. Top-level keys become context fields; values are
/// recursively JSON-encoded.
pub fn value_to_eval_context(action: &Value16) -> EvaluationContext {
    let mut ctx = EvaluationContext::new();
    if let Some(obj) = action.as_object() {
        for (key, val) in obj {
            ctx.insert(key.to_string(), value_to_serde_json(val));
        }
    }
    ctx
}

/// Build a [`Constitution`] from a script-level object.
///
/// Expected shape:
/// ```text
/// {
///   laws: [
///     {
///       name: "max_tokens",
///       enforcement_level: "mandatory",
///       rules: ["data_size < 1000", "priority == high"]
///     },
///     …
///   ]
/// }
/// ```
///
/// Missing / malformed pieces fall through to defaults so partially-correct
/// scripts still evaluate (advisory level, empty rule set, etc.). Rule
/// strings go through [`parse_rule_to_condition`]; unrecognised rule
/// values are fail-closed — any unknown shape becomes an always-fail
/// condition, matching GOV-001 in the VM's original handler.
pub fn value_to_constitution(name: &str, val: &Value16) -> Constitution {
    let mut laws = HashMap::new();

    if let Some(obj) = val.as_object() {
        if let Some(law_arr_val) = obj.get("laws") {
            if let Some(law_arr) = law_arr_val.as_array() {
                for (idx, law_val) in law_arr.iter().enumerate() {
                    let Some(law_obj) = law_val.as_object() else {
                        continue;
                    };

                    let law_name = law_obj
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("law_{}", idx));

                    let enforcement_str = law_obj
                        .get("enforcement_level")
                        .and_then(|v| v.as_str())
                        .unwrap_or("advisory");

                    let enforcement_level = match enforcement_str {
                        "mandatory" => EnforcementLevel::Mandatory,
                        "optional" => EnforcementLevel::Optional,
                        _ => EnforcementLevel::Advisory,
                    };

                    let mut conditions = Vec::new();
                    if let Some(rules_val) = law_obj.get("rules") {
                        if let Some(rules) = rules_val.as_array() {
                            for rule in rules {
                                if let Some(rule_str) = rule.as_str() {
                                    if let Some(cond) = parse_rule_to_condition(rule_str) {
                                        conditions.push(cond);
                                    } else {
                                        // GOV-001 fail-closed: unparseable rule
                                        // strings (unknown grammar / format) are
                                        // treated as always-fail. Previously we
                                        // silently dropped them, which allowed
                                        // an unknown-format rule to behave as
                                        // "no law" and let actions through
                                        // (fail-open, security bug).
                                        conditions.push(Condition::Equals {
                                            field: "__always_fail".to_string(),
                                            value: json!(true),
                                        });
                                    }
                                } else if let Some(b) = rule.as_bool() {
                                    if !b {
                                        // `false` rule is a hard-fail marker.
                                        conditions.push(Condition::Equals {
                                            field: "__always_fail".to_string(),
                                            value: json!(true),
                                        });
                                    }
                                    // `true` rule → no condition (trivially passes).
                                } else {
                                    // Any other shape → fail-closed (GOV-001).
                                    conditions.push(Condition::Equals {
                                        field: "__always_fail".to_string(),
                                        value: json!(true),
                                    });
                                }
                            }
                        }
                    }

                    let law_id = format!("{}.law{}", name, idx);
                    laws.insert(
                        law_id.clone(),
                        Law {
                            id: law_id,
                            constitution_id: name.to_string(),
                            name: law_name,
                            description: String::new(),
                            enforcement_level,
                            conditions,
                        },
                    );
                }
            }
        }
    }

    Constitution {
        id: name.to_string(),
        name: name.to_string(),
        description: None,
        laws,
        created_at: Utc::now(),
        version: 1,
    }
}
