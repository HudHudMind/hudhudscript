use crate::vm::config_types::{OutputLocale, SandboxConfig};
use crate::vm::mcp_dispatch::{dispatch_mcp_tool_call, McpContext};
use crate::vm::prepack::PACK_SENTINEL;
use crate::vm::provider_dispatch::{dispatch_provider_call, ProviderCallConfig, ProviderContext};
use crate::vm::registry::{BuiltinFn, ModuleRegistry};
use crate::vm::util::builtin_name_set;
use crate::vm::VM;
use hudhudscript_bytecode::cache_utils::{enforce_cache_limit, MAX_MCP_CACHE, MAX_RAG_STORE_CACHE};
use hudhudscript_bytecode::error::{compile_codes, CompileError, CompileResult, SourcePosition};
use hudhudscript_bytecode::packed_instruction;
use hudhudscript_bytecode::shared_value::{
    num_add, num_div, num_eq, num_ge, num_gt, num_le, num_lt, num_mod, num_mul, num_neg, num_sub,
};
use hudhudscript_bytecode::{
    Bytecode, ClassData, FunctionChunk, FunctionData, GeneratorState16, InstanceData, Instruction,
    PromiseState16, Value16,
};
use hudhudscript_debug::Debugger;
use hudhudscript_errors::HudHudResult;
use hudhudscript_governance::enforcement::{enforce_constitution, EvaluationContext};
use hudhudscript_governance::{Condition, Constitution};
use hudhudscript_mcp::{McpClient, Tool as McpToolDefinition};
use hudhudscript_rag::{
    DistanceMetric, EmbeddingProvider, SimpleEmbedding, VectorStore, VectorStoreConfig,
};
use hudhudscript_runtime::provider::ProviderRegistry;
use hudhudscript_tools::ToolRegistry;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Convert a float to Int if it fits in i64 (whole number), otherwise keep as Number.
fn floor_to_int_or_number(v: f64) -> Value16 {
    if v >= i64::MIN as f64 && v <= i64::MAX as f64 {
        Value16::int(v as i64)
    } else {
        Value16::number(v)
    }
}

