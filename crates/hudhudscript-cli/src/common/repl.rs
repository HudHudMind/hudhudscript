use crate::common::{register_vm_stdlib_modules, setup_provider_registry, CliError, HudHudConfig};
use hudhudscript_compiler::{Bytecode, Compiler};
use hudhudscript_deploy_core::adapters::{create_adapter, Adapter};
use hudhudscript_formatter::Formatter;
use hudhudscript_mcp::{McpClient, TransportConfig};
use hudhudscript_parser::{parse, parse_with_recovery};
use hudhudscript_vm::{OutputLocale, VM};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn run_repl(debug: bool, load_file: Option<PathBuf>) -> Result<(), CliError> {
    use crate::repl::HudCompleter;
    use rustyline::error::ReadlineError;
    use rustyline::Editor;
    // In-memory history mirror for :history command (rustyline's history is opaque)
    let mut repl_history: Vec<String> = Vec::new();

    println!("HudHudScript REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type 'exit' or press Ctrl+D to quit");
    println!("Type 'help' for available commands");
    println!();

    let mut rl = Editor::new()
        .map_err(|e| CliError::Runtime(format!("Failed to initialize REPL: {}", e)))?;
    rl.set_helper(Some(HudCompleter::new()));

    // Load history if it exists
    let history_path = dirs::home_dir().map(|mut p| {
        p.push(".hudhudscript_history");
        p
    });

    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    // Provider registry is still needed for `ai.*` / `council.*` builtins;
    // setup kept for future wiring (VM pulls providers from shared crates,
    // so we don't need to stash the registry on the VM the way the
    // interpreter did).
    let _provider_registry = setup_provider_registry(debug, None).map_err(CliError::Runtime)?;

    // Keep a Tokio runtime alive for the whole REPL so async providers
    // (reqwest, tcp, ws) have a context when VM-side builtins call them.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CliError::Runtime(format!("Failed to create runtime: {}", e)))?;
    let _rt_guard = rt.enter();

    // Faz 4 REPL migration: one persistent VM; per-line we parse + compile
    // and `execute` on the same VM so `scopes[0]` accumulates bindings,
    // class metadata, actors, etc. across lines.
    let mut vm = hudhudscript_vm::VM::new();
    register_vm_stdlib_modules(&mut vm);

    let mut line_buffer = String::new();
    let mut in_multiline = false;

    // Load file if specified — runs through the same VM so the REPL user
    // picks up the loaded globals on line 1.
    if let Some(load_path) = load_file {
        println!("Loading: {}", load_path.display());
        match fs::read_to_string(&load_path) {
            Ok(source) => match parse(&source) {
                Ok(ast) => {
                    let canonical_script =
                        fs::canonicalize(&load_path).unwrap_or(load_path.clone());
                    let module_base = canonical_script
                        .parent()
                        .unwrap_or(Path::new("."))
                        .to_path_buf();
                    let mut compiler = hudhudscript_compiler::Compiler::new();
                    compiler.set_module_base_dir(module_base.clone());
                    match compiler.compile(&ast) {
                        Ok(bytecode) => match vm.execute(&bytecode) {
                            Ok(_) => println!("✓ Loaded successfully"),
                            Err(e) => {
                                let u: hudhudscript_errors::Error = e;
                                eprintln!("Error executing file: {}", u.render_full());
                            }
                        },
                        Err(e) => {
                            let u: hudhudscript_errors::Error = e;
                            eprintln!("Error compiling file: {}", u.render_full());
                        }
                    }
                }
                Err(e) => {
                    let u: hudhudscript_errors::Error = e;
                    eprintln!("Error parsing file: {}", u.render_full());
                }
            },
            Err(e) => eprintln!("Error reading file: {}", e),
        }
        println!();
    }

    loop {
        let prompt = if in_multiline { ".. " } else { ">> " };
        let readline = rl.readline(prompt);

        match readline {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() && !in_multiline {
                    continue;
                }

                // Handle REPL commands
                if !in_multiline && line.starts_with(':') {
                    if handle_repl_command(line, &mut vm, debug, &repl_history) {
                        break; // :quit was issued
                    }
                    continue;
                }

                if !in_multiline && (line == "exit" || line == "quit") {
                    break;
                }

                if !in_multiline && line == "help" {
                    show_repl_help();
                    continue;
                }

                // Add to history (both rustyline and our in-memory mirror)
                let _ = rl.add_history_entry(line);
                if !line.is_empty() {
                    repl_history.push(line.to_string());
                }

                // Check for multiline input: trailing { ( [ or backslash continuation
                if line.ends_with('\\') {
                    // Strip the trailing backslash and continue reading
                    let trimmed = line.trim_end_matches('\\');
                    in_multiline = true;
                    line_buffer.push_str(trimmed);
                    line_buffer.push('\n');
                    continue;
                }

                if line.ends_with('{') || line.ends_with('(') || line.ends_with('[') {
                    in_multiline = true;
                    line_buffer.push_str(line);
                    line_buffer.push('\n');
                    continue;
                }

                if in_multiline {
                    line_buffer.push_str(line);
                    line_buffer.push('\n');

                    // Check if we should exit multiline mode
                    if line.ends_with('}') || line.ends_with(')') || line.ends_with(']') {
                        in_multiline = false;
                        let code = line_buffer.clone();
                        line_buffer.clear();

                        execute_code(&code, &mut vm, debug);
                    }
                    continue;
                }

                // Single line execution
                execute_code(line, &mut vm, debug);
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                if in_multiline {
                    in_multiline = false;
                    line_buffer.clear();
                    println!("Multiline input cancelled");
                } else {
                    break;
                }
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history
    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
    }

    Ok(())
}

pub fn execute_code(code: &str, vm: &mut hudhudscript_vm::VM, debug: bool) {
    // Parse → compile → VM execute. Errors routed through the unified
    // catalog (render_full) so REPL users see the same diagnostics as
    // `hudi run` / `hudi check`.
    let ast = match parse(code) {
        Ok(a) => a,
        Err(e) => {
            let unified: hudhudscript_errors::Error = e;
            eprintln!("{}", unified.render_full());
            return;
        }
    };

    if debug {
        println!("AST: {:#?}", &ast);
    }

    let module_base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut compiler = hudhudscript_compiler::Compiler::new();
    compiler.set_module_base_dir(module_base);
    let bytecode = match compiler.compile(&ast) {
        Ok(b) => b,
        Err(e) => {
            let unified: hudhudscript_errors::Error = e;
            eprintln!("{}", unified.render_full());
            return;
        }
    };

    match vm.execute(&bytecode) {
        Ok(()) => {
            // VM::execute returns () — REPL output comes via print() captured
            // globals, not a last-expression value.
        }
        Err(e) => {
            let unified: hudhudscript_errors::Error = e;
            eprintln!("{}", unified.render_full());
        }
    }
}

/// Display all user-defined variables in the interpreter's environment (Issue #303).
///
/// Filters out built-in native functions so the user sees only their own bindings.
fn show_vars(vm: &hudhudscript_vm::VM) {
    let mut user_vars: Vec<(String, String, String)> = vm
        .all_globals()
        .filter(|(sym, v)| {
            let name = hudhudscript_bytecode::interner::resolve(
                hudhudscript_bytecode::interner::SymbolId(sym.0),
            );
            !name.starts_with("__")
                && !v
                    .as_object()
                    .map(|obj| obj.contains_key("__module"))
                    .unwrap_or(false)
        })
        .map(|(sym, value)| {
            let name = hudhudscript_bytecode::interner::resolve(
                hudhudscript_bytecode::interner::SymbolId(sym.0),
            );
            let type_name = vm_value_type_name(value).to_string();
            let display = format!("{:?}", value);
            let display = if display.len() > 80 {
                format!("{}...", &display[..77])
            } else {
                display
            };
            (name.clone(), type_name, display)
        })
        .collect();

    if user_vars.is_empty() {
        println!("No user-defined variables.");
        return;
    }

    user_vars.sort_by(|a, b| a.0.cmp(&b.0));

    println!("Variables ({}):", user_vars.len());
    for (name, type_name, display) in &user_vars {
        println!("  {} : {} = {}", name, type_name, display);
    }
}

/// Map a VM `Value16` to a short type name for REPL display.
fn vm_value_type_name(v: &hudhudscript_bytecode::Value16) -> &'static str {
    if v.is_null() {
        return "null";
    }
    if v.as_bool().is_some() {
        return "boolean";
    }
    if v.as_int().is_some() || v.as_number().is_some() {
        return "number";
    }
    if v.as_str().is_some() {
        return "string";
    }
    if v.as_array().is_some() {
        return "array";
    }
    if v.as_object().is_some() {
        return "object";
    }
    if v.as_function_data().is_some() {
        return "function";
    }
    if v.as_class_data().is_some() {
        return "class";
    }
    if v.as_instance_data().is_some() {
        return "instance";
    }
    if v.as_data_data().is_some() {
        return "data";
    }
    if v.as_promise_state().is_some() {
        return "promise";
    }
    if v.as_option().is_some() {
        return "option";
    }
    if v.as_result().is_some() {
        return "result";
    }
    if v.as_set().is_some() {
        return "set";
    }
    if v.as_map_pairs().is_some() {
        return "map";
    }
    if v.as_generator_state().is_some() {
        return "generator";
    }
    if v.as_tool_ref().is_some() {
        return "tool";
    }
    if v.as_resource_ref().is_some() {
        return "resource";
    }
    "unknown"
}

