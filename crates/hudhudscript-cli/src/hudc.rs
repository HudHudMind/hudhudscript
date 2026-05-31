//! hudc - HudHud Compiler
//!
//! Standalone compiler CLI for HudHudScript.
//! Supports both .hud and .hudhud file extensions.

// P7.2 — swap the system allocator for mimalloc.  The `--run` path executes
// bytecode via the VM, which also benefits from faster allocation.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::fs;
use std::path::PathBuf;
use std::process;

use clap::Parser;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

#[derive(Parser)]
#[command(name = "hudc")]
#[command(version, about = "HudHud Compiler - Compile HudHudScript to bytecode", long_about = None)]
struct Cli {
    /// Input file (.hud or .hudhud)
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output file (.hudb) - defaults to input with .hudb extension
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Show detailed compilation info
    #[arg(short, long)]
    verbose: bool,

    /// Show bytecode instructions
    #[arg(long)]
    show_bytecode: bool,

    /// Syntax check only (parse, no compile)
    #[arg(long)]
    check: bool,

    /// Show AST
    #[arg(long)]
    ast: bool,

    /// Compile and run immediately
    #[arg(long)]
    run: bool,
}

/// Exit codes:
/// - 0: success
/// - 1: runtime error
/// - 2: parse/compile error
/// - 3: file not found / IO error
fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(&cli) {
        eprintln!("Error: {}", e.msg);
        process::exit(e.code);
    }
}

/// Structured error for hudc with exit code.
struct HudcError {
    code: i32,
    msg: String,
}

impl HudcError {
    fn io(msg: String) -> Self {
        Self { code: 3, msg }
    }
    fn parse(msg: String) -> Self {
        Self { code: 2, msg }
    }
    fn runtime(msg: String) -> Self {
        Self { code: 1, msg }
    }
}

fn run(cli: &Cli) -> Result<(), HudcError> {
    // Read source file
    let source = fs::read_to_string(&cli.input)
        .map_err(|e| HudcError::io(format!("Failed to read file: {}", e)))?;

    if cli.verbose {
        println!("📖 Reading: {}", cli.input.display());
    }

    // Parse
    let ast = parse(&source).map_err(|e| HudcError::parse(format!("Parse error: {}", e)))?;

    if cli.verbose {
        println!("🔍 Parsing...");
        println!("✅ Parsed {} statements", ast.len());
    }

    // --ast: show AST and exit
    if cli.ast {
        println!("AST:");
        println!("{:#?}", ast);
        return Ok(());
    }

    // --check: syntax check only
    if cli.check {
        println!("✓ Syntax OK ({} statements)", ast.len());
        return Ok(());
    }

    // Compile
    if cli.verbose {
        println!("⚙️  Compiling to bytecode...");
    }

    let mut compiler = Compiler::new();
    let bytecode = compiler
        .compile(&ast)
        .map_err(|e| HudcError::parse(format!("Compile error: {}", e)))?;

    if cli.verbose {
        println!("✅ Compiled!");
        println!("   Constants: {}", bytecode.constants.len());
        println!("   Instructions: {}", bytecode.instructions.len());
    }

    if cli.show_bytecode {
        println!("\n📋 Bytecode:");
        println!("   Version: {}", bytecode.version);
        println!("   Constants:");
        for (i, constant) in bytecode.constants.iter().enumerate() {
            println!("     [{}] {:?}", i, constant);
        }
        println!("   Instructions:");
        for (i, instruction) in bytecode.instructions.iter().enumerate() {
            println!("     [{}] {:?}", i, instruction);
        }
        println!("   Functions:");
        for (name, chunk) in bytecode.functions.borrow().iter() {
            println!("     Function: {} (local_count={}, local_names={:?})", name, chunk.local_count, chunk.local_names);
            for (i, instruction) in chunk.instructions.iter().enumerate() {
                println!("       [{}] {:?}", i, instruction);
            }
        }
        println!();
    }

    // Determine output path
    let output_path = cli.output.clone().unwrap_or_else(|| {
        let mut p = cli.input.clone();
        p.set_extension("hudb");
        p
    });

    // Save bytecode
    let bytes = bytecode
        .to_bytes()
        .map_err(|e| HudcError::io(format!("Serialization error: {}", e)))?;

    fs::write(&output_path, &bytes)
        .map_err(|e| HudcError::io(format!("Failed to write output: {}", e)))?;

    println!("💾 Compiled to: {}", output_path.display());
    println!("   Size: {} bytes", bytes.len());

    // --run: compile and execute immediately
    if cli.run {
        println!();
        let locale = VM::detect_locale(&source);
        let mut vm = VM::with_locale(locale);
        vm.execute(&bytecode)
            .map_err(|e| HudcError::runtime(format!("VM error: {}", e)))?;
    }

    Ok(())
}
