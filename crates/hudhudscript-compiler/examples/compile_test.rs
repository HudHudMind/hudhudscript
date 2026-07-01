//! Test program to compile HudHudScript to bytecode

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read source file
    let source_path = "samples/hello.hud";
    let source = fs::read_to_string(source_path)?;

    println!("📖 Reading: {}", source_path);
    println!("Source code:");
    println!("{}", source);
    println!();

    // Parse to AST
    println!("🔍 Parsing...");
    let ast = parse(&source)?;
    println!("✅ Parsed {} statements", ast.len());
    println!();

    // Compile to bytecode
    println!("⚙️  Compiling to bytecode...");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast)?;

    println!("✅ Compiled!");
    println!("   Constants: {}", bytecode.constants.len());
    println!("   Instructions: {}", bytecode.instructions.len());
    println!();

    // Show bytecode
    println!("📋 Bytecode:");
    println!("   Version: {}", bytecode.version);
    println!("   Constants:");
    for (i, constant) in bytecode.constants.iter().enumerate() {
        println!("     [{}] {:?}", i, constant);
    }
    println!("   Instructions:");
    for (i, instruction) in bytecode.instructions.iter().enumerate() {
        println!("     [{}] {:?}", i, instruction);
    }
    println!();

    // Save to .hudb file
    let output_path = "samples/hello.hudb";
    let bytes = bytecode.to_bytes()?;
    fs::write(output_path, &bytes)?;

    println!("💾 Saved to: {}", output_path);
    println!("   Size: {} bytes", bytes.len());
    println!();

    println!("🎉 Compilation successful!");
    println!();
    println!("📝 File extensions:");
    println!("   .hudhud  - Source code");
    println!("   .hudb    - Bytecode (HudHud Bytecode)");
    println!();
    println!("🔧 Compiler:");
    println!("   hdc      - HudHud Compiler");

    Ok(())
}
