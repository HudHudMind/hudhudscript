use crate::common::{CliError, HudHudConfig};
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

pub fn compile_file(
    path: &PathBuf,
    output: Option<PathBuf>,
    verbose: bool,
    strict: bool,
    lint: &crate::common::LintConfig,
) -> Result<(), CliError> {
    // Read file
    let source = fs::read_to_string(path)
        .map_err(|e| CliError::Io(format!("Failed to read file: {}", e)))?;

    if verbose {
        println!("📖 Reading: {}", path.display());
    }

    // Parse with recovery — report all errors, continue with partial AST if possible.
    let (ast, parse_errors) = parse_with_recovery(&source);

    // Report all parse errors (not just the first)
    if !parse_errors.is_empty() {
        for err in &parse_errors {
            eprintln!("{}", err.render_full());
        }
        if ast.is_empty() {
            return Err(CliError::ParseCompile(format!(
                "{} parse error(s) found",
                parse_errors.len()
            )));
        }
        // If we got some valid statements, continue with partial AST
        eprintln!(
            "Warning: {} error(s) found, continuing with partial parse",
            parse_errors.len()
        );
    }

    if verbose {
        println!("🔍 Parsing...");
        println!("✅ Parsed {} statements", ast.len());
    }

    // Issue #866 TYPE-001: Run type checker before compilation when --strict is enabled
    // Issue #920: Use AnnotatedAST pipeline (TypeChecker → AnnotatedAST → Compiler)
    let mut compiler = Compiler::new();
    let bytecode = if strict {
        if verbose {
            println!("🔒 Strict type checking...");
        }
        let mut type_checker = hudhudscript_types::TypeChecker::new_strict();
        type_checker.set_redeclare_policy(lint_policy(lint));
        let annotated = type_checker.check_and_annotate(ast).map_err(|errors| {
            CliError::ParseCompile(
                errors
                    .iter()
                    .map(|e| format!("Type error: {}", e))
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        })?;
        for diag in &annotated.diagnostics {
            eprintln!("Type warning: {}", diag.message);
        }
        if verbose {
            println!("✅ Strict type checking passed");
            println!("⚙️  Compiling annotated AST to bytecode...");
        }
        compiler.compile_annotated(&annotated).map_err(|e| {
            let unified: hudhudscript_errors::Error = e;
            CliError::ParseCompile(unified.render_full())
        })?
    } else {
        if verbose {
            println!("⚙️  Compiling to bytecode...");
        }
        compiler.compile(&ast).map_err(|e| {
            let unified: hudhudscript_errors::Error = e;
            CliError::ParseCompile(unified.render_full())
        })?
    };

    if verbose {
        println!("✅ Compiled!");
        println!("   Constants: {}", bytecode.constants.len());
        println!("   Instructions: {}", bytecode.instructions.len());
    }

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let mut p = path.clone();
        p.set_extension("hudb");
        p
    });

    // Save bytecode
    let bytes = bytecode
        .to_bytes()
        .map_err(|e| CliError::Io(format!("Serialization error: {}", e)))?;

    fs::write(&output_path, &bytes)
        .map_err(|e| CliError::Io(format!("Failed to write output: {}", e)))?;

    println!("💾 Compiled to: {}", output_path.display());
    println!("   Size: {} bytes", bytes.len());

    Ok(())
}

/// G2: Convert CLI RedeclarePolicy to checker RedeclarePolicy (Kural 7 — single source).
fn lint_policy(lint: &crate::common::LintConfig) -> hudhudscript_types::checker::RedeclarePolicy {
    match lint.redeclare {
        crate::common::RedeclarePolicy::Allow => hudhudscript_types::checker::RedeclarePolicy::Allow,
        crate::common::RedeclarePolicy::Warn  => hudhudscript_types::checker::RedeclarePolicy::Warn,
        crate::common::RedeclarePolicy::Error => hudhudscript_types::checker::RedeclarePolicy::Error,
    }
}

pub fn check_file(path: &PathBuf, show_ast: bool, strict: bool, lint: &crate::common::LintConfig) -> Result<(), CliError> {
    // Read file
    let source = fs::read_to_string(path)
        .map_err(|e| CliError::Io(format!("Failed to read file: {}", e)))?;

    // Parse with recovery — report all errors, continue with partial AST if possible.
    let (ast, parse_errors) = parse_with_recovery(&source);

    // Report all parse errors (not just the first)
    if !parse_errors.is_empty() {
        for err in &parse_errors {
            eprintln!("{}", err.render_full());
        }
        if ast.is_empty() {
            return Err(CliError::ParseCompile(format!(
                "{} parse error(s) found",
                parse_errors.len()
            )));
        }
        // If we got some valid statements, continue with partial AST
        eprintln!(
            "Warning: {} error(s) found, continuing with partial parse",
            parse_errors.len()
        );
    }

    println!("✓ Syntax OK");

    // Issue #1010: Always run type checker; strict mode controls severity.
    {
        let mut type_checker = if strict {
            hudhudscript_types::TypeChecker::new_strict()
        } else {
            hudhudscript_types::TypeChecker::new()
        };
        type_checker.set_redeclare_policy(lint_policy(lint));
        if let Err(e) = type_checker.check_program(&ast) {
            // G2: redeclare="error" is always fatal, regardless of strict mode
            let is_fatal = strict || matches!(lint.redeclare, crate::common::RedeclarePolicy::Error);
            if is_fatal {
                return Err(CliError::ParseCompile(format!("Type error: {}", e)));
            } else {
                eprintln!("Type hint: {}", e);
            }
        }
        let diagnostics = type_checker.errors();
        if !diagnostics.is_empty() {
            let label = if strict { "Type error" } else { "Type hint" };
            for err in diagnostics {
                eprintln!("{}: {}", label, err);
            }
            if strict {
                return Err(CliError::ParseCompile(format!(
                    "{} type error(s) found",
                    diagnostics.len()
                )));
            }
        }
        if strict {
            println!("✓ Type check OK (strict mode)");
        } else {
            println!("✓ Type check OK");
        }
    }

    if show_ast {
        println!("\nAST:");
        println!("{:#?}", ast);
    }

    Ok(())
}
