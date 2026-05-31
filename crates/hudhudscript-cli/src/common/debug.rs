use crate::common::{
    load_hudhud_config_with_path, register_vm_stdlib_modules, CliError, HudHudConfig,
};
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

pub fn run_debug(
    path: &PathBuf,
    breakpoints: &[String],
    stop_on_entry: bool,
) -> Result<(), CliError> {
    run_debug_with_config(path, breakpoints, stop_on_entry, None)
}

/// Run debug with an optional explicit config path (Issue #1006).
pub fn run_debug_with_config(
    path: &PathBuf,
    breakpoints: &[String],
    stop_on_entry: bool,
    config_path: Option<&std::path::Path>,
) -> Result<(), CliError> {
    use hudhudscript_debug::{DebugState, Debugger};
    use std::io::{self, BufRead, Write};

    let source = fs::read_to_string(path)
        .map_err(|e| CliError::Io(format!("Failed to read file: {}", e)))?;
    let ast = parse(&source).map_err(|e| {
        let unified: hudhudscript_errors::Error = e;
        CliError::ParseCompile(unified.render_full())
    })?;
    let file_str = path.to_string_lossy().to_string();

    let mut debugger = Debugger::new();

    // Set initial breakpoints
    for bp_str in breakpoints {
        let parts: Vec<&str> = bp_str.rsplitn(2, ':').collect();
        if parts.len() == 2 {
            if let Ok(line) = parts[0].parse::<usize>() {
                let file = parts[1].to_string();
                let id = debugger.add_breakpoint(file.clone(), line);
                println!("Breakpoint {} set at {}:{}", id, file, line);
            }
        } else if let Ok(line) = bp_str.parse::<usize>() {
            let id = debugger.add_breakpoint(file_str.clone(), line);
            println!("Breakpoint {} set at {}:{}", id, file_str, line);
        }
    }

    if stop_on_entry {
        debugger.pause();
    }

    let config = load_hudhud_config_with_path(false, config_path);

    // Compile up-front so bytecode carries source positions for
    // statement-level breakpoints (Issue #661 / 3a6c17da).
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).map_err(|e| {
        let unified: hudhudscript_errors::Error = e;
        CliError::ParseCompile(unified.render_full())
    })?;

    let mut vm = if config.runtime.fuel_limit > 0 {
        let mut v = VM::new();
        v.with_fuel(config.runtime.fuel_limit);
        v
    } else {
        VM::new()
    };
    register_vm_stdlib_modules(&mut vm);
    vm.set_current_file(file_str.clone());
    vm.attach_debugger(debugger);

    println!("HudHud Debugger — type 'help' for commands");
    if stop_on_entry {
        println!("Stopped on entry at {}:1", file_str);
    }

    // Run VM in a separate thread so the main thread drives the REPL.
    // The debugger's on_statement / on_exception hooks busy-wait when
    // paused; the REPL below flips `resume` / `step` state via the
    // debugger handle shared through the Arc<Mutex<VM>>.
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CliError::Runtime(format!("Failed to create runtime: {}", e)))?;

    let vm_arc = std::sync::Arc::new(std::sync::Mutex::new(vm));
    let vm_thread = vm_arc.clone();
    let handle = std::thread::spawn(move || {
        let mut vm = vm_thread.lock().unwrap();
        rt.block_on(async { vm.execute(&bytecode) })
    });

    // Interactive debugger REPL
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        // Check if VM thread finished
        if handle.is_finished() {
            match handle.join() {
                Ok(Ok(())) => println!("Program finished"),
                Ok(Err(e)) => println!("Runtime error: {}", e),
                Err(_) => println!("VM thread panicked"),
            }
            break;
        }

        // Check if debugger is paused
        {
            let vm = vm_arc.lock().unwrap();
            if let Some(dbg) = vm.debugger_ref() {
                if dbg.state() != DebugState::Paused {
                    drop(vm);
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    continue;
                }
                if let Some((file, line)) = dbg.current_location() {
                    println!("Paused at {}:{}", file, line);
                }
                if let Some(reason) = dbg.pause_reason() {
                    println!("Reason: {:?}", reason);
                }
            } else {
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }
        }

        // Prompt
        print!("(hud-dbg) ");
        stdout.flush().unwrap();

        let mut input = String::new();
        if stdin.lock().read_line(&mut input).is_err() {
            break;
        }
        let cmd = input.trim();

        let mut vm = vm_arc.lock().unwrap();
        let dbg = vm.debugger_mut().unwrap();

        match cmd {
            "c" | "continue" => {
                dbg.resume();
            }
            "n" | "next" | "step_over" => {
                dbg.step(hudhudscript_debug::StepMode::Over);
            }
            "s" | "step" | "step_into" => {
                dbg.step(hudhudscript_debug::StepMode::Into);
            }
            "o" | "out" | "step_out" => {
                dbg.step(hudhudscript_debug::StepMode::Out);
            }
            "bt" | "backtrace" => {
                let stack = dbg.call_stack();
                if stack.is_empty() {
                    println!("  (empty call stack)");
                } else {
                    for (i, frame) in stack.iter().rev().enumerate() {
                        println!("  #{} {}", i, frame);
                    }
                }
            }
            "q" | "quit" => {
                println!("Debugger exiting.");
                std::process::exit(0);
            }
            "help" | "h" => {
                println!("Commands:");
                println!("  c, continue    — Resume execution");
                println!("  n, next        — Step over");
                println!("  s, step        — Step into");
                println!("  o, out         — Step out");
                println!("  bt, backtrace  — Show call stack");
                println!("  b <line>       — Add breakpoint at line");
                println!("  q, quit        — Exit debugger");
            }
            _ if cmd.starts_with("b ") || cmd.starts_with("break ") => {
                let rest = cmd.split_whitespace().nth(1).unwrap_or("");
                if let Ok(line) = rest.parse::<usize>() {
                    let id = dbg.add_breakpoint(file_str.clone(), line);
                    println!("Breakpoint {} set at {}:{}", id, file_str, line);
                } else {
                    println!("Usage: b <line_number>");
                }
            }
            "" => {}
            _ => {
                println!("Unknown command: '{}'. Type 'help' for commands.", cmd);
            }
        }
    }

    Ok(())
}

/// Start DAP debug server (Issue #661).
/// Communicates via stdin/stdout using the Debug Adapter Protocol.
pub fn run_dap_server(path: &PathBuf) -> Result<(), CliError> {
    use hudhudscript_debug::DapServer;

    let _source = fs::read_to_string(path)
        .map_err(|e| CliError::Io(format!("Failed to read file: {}", e)))?;

    let mut server = DapServer::new();
    server
        .run_stdio()
        .map_err(|e| CliError::Runtime(format!("DAP server error: {}", e)))
}

/// Start the LSP server (#713)

pub fn run_lsp_server(transport: &str, port: u16) -> Result<(), CliError> {
    match transport {
        "stdio" => {
            eprintln!("HudHudScript LSP server starting on stdio...");
            hudhudscript_lsp::run_stdio()
                .map_err(|e| CliError::Runtime(format!("LSP server error: {}", e)))
        }
        "tcp" => {
            eprintln!(
                "HudHudScript LSP server starting on tcp://127.0.0.1:{}...",
                port
            );
            hudhudscript_lsp::run_tcp(port)
                .map_err(|e| CliError::Runtime(format!("LSP server error: {}", e)))
        }
        other => Err(CliError::Runtime(format!(
            "Unknown LSP transport '{}'. Supported: stdio, tcp",
            other
        ))),
    }
}
