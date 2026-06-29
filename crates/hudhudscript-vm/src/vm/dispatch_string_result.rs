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

impl crate::vm::VM {
    pub(crate) fn dispatch_builtin_group2(
        &mut self,
        name: &str,
        arg_count: u8,
        first_arg: u8,
        _bytecode: &hudhudscript_bytecode::Bytecode,
    ) -> hudhudscript_bytecode::error::CompileResult<bool> {
        match name {
            "indexOf" => {
                self.check_arg_count("indexOf()", 2, arg_count)?;
                let haystack = self.registers[first_arg as usize];
                let needle = self.registers[first_arg as usize + 1];
                let haystack_v = haystack;
                let needle_v = needle;
                if let (Some(s), Some(n)) = (haystack_v.as_string(), needle_v.as_string()) {
                    let idx = s.find(n.as_str()).map(|i| i as f64).unwrap_or(-1.0);
                    self.registers[255] = Value16::number(idx);
                } else if let Some(arr) = haystack_v.as_array() {
                    let idx = arr
                        .iter()
                        .position(|v| self.values_equal(v, &needle))
                        .map(|i| i as f64)
                        .unwrap_or(-1.0);
                    self.registers[255] = Value16::number(idx);
                } else {
                    return Err(compile_codes::runtime_error(
                        "indexOf() requires string or array".to_string(),
                    ));
                }
                Ok(true)
            }
            "contains" => {
                self.check_arg_count("contains()", 2, arg_count)?;
                let haystack = self.registers[first_arg as usize];
                let needle = self.registers[first_arg as usize + 1];
                let haystack_v = haystack;
                let needle_v = needle;
                let result =
                    if let (Some(s), Some(n)) = (haystack_v.as_string(), needle_v.as_string()) {
                        s.contains(n.as_str())
                    } else if let Some(arr) = haystack_v.as_array() {
                        arr.iter().any(|v| self.values_equal(v, &needle))
                    } else {
                        false
                    };
                self.registers[255] = Value16::bool_(result);
                Ok(true)
            }
            "split" => {
                self.check_arg_count("split()", 2, arg_count)?;
                let val = self.registers[first_arg as usize];
                let delimiter = self.registers[first_arg as usize + 1];
                let val_v = val;
                let delimiter_v = delimiter;
                if let (Some(s), Some(d)) = (val_v.as_string(), delimiter_v.as_string()) {
                    let parts: Vec<Value16> = s
                        .split(&d)
                        .map(|p| Value16::string(p.to_string()))
                        .collect();
                    self.registers[255] = Value16::array(parts);
                } else {
                    return Err(compile_codes::runtime_error(
                        "split() requires string arguments".to_string(),
                    ));
                }
                Ok(true)
            }
            "join" => {
                if arg_count != 2 {
                    return Err(compile_codes::runtime_error(format!(
                        "join() expects 2 arguments, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let delimiter = self.registers[first_arg as usize + 1];
                let val_v = val;
                let delimiter_v = delimiter;
                if let (Some(arr), Some(d)) = (val_v.as_array(), delimiter_v.as_string()) {
                    let parts: Vec<String> = arr
                        .iter()
                        .map(|v| self.value_to_string(&v.clone()))
                        .collect();
                    self.registers[255] = Value16::string(parts.join(&d));
                } else {
                    return Err(compile_codes::runtime_error(
                        "join() requires array and string".to_string(),
                    ));
                }
                Ok(true)
            }
            "replace" => {
                if arg_count != 3 {
                    return Err(compile_codes::runtime_error(format!(
                        "replace() expects 3 arguments, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let pattern = self.registers[first_arg as usize + 1];
                let replacement = self.registers[first_arg as usize + 2];
                let val_v = val;
                let pattern_v = pattern;
                let replacement_v = replacement;
                if let (Some(s), Some(p), Some(r)) = (
                    val_v.as_string(),
                    pattern_v.as_string(),
                    replacement_v.as_string(),
                ) {
                    self.registers[255] = Value16::string(s.replace(&p, &r));
                } else {
                    return Err(compile_codes::runtime_error(
                        "replace() requires string arguments".to_string(),
                    ));
                }
                Ok(true)
            }
            "trim" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "trim() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                if let Some(s) = val.as_string() {
                    self.registers[255] = Value16::string(s.trim().to_string());
                } else {
                    return Err(compile_codes::runtime_error(
                        "trim() requires a string".to_string(),
                    ));
                }
                Ok(true)
            }
            "toUpperCase" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "toUpperCase() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                if let Some(s) = val.as_string() {
                    self.registers[255] = Value16::string(s.to_uppercase());
                } else {
                    return Err(compile_codes::runtime_error(
                        "toUpperCase() requires a string".to_string(),
                    ));
                }
                Ok(true)
            }
            "toLowerCase" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "toLowerCase() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                if let Some(s) = val.as_string() {
                    self.registers[255] = Value16::string(s.to_lowercase());
                } else {
                    return Err(compile_codes::runtime_error(
                        "toLowerCase() requires a string".to_string(),
                    ));
                }
                Ok(true)
            }

            "startsWith" => {
                if arg_count != 2 {
                    return Err(compile_codes::runtime_error(format!(
                        "startsWith() expects 2 arguments, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let prefix = self.registers[first_arg as usize + 1];
                if let (Some(s), Some(p)) = (val.as_string(), prefix.as_string()) {
                    self.registers[255] = Value16::bool_(s.starts_with(p.as_str()));
                } else {
                    return Err(compile_codes::runtime_error(
                        "startsWith() requires string arguments".to_string(),
                    ));
                }
                Ok(true)
            }

            "endsWith" => {
                if arg_count != 2 {
                    return Err(compile_codes::runtime_error(format!(
                        "endsWith() expects 2 arguments, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let suffix = self.registers[first_arg as usize + 1];
                if let (Some(s), Some(suf)) = (val.as_string(), suffix.as_string()) {
                    self.registers[255] = Value16::bool_(s.ends_with(suf.as_str()));
                } else {
                    return Err(compile_codes::runtime_error(
                        "endsWith() requires string arguments".to_string(),
                    ));
                }
                Ok(true)
            }

            "substring" => {
                if !(2..=3).contains(&arg_count) {
                    return Err(compile_codes::runtime_error(format!(
                        "substring() expects 2-3 arguments, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let start = self.pop_number(first_arg + 1)? as usize;
                let end = if arg_count == 3 {
                    Some(self.pop_number(first_arg + 2)? as usize)
                } else {
                    None
                };
                if let Some(s) = val.as_string() {
                    let end = end.unwrap_or(s.len());
                    let result: String = s
                        .chars()
                        .skip(start)
                        .take(end.saturating_sub(start))
                        .collect();
                    self.registers[255] = Value16::string(result);
                } else {
                    return Err(compile_codes::runtime_error(
                        "substring() requires a string".to_string(),
                    ));
                }
                Ok(true)
            }
            "length" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "length() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let val_v = val;
                let length = if let Some(s) = val_v.as_string() {
                    s.chars().count() as i64
                } else if let Some(arr) = val_v.as_array() {
                    arr.len() as i64
                } else if let Some(obj) = val_v.as_object() {
                    obj.len() as i64
                } else {
                    return Err(compile_codes::runtime_error(format!(
                        "length() not supported for type {}",
                        self.type_name_of(&val)
                    )));
                };
                self.registers[255] = Value16::int(length);
                Ok(true)
            }
            // Option/Result constructors
            "Some" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "Some() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let val_v = val;
                self.registers[255] = Value16::option(Some(val_v.clone()));

                Ok(true)
            }
            "Ok" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "Ok() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let val_v = val;
                self.registers[255] = Value16::result(Ok(val_v.clone()));

                Ok(true)
            }
            "Err" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "Err() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let msg = self.value_to_string(&val);
                self.registers[255] = Value16::result(Err(msg.clone()));

                Ok(true)
            }
            "unwrap" => {
                if arg_count != 1 {
                    return Err(compile_codes::runtime_error(format!(
                        "unwrap() expects 1 argument, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let val_v = val;
                if let Some(opt) = val_v.as_option() {
                    match opt {
                        Some(v) => self.registers[255] = *v,
                        None => {
                            return Err(compile_codes::runtime_error(
                                "called unwrap() on None".to_string(),
                            ))
                        }
                    }
                } else if let Some(res) = val_v.as_result() {
                    match res {
                        Ok(v) => self.registers[255] = *v,
                        Err(e) => {
                            return Err(compile_codes::runtime_error(format!(
                                "called unwrap() on Err({})",
                                e
                            )))
                        }
                    }
                } else if val_v.is_null() {
                    return Err(compile_codes::runtime_error(
                        "called unwrap() on None".to_string(),
                    ));
                } else {
                    return Err(compile_codes::runtime_error(
                        "unwrap() requires Option or Result".to_string(),
                    ));
                }
                Ok(true)
            }
            "unwrap_or" => {
                if arg_count != 2 {
                    return Err(compile_codes::runtime_error(format!(
                        "unwrap_or() expects 2 arguments, got {}",
                        arg_count
                    )));
                }
                let val = self.registers[first_arg as usize];
                let default = self.registers[first_arg as usize + 1];
                let val_v = val;
                if let Some(opt) = val_v.as_option() {
                    match opt {
                        Some(v) => self.registers[255] = *v,
                        None => self.registers[255] = default,
                    }
                } else if let Some(res) = val_v.as_result() {
                    match res {
                        Ok(v) => self.registers[255] = *v,
                        Err(_) => self.registers[255] = default,
                    }
                } else if val_v.is_null() {
                    self.registers[255] = default;
                } else {
                    return Err(compile_codes::runtime_error(
                        "unwrap_or() requires Option or Result".to_string(),
                    ));
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
