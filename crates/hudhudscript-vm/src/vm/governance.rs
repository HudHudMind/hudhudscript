use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::Value16;
use hudhudscript_governance::enforcement::EvaluationContext;
use hudhudscript_governance::{Condition, Constitution};

impl crate::vm::VM {
    /// `value_to_constitution`, `value_to_eval_context`,
    /// `value_to_serde_json`, `parse_rule_to_condition`, and
    /// `parse_rule_value` used to live inline here — the VM carried a
    /// strict subset of the interpreter's rule parser (no OR/AND/IN/
    /// parentheses) and a duplicated constitution builder. They now live
    /// in `crate::vm::governance_ops`; both runtimes
    /// call through the same code, which means the VM transparently gains
    /// OR/AND/IN/parentheses support and any future rule-language fix
    /// lands in exactly one place.
    ///
    /// Backwards-compatible thin wrappers stay here so the existing call
    /// sites inside the VM don't need to be touched. Each wrapper is a
    /// one-line delegation to the shared implementation.

    pub(crate) fn value_to_constitution(name: &str, val: &Value16) -> Constitution {
        crate::vm::governance_ops::value_to_constitution(name, val)
    }

    pub(crate) fn value_to_eval_context(action: &Value16) -> EvaluationContext {
        crate::vm::governance_ops::value_to_eval_context(action)
    }

    #[allow(dead_code)]
    pub(crate) fn value_to_serde_json(val: &Value16) -> serde_json::Value {
        crate::vm::governance_ops::value_to_serde_json(val)
    }

    #[allow(dead_code)]
    pub(crate) fn parse_rule_to_condition(rule: &str) -> Option<Condition> {
        crate::vm::governance_ops::parse_rule_to_condition(rule)
    }

    /// Check that a bytecode Value matches the expected type name (Issue #452).
    ///
    /// `expected_type` is a lowercase string such as "number", "string",
    /// "boolean", "null", "array", or a union like "number|string".
    pub(crate) fn check_bytecode_value_type(
        value: &Value16,
        expected_type: &str,
        var_name: &str,
    ) -> CompileResult<()> {
        if expected_type == "any" {
            return Ok(());
        }

        // Handle union types: "number|string"
        if expected_type.contains('|') {
            let any_match = expected_type
                .split('|')
                .any(|t| Self::check_bytecode_value_type(value, t.trim(), var_name).is_ok());
            if any_match {
                return Ok(());
            }
            let found = Self::bytecode_value_type_name(value);
            return Err(compile_codes::runtime_error(format!(
                "Type error: expected {}, got {} in declaration of '{}'",
                expected_type, found, var_name,
            )));
        }

        let found = Self::bytecode_value_type_name(value);
        let matches = match expected_type {
            "number" => value.as_number().is_some() || value.as_int().is_some(),
            "string" => value.as_string().is_some(),
            "boolean" => value.as_bool().is_some(),
            "null" => value.is_null(),
            "array" => value.as_array().is_some(),
            "object" => value.as_object().is_some(),
            "tool" => value.as_tool_ref().is_some(),
            "resource" => value.as_resource_ref().is_some(),
            "server" => value.as_object().is_some(),
            _ => true, // unknown / generic → skip check
        };

        if matches {
            Ok(())
        } else {
            Err(compile_codes::runtime_error(format!(
                "Type error: expected {}, got {} in declaration of '{}'",
                expected_type, found, var_name,
            )))
        }
    }

    /// Return the lowercase runtime type name of a bytecode Value.
    pub(crate) fn bytecode_value_type_name(value: &Value16) -> &'static str {
        let v = value;
        match v {
            // A3a: Int reports legacy "number" for backward compat.
            _ if value.is_int() || value.is_number() => "number",
            _ if value.as_string().is_some() => "string",
            _ if value.is_bool() => "boolean",
            _ if value.is_null() => "null",
            _ if value.as_array().is_some() => "array",
            _ if value.as_object().is_some() => "object",
            _ if value.as_function_data().is_some() => "function",
            _ if value.as_promise_state().is_some() => "promise",
            _ if value.as_tool_ref().is_some() => "tool",
            _ if value.as_resource_ref().is_some() => "resource",
            _ if value.as_class_data().is_some() => "class",
            _ if value.as_instance_data().is_some() => "instance",
            _ if value.as_data_data().is_some() => "data",
            _ if value.as_option().is_some() => "option",
            _ if value.as_result().is_some() => "result",
            _ if value.as_generator_state().is_some() => "generator",
            _ if value.as_set().is_some() => "set",
            _ if value.as_map_pairs().is_some() => "map",

            _ => "unknown",
        }
    }
}
// Note: governance.rs fully extracted from vm/mod.rs
