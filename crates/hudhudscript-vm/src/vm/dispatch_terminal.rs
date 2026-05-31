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
    pub(crate) fn dispatch_builtin_group7(
        &mut self,
        name: &str,
        arg_count: u8,
        first_arg: u8,
        _bytecode: &hudhudscript_bytecode::Bytecode,
    ) -> hudhudscript_bytecode::error::CompileResult<bool> {
        match name {
            "confirm" => {
                let mut args = Vec::new();
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                let prompt = match args.first().and_then(|v| v.as_string()) {
                    Some(s) => s.to_string(),
                    _ => "Confirm?".to_string(),
                };
                print!("{} [y/N] ", prompt);
                use std::io::Write;
                // Flush is best-effort — failure doesn't affect program correctness
                let _ = std::io::stdout().flush();
                let mut line = String::new();
                std::io::stdin()
                    .read_line(&mut line)
                    .map_err(|e| compile_codes::runtime_error(format!("confirm error: {}", e)))?;
                let answer = line.trim().to_lowercase();
                self.registers[255] = Value16::bool_(matches!(
                    answer.as_str(),
                    "y" | "yes" | "evet" | "e"
                ));
                Ok(true)
            }

            // Timer builtins (v0.4.38 — #618, Kural 7 parity).
            //
            // Both the VM and the interpreter now route through
            // `hudhudscript_builtins::timer_ops`. The VM used to
            // silently tolerate missing / negative ms values, skip id
            // generation, and ignore the callback argument entirely;
            // the shared helper validates all three, allocates a
            // process-wide unique timer id, and emits a descriptor
            // including a `callback_pending` field so scripts can tell
            // the builtin has accepted the callback even though the
            // synchronous path can't yet dispatch it. Real callback
            // invocation remains a pending feature of the async runtime
            // in both runtimes — at least now the gap is symmetric and
            // explicit, not a silent fake.
            "setTimeout" => {
                let mut args = Vec::with_capacity(arg_count as usize);
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                let result = hudhud_timer::timer_ops::shared_set_timeout(&args)?;
                self.registers[255] = result;

                Ok(true)
            }
            "setInterval" => {
                let mut args = Vec::with_capacity(arg_count as usize);
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                let result = hudhud_timer::timer_ops::shared_set_interval(&args)?;
                self.registers[255] = result;

                Ok(true)
            }
            "clearTimeout" => {
                let mut args = Vec::with_capacity(arg_count as usize);
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                let result = hudhud_timer::timer_ops::shared_clear_timer(&args, "clearTimeout")?;
                self.registers[255] = result;

                Ok(true)
            }
            "clearInterval" => {
                let mut args = Vec::with_capacity(arg_count as usize);
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                args.reverse();
                let result = hudhud_timer::timer_ops::shared_clear_timer(&args, "clearInterval")?;
                self.registers[255] = result;

                Ok(true)
            }

            // style function (v0.4.38 — #657)
            "style" => {
                let mut args = Vec::new();
                for _i in 0..arg_count {
                    args.push(self.registers[first_arg as usize + _i as usize]);
                }
                // args.reverse() removed — Bug E fix
                let text = match args.first().and_then(|v| v.as_string()) {
                    Some(s) => s.to_string(),
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "style() requires a string argument".to_string(),
                        ))
                    }
                };
                let opts = match args.get(1).and_then(|v| v.as_object()) {
                    Some(obj) => obj.clone(),
                    _ => {
                        self.registers[255] = Value16::string(text);

                        return Ok(true);
                    }
                };
                let mut codes = Vec::new();
                if let Some(color) = opts.get("color").and_then(|v| v.as_string()) {
                    match color.as_str() {
                        "red" => codes.push("31"),
                        "green" => codes.push("32"),
                        "yellow" => codes.push("33"),
                        "blue" => codes.push("34"),
                        "magenta" => codes.push("35"),
                        "cyan" => codes.push("36"),
                        "white" => codes.push("37"),
                        "gray" | "grey" => codes.push("90"),
                        _ => {}
                    }
                }
                if let Some(bg) = opts.get("bg").and_then(|v| v.as_string()) {
                    match bg.as_str() {
                        "red" => codes.push("41"),
                        "green" => codes.push("42"),
                        "yellow" => codes.push("43"),
                        "blue" => codes.push("44"),
                        "magenta" => codes.push("45"),
                        "cyan" => codes.push("46"),
                        "white" => codes.push("47"),
                        _ => {}
                    }
                }
                if opts
                    .get("bold")
                    .map_or(false, |v| v.as_bool() == Some(true))
                {
                    codes.push("1");
                }
                if opts.get("dim").map_or(false, |v| v.as_bool() == Some(true)) {
                    codes.push("2");
                }
                if opts
                    .get("italic")
                    .map_or(false, |v| v.as_bool() == Some(true))
                {
                    codes.push("3");
                }
                if opts
                    .get("underline")
                    .map_or(false, |v| v.as_bool() == Some(true))
                {
                    codes.push("4");
                }
                if opts
                    .get("strikethrough")
                    .map_or(false, |v| v.as_bool() == Some(true))
                {
                    codes.push("9");
                }
                if codes.is_empty() {
                    self.registers[255] = Value16::string(text);

                } else {
                    let result = format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text);
                    self.registers[255] = Value16::string(result);

                }
                Ok(true)
            }

            _ => Ok(false),
        }
    }
}
