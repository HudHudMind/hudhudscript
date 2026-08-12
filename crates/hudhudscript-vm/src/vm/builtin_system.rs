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
    pub(crate) fn call_stdin_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_term::stdin_ops::StdinMethodId>()?;
            hudhud_term::stdin_ops::dispatch(id, &args)
        }
    }

    // ── Terminal methods (v0.4.38 — #657) ───────────────────────────

    pub(crate) fn call_terminal_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_term::terminal_ops::TerminalMethodId>()?;
            hudhud_term::terminal_ops::dispatch(id, &args)
        }
    }

    // ── log methods (v0.4.38 — #662) ────────────────────────────────

    pub(crate) fn call_log_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_term::log_ops::LogMethodId>()?;
            hudhud_term::log_ops::dispatch(id, &args)
        }
    }

    // ── exec methods (v0.4.38 — #674) ──────────────────────────────

    pub(crate) fn call_exec_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        // HOST-6: enforce host_access exec policy.
        self.host_access_policy.ensure_module_allowed("exec")?;
        self.host_access_policy.ensure_exec_method(method)?;

        // Methods that spawn a child process must also pass command-level checks.
        const COMMAND_METHODS: &[&str] = &["run", "output", "stream", "lines", "spawn", "timeout"];
        if COMMAND_METHODS.contains(&method) {
            match hudhud_exec::exec_ops::utils::parse_cmd(&args) {
                Ok((command, _)) => {
                    self.host_access_policy.ensure_command_allowed(&command)?;
                }
                Err(e) => {
                    // Only fail parsing if the command is actually needed.
                    // Some methods may be called with no args for inspection.
                    return Err(compile_codes::runtime_error(e.to_string()));
                }
            }
        }

        hudhud_exec::exec_ops::dispatch(method, &args)
    }

    // ── TCP module (v0.4.38 — #675) ─────────────────────────────────
    pub(crate) fn call_tcp_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        hudhud_net::tcp_ops::dispatch(method, &args)
    }

    // ── UDP module (v0.4.38 — #675) ─────────────────────────────────
    pub(crate) fn call_udp_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        hudhud_net::udp_ops::dispatch(method, &args)
    }

    // ── Unix domain socket module (v0.4.38 — #676) ──────────────────
    #[cfg(not(unix))]
    pub(crate) fn call_unix_method(
        &self,
        _method: &str,
        _args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        Err(compile_codes::runtime_error(
            "Unix sockets not supported on this platform".to_string(),
        ))
    }

    #[cfg(unix)]
    pub(crate) fn call_unix_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        use std::io::{Read, Write};
        use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
        use std::os::unix::net::UnixStream;

        let extract_fd = |args: &[Value16]| -> CompileResult<RawFd> {
            match args.first() {
                Some(obj_val) => match obj_val.as_object().and_then(|obj| obj.get("fd")) {
                    Some(fd_val) => match fd_val.as_number() {
                        Some(n) => Ok(n as RawFd),
                        _ => Err(compile_codes::runtime_error("unix: missing fd".to_string())),
                    },
                    _ => Err(compile_codes::runtime_error("unix: missing fd".to_string())),
                },
                _ => Err(compile_codes::runtime_error(
                    "unix: expected connection object".to_string(),
                )),
            }
        };

        match method {
            "connect" => {
                let path = match args.first().and_then(|v| v.as_string()) {
                    Some(s) => s.to_string(),
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "unix.connect: expected path string".to_string(),
                        ))
                    }
                };
                let stream = UnixStream::connect(&path).map_err(|e| {
                    compile_codes::runtime_error(format!("unix.connect error: {}", e))
                })?;
                // Timeout failure is non-fatal — reads will just block indefinitely
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(30)));
                let fd = stream.as_raw_fd();
                // Prevent dropping the stream so the fd stays open —
                // the caller manages the fd lifetime via the returned object.
                let _ = std::mem::ManuallyDrop::new(stream);
                let mut obj = hudhudscript_bytecode::ObjMap::default();
                obj.insert(
                    "__type".to_string(),
                    Value16::string("UnixStream".to_string()),
                );
                obj.insert("fd".to_string(), Value16::number(fd as f64));
                obj.insert("path".to_string(), Value16::string(path));
                Ok(Value16::object(obj))
            }
            "write" => {
                let fd = extract_fd(&args)?;
                let data = match args.get(1).and_then(|v| v.as_string()) {
                    Some(s) => s.as_bytes().to_vec(),
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "unix.write: expected data string".to_string(),
                        ))
                    }
                };
                // SAFETY: fd was obtained from a valid UnixStream via as_raw_fd().
                // ManuallyDrop ensures the fd is not closed when the reconstructed
                // stream goes out of scope — the original owner manages the fd lifetime.
                let stream = unsafe { UnixStream::from_raw_fd(fd) };
                let mut stream = std::mem::ManuallyDrop::new(stream);
                let result = stream.write_all(&data);
                result.map_err(|e| {
                    compile_codes::runtime_error(format!("unix.write error: {}", e))
                })?;
                Ok(Value16::number(data.len() as f64))
            }
            "read" => {
                let fd = extract_fd(&args)?;
                let buf_size = match args.get(1).and_then(|v| v.as_number()) {
                    Some(n) => n as usize,
                    _ => 4096,
                };
                // SAFETY: fd was obtained from a valid UnixStream via as_raw_fd().
                // ManuallyDrop ensures the fd is not closed when the reconstructed
                // stream goes out of scope — the original owner manages the fd lifetime.
                let stream = unsafe { UnixStream::from_raw_fd(fd) };
                let mut stream = std::mem::ManuallyDrop::new(stream);
                let mut buf = vec![0u8; buf_size];
                let result = stream.read(&mut buf);
                let n = result
                    .map_err(|e| compile_codes::runtime_error(format!("unix.read error: {}", e)))?;
                Ok(Value16::string(
                    String::from_utf8_lossy(&buf[..n]).to_string(),
                ))
            }
            "close" => {
                let fd = extract_fd(&args)?;
                // SAFETY: fd was obtained from a valid UnixStream via as_raw_fd().
                // We intentionally reconstruct and drop to close the fd.
                unsafe {
                    drop(UnixStream::from_raw_fd(fd));
                }
                Ok(Value16::null())
            }
            "http" => {
                let path = match args.first().and_then(|v| v.as_string()) {
                    Some(s) => s.to_string(),
                    _ => {
                        return Err(compile_codes::runtime_error(
                            "unix.http: expected socket path".to_string(),
                        ))
                    }
                };
                let http_method = match args.get(1).and_then(|v| v.as_string()) {
                    Some(s) => s.to_uppercase(),
                    _ => "GET".to_string(),
                };
                let uri_path = match args.get(2).and_then(|v| v.as_string()) {
                    Some(s) => s.to_string(),
                    _ => "/".to_string(),
                };
                let body = match args.get(3) {
                    Some(v) => match v.as_string() {
                        Some(s) => Some(s.to_string()),
                        None => match v.as_object() {
                            Some(obj) => Some(
                                serde_json::to_string(obj).unwrap_or_else(|_| "{}".to_string()),
                            ),
                            None => None,
                        },
                    },
                    _ => None,
                };

                let mut stream = UnixStream::connect(&path).map_err(|e| {
                    compile_codes::runtime_error(format!("unix.http connect error: {}", e))
                })?;
                // Timeout failure is non-fatal — reads will just block indefinitely
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(30)));

                let content = body.as_deref().unwrap_or("");
                let request = if content.is_empty() {
                    format!(
                        "{} {} HTTP/1.0\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                        http_method, uri_path
                    )
                } else {
                    format!(
                        "{} {} HTTP/1.0\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        http_method, uri_path, content.len(), content
                    )
                };

                stream.write_all(request.as_bytes()).map_err(|e| {
                    compile_codes::runtime_error(format!("unix.http write error: {}", e))
                })?;
                // Shutdown is best-effort — the read side will still see EOF
                let _ = stream.shutdown(std::net::Shutdown::Write);

                let mut response = String::new();
                stream.read_to_string(&mut response).map_err(|e| {
                    compile_codes::runtime_error(format!("unix.http read error: {}", e))
                })?;

                let mut result = hudhudscript_bytecode::ObjMap::default();
                if let Some(header_end) = response.find("\r\n\r\n") {
                    let header_part = &response[..header_end];
                    let body_part = &response[header_end + 4..];

                    if let Some(first_line) = header_part.lines().next() {
                        let parts: Vec<&str> = first_line.splitn(3, ' ').collect();
                        if parts.len() >= 2 {
                            let status: f64 = parts[1].parse().unwrap_or(0.0);
                            result.insert("status".to_string(), Value16::number(status));
                            result.insert(
                                "ok".to_string(),
                                Value16::bool_((200.0..300.0).contains(&status)),
                            );
                        }
                    }

                    let mut headers = hudhudscript_bytecode::ObjMap::default();
                    for line in header_part.lines().skip(1) {
                        if let Some((k, v)) = line.split_once(": ") {
                            headers.insert(
                                k.to_lowercase().to_string(),
                                Value16::string(v.to_string()),
                            );
                        }
                    }
                    result.insert("headers".to_string(), Value16::object(headers));
                    result.insert("body".to_string(), Value16::string(body_part.to_string()));

                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(body_part) {
                        result.insert(
                            "json".to_string(),
                            hudhud_http::json::serde_to_value(&json_val),
                        );
                    } else {
                        result.insert("json".to_string(), Value16::null());
                    }
                } else {
                    result.insert("status".to_string(), Value16::number(0.0));
                    result.insert("ok".to_string(), Value16::bool_(false));
                    result.insert("body".to_string(), Value16::string(response));
                    result.insert(
                        "headers".to_string(),
                        Value16::object(hudhudscript_bytecode::ObjMap::default()),
                    );
                    result.insert("json".to_string(), Value16::null());
                }

                Ok(Value16::object(result))
            }
            _ => Err(compile_codes::runtime_error(format!(
                "Unknown unix method: {}",
                method
            ))),
        }
    }

    // ── WebSocket module (v0.4.38 — #616) ───────────────────────────
    pub(crate) fn call_ws_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        hudhud_net::ws_ops::dispatch(method, &args)
    }

    // ── Daemon module (v0.4.38 — #596) ─────────────────────────────────

    pub(crate) fn call_daemon_method(
        &mut self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        hudhud_exec::daemon_ops::dispatch(method, &args)
    }

    // ── Filesystem operations (v0.4.38 — #604) ──────────────────────────

    pub(crate) fn call_fs_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        hudhud_fs::fs_builtins::dispatch(method, &args)
    }

    // ── tokenomics methods (T3 — #TOK-3) ───────────────────────────

    pub(crate) fn call_tokenomics_method(
        &self,
        method: &str,
        _args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        match method {
            "session_cost" => Ok(Value16::number(0.0)),
            "usage" => {
                let mut m = hudhudscript_bytecode::ObjMap::default();
                m.insert("daily".to_string(), Value16::int(0));
                m.insert("monthly".to_string(), Value16::int(0));
                Ok(Value16::object(m))
            }
            "budget_health" => Ok(Value16::number(1.0)),
            _ => Err(compile_codes::runtime_error(format!(
                "Unknown tokenomics method: {}",
                method
            ))),
        }
    }

    // ── channel methods (CH2 — #CH-2) ────────────────────────────

    pub(crate) fn call_channel_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        match method {
            "send" => {
                if args.len() < 2 {
                    return Err(compile_codes::runtime_error(
                        "channel.send() requires at least 2 arguments: channel_name, text"
                            .to_string(),
                    ));
                }
                let _channel_name = args[0].as_string().ok_or_else(|| {
                    compile_codes::runtime_error(
                        "channel.send() first argument must be a string (channel name)".to_string(),
                    )
                })?;
                let _text = args[1].as_string().ok_or_else(|| {
                    compile_codes::runtime_error(
                        "channel.send() second argument must be a string (message text)"
                            .to_string(),
                    )
                })?;
                // CH4: wire to ChannelRegistry
                Ok(Value16::null())
            }
            "notify" => {
                if args.is_empty() {
                    return Err(compile_codes::runtime_error(
                        "channel.notify() requires at least 1 argument: text".to_string(),
                    ));
                }
                let _text = args[0].as_string().ok_or_else(|| {
                    compile_codes::runtime_error(
                        "channel.notify() first argument must be a string (message text)"
                            .to_string(),
                    )
                })?;
                Ok(Value16::null())
            }
            _ => Err(compile_codes::runtime_error(format!(
                "Unknown channel method: {}",
                method
            ))),
        }
    }
}