impl crate::vm::VM {
    pub(crate) fn dispatch_builtin_group1(
        &mut self,
        name: &str,
        arg_count: u8,
        first_arg: u8,
        _bytecode: &hudhudscript_bytecode::Bytecode,
    ) -> hudhudscript_bytecode::error::CompileResult<bool> {
        match name {
            n if crate::vm::builtin_aliases::is_print_alias(n)
                | crate::vm::builtin_aliases::is_println_alias(n) =>
            {
                let mut args: Vec<Value16> = Vec::new();
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                let parts: Vec<String> = args.iter().map(|v| self.value_to_string(v)).collect();
                hudhud_print::print_ops::print_line(&parts.join(" "));
                self.registers[255] = Value16::null();
                Ok(true)
            }
            n if crate::vm::builtin_aliases::is_eprint_alias(n) => {
                let mut args: Vec<Value16> = Vec::new();
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                let parts: Vec<String> = args.iter().map(|v| self.value_to_string(v)).collect();
                hudhud_print::print_ops::eprint_str(&parts.join(" "));
                self.registers[255] = Value16::null();
                Ok(true)
            }
            n if crate::vm::builtin_aliases::is_eprintln_alias(n) => {
                let mut args: Vec<Value16> = Vec::new();
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                let parts: Vec<String> = args.iter().map(|v| self.value_to_string(v)).collect();
                hudhud_print::print_ops::eprintln_str(&parts.join(" "));
                self.registers[255] = Value16::null();
                Ok(true)
            }

            "toml" => {
                self.check_arg_count("config()", 1, arg_count)?;
                let key = self.registers[first_arg as usize];
                let key_str = self.value_to_string(&key);
                let result = self.toml_config_lookup(&key_str);
                self.registers[255] = result;
                Ok(true)
            }

            n if crate::vm::builtin_aliases::is_put_alias(n) => {
                let mut parts = Vec::new();
                for _i in 0..arg_count {
                    parts.push(
                        self.value_to_string(&self.registers[first_arg as usize + _i as usize]),
                    );
                }
                hudhud_print::print_ops::print_str(&parts.join(" "));
                self.registers[255] = Value16::null();
                Ok(true)
            }

            n if crate::vm::builtin_aliases::is_putf_alias(n) => {
                use hudhud_print::format::{sprintf, FmtArg};
                if arg_count < 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "putf() requires at least 1 argument"
                    )));
                }
                let fmt = self.value_to_string(&self.registers[first_arg as usize]);
                let mut args = Vec::new();
                for _i in 1..arg_count {
                    let v = self.registers[first_arg as usize + _i as usize];
                    if let Some(i) = v.as_int() {
                        args.push(FmtArg::Int(i));
                    } else if let Some(n) = v.as_number() {
                        args.push(FmtArg::Float(n));
                    } else {
                        args.push(FmtArg::Str(self.value_to_string(&v)));
                    }
                }
                match sprintf(&fmt, &args) {
                    Ok(s) => hudhud_print::print_ops::print_str(&s),
                    Err(e) => return Err(compile_codes::runtime_error(e)),
                }
                self.registers[255] = Value16::null();
                Ok(true)
            }

            "len" | "uzunluk" => {
                self.check_arg_count("len()", 1, arg_count)?;
                let val = self.registers[first_arg as usize];
                let result = hudhud_typeops::types_ops::shared_len(&[val])?;
                self.registers[255] = result;

                Ok(true)
            }

            "type" | "typeof" => {
                self.check_arg_count("typeof()", 1, arg_count)?;
                let val = self.registers[first_arg as usize];
                let result = hudhud_typeops::types_ops::shared_type_of(&[val])?;
                self.registers[255] = result;

                Ok(true)
            }

            "is" => {
                self.check_arg_count("is()", 2, arg_count)?;
                let val = self.registers[first_arg as usize];
                let type_query = self.registers[first_arg as usize + 1];
                let type_str = match type_query.as_string() {
                    Some(s) => s.to_string(),
                    None => {
                        return Err(compile_codes::runtime_error(
                            "is() second argument must be a string".to_string(),
                        ))
                    }
                };
                let val_v = val;
                let result = match type_str.as_str() {
                    "String" | "string" => val_v.as_string().is_some(),
                    "Number" | "number" => val_v.as_number().is_some(),
                    "Boolean" | "boolean" | "bool" => val_v.as_bool().is_some(),
                    "Array" | "array" => val_v.as_array().is_some(),
                    "Object" | "object" => val_v.as_object().is_some(),
                    "Function" | "function" => val_v.as_function_data().is_some(),
                    "Null" | "null" => val_v.is_null(),
                    "Set" | "set" => val_v.as_set().is_some(),
                    "Map" | "map" => val_v.as_map_pairs().is_some(),
                    "Promise" | "promise" => val_v.as_promise_state().is_some(),
                    "Option" | "option" => val_v.as_option().is_some(),
                    "Result" | "result" => val_v.as_result().is_some(),
                    "Generator" | "generator" => val_v.as_generator_state().is_some(),
                    class_name => {
                        if let Some(inst) = val_v.as_instance_data() {
                            inst.class_name == class_name
                        } else {
                            false
                        }
                    }
                };
                self.registers[255] = Value16::bool_(result);
                Ok(true)
            }

            "toString" | "metneÇevir" => {
                self.check_arg_count("toString()", 1, arg_count)?;
                let val = self.registers[first_arg as usize];
                let result = hudhud_typeops::types_ops::shared_to_string(&[val])?;
                self.registers[255] = result;

                Ok(true)
            }

            "toNumber" | "sayıyaÇevir" => {
                self.check_arg_count("toNumber()", 1, arg_count)?;
                let val = self.registers[first_arg as usize];
                let result = hudhud_typeops::types_ops::shared_to_number(&[val])?;
                self.registers[255] = result;

                Ok(true)
            }

            "toBoolean" | "mantıksalaÇevir" | "booleÇevir" => {
                self.check_arg_count("toBoolean()", 1, arg_count)?;
                let val = self.registers[first_arg as usize];
                let result = hudhud_typeops::types_ops::shared_to_boolean(&[val])?;
                self.registers[255] = result;

                Ok(true)
            }

            // Math
            "sqrt" => {
                self.check_arg_count("sqrt()", 1, arg_count)?;
                let n = self.pop_number(first_arg)?;
                self.registers[255] = Value16::number(n.sqrt());
                Ok(true)
            }
            "abs" => {
                self.check_arg_count("abs()", 1, arg_count)?;
                let n = self.pop_number(first_arg)?;
                self.registers[255] = Value16::number(n.abs());
                Ok(true)
            }
            "floor" => {
                self.check_arg_count("floor()", 1, arg_count)?;
                let n = self.pop_number(first_arg)?;
                let floored = n.floor();
                self.registers[255] = floor_to_int_or_number(floored);
                Ok(true)
            }
            "ceil" => {
                self.check_arg_count("ceil()", 1, arg_count)?;
                let n = self.pop_number(first_arg)?;
                let ceiled = n.ceil();
                self.registers[255] = floor_to_int_or_number(ceiled);
                Ok(true)
            }
            "round" => {
                self.check_arg_count("round()", 1, arg_count)?;
                let n = self.pop_number(first_arg)?;
                let rounded = n.round();
                self.registers[255] = floor_to_int_or_number(rounded);
                Ok(true)
            }
            "idiv" => {
                self.check_arg_count("Math.idiv()", 2, arg_count)?;
                let a = self.registers[first_arg as usize]
                    .as_number_fast()
                    .map(|n| n as i64)
                    .ok_or_else(|| compile_codes::runtime_error("Math.idiv(): arg1 not numeric"))?;
                let b = self.registers[(first_arg + 1) as usize]
                    .as_number_fast()
                    .map(|n| n as i64)
                    .ok_or_else(|| compile_codes::runtime_error("Math.idiv(): arg2 not numeric"))?;
                if b == 0 {
                    return Err(compile_codes::runtime_error(
                        "Math.idiv(): division by zero",
                    ));
                }
                self.registers[255] = Value16::int(a / b);
                Ok(true)
            }
            "min" => {
                self.check_arg_count("min()", 2, arg_count)?;
                let a = self.pop_number(first_arg)?;
                let b = self.pop_number(first_arg + 1)?;
                self.registers[255] = Value16::number(a.min(b));
                Ok(true)
            }
            "max" => {
                self.check_arg_count("max()", 2, arg_count)?;
                let a = self.pop_number(first_arg)?;
                let b = self.pop_number(first_arg + 1)?;
                self.registers[255] = Value16::number(a.max(b));
                Ok(true)
            }

            // Array/Object helpers
            "push" => {
                self.check_arg_count("push()", 2, arg_count)?;
                let mut arr_val = self.registers[first_arg as usize];
                let elem = self.registers[first_arg as usize + 1];
                if let Some(arr) = arr_val.as_array_mut() {
                    arr.push(elem);
                    let len = arr.len() as i64;
                    self.registers[255] = Value16::int(len);
                } else {
                    return Err(compile_codes::runtime_error(
                        "push() requires an array".to_string(),
                    ));
                }
                Ok(true)
            }
            "pop" => {
                self.check_arg_count("pop()", 1, arg_count)?;
                let arr_val = self.registers[first_arg as usize];
                let arr_v = arr_val;
                if let Some(arr) = arr_v.as_array() {
                    let mut new_arr = arr.clone();
                    let popped = new_arr.pop().unwrap_or(Value16::null());
                    self.registers[255] = popped;
                } else {
                    return Err(compile_codes::runtime_error(
                        "pop() requires an array".to_string(),
                    ));
                }
                Ok(true)
            }
            "keys" => {
                self.check_arg_count("keys()", 1, arg_count)?;
                let val = self.registers[first_arg as usize];
                let result = hudhud_typeops::types_ops::shared_keys(&[val])?;
                self.registers[255] = result;

                Ok(true)
            }
            "values" => {
                self.check_arg_count("values()", 1, arg_count)?;
                let val = self.registers[first_arg as usize];
                let result = hudhud_typeops::types_ops::shared_values(&[val])?;
                self.registers[255] = result;

                Ok(true)
            }

            // String helpers
            "concat" => {
                let mut args: Vec<Value16> = Vec::new();
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                let result: String = args.iter().map(|v| self.value_to_string(v)).collect();
                self.registers[255] = Value16::string(result);

                Ok(true)
            }
            // T1-8: Native strrev builtin — single alloc, O(n) reverse.
            "strrev" => {
                self.check_arg_count("strrev()", 1, arg_count)?;
                let val = self.registers[first_arg as usize];
                let s = match val.as_str() {
                    Some(s) => s,
                    None => {
                        return Err(compile_codes::runtime_error(
                            "strrev() requires a string".to_string(),
                        ))
                    }
                };
                let result = crate::vm::string::call_string_method(s, "reverse", &[], false)?;
                self.registers[255] = result;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