/// Handle a REPL colon-command.  Returns `true` if the REPL should exit.
pub fn handle_repl_command(
    cmd: &str,
    vm: &mut hudhudscript_vm::VM,
    debug: bool,
    history: &[String],
) -> bool {
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    match parts.first().copied() {
        Some(":help") | Some(":h") => {
            show_repl_help();
        }
        Some(":clear") | Some(":c") => {
            print!("\x1B[2J\x1B[1;1H"); // ANSI: clear screen, cursor home
        }
        Some(":reset") | Some(":r") => {
            *vm = hudhudscript_vm::VM::new();
            register_vm_stdlib_modules(vm);
            println!("VM reset");
        }
        Some(":debug") | Some(":d") => {
            println!("Debug mode: {}", if debug { "ON" } else { "OFF" });
        }
        Some(":vars") | Some(":v") => {
            show_vars(vm);
        }
        Some(":history") => {
            // Optional argument: :history N  →  show last N entries
            let limit: usize = parts
                .get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(history.len());
            let start = history.len().saturating_sub(limit);
            for (i, entry) in history[start..].iter().enumerate() {
                println!("{:4}  {}", start + i + 1, entry);
            }
        }
        Some(":quit") | Some(":q") => {
            return true;
        }
        _ => {
            println!("Unknown command: {}", cmd);
            println!("Type ':help' for available commands");
        }
    }
    false
}

pub fn show_repl_help() {
    println!("REPL Commands:");
    println!("  :help, :h          Show this help");
    println!("  :clear, :c         Clear screen");
    println!("  :reset, :r         Reset interpreter state");
    println!("  :debug, :d         Show debug status");
    println!("  :vars, :v          Show variables");
    println!("  :history [N]       Show command history (last N entries, default: all)");
    println!("  :quit, :q          Exit REPL");
    println!();
    println!("Special:");
    println!("  exit, quit         Exit REPL");
    println!("  Ctrl+C             Cancel multiline input or exit");
    println!("  Ctrl+D             Exit REPL");
    println!();
    println!("Multiline input:");
    println!("  Lines ending with {{ ( [ will continue on next line");
    println!("  Lines ending with }} ) ] will execute the buffer");
    println!("  Lines ending with \\ will continue on next line (backslash continuation)");
}
