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

pub fn format_path(path: &PathBuf, write: bool, check: bool) -> Result<(), CliError> {
    if path.is_dir() {
        format_directory(path, write, check)
    } else {
        format_file(path, write, check)
    }
}

/// Recursively find and format all .hudhud / .hud files in a directory.
fn format_directory(dir: &PathBuf, write: bool, check: bool) -> Result<(), CliError> {
    let mut unformatted: Vec<PathBuf> = Vec::new();
    let mut file_count = 0u64;

    fn walk(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), CliError> {
        let entries = fs::read_dir(dir).map_err(|e| {
            CliError::Io(format!("Failed to read directory {}: {}", dir.display(), e))
        })?;
        for entry in entries {
            let entry = entry.map_err(|e| CliError::Io(format!("Directory entry error: {}", e)))?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, files)?;
            } else if let Some(ext) = path.extension() {
                if ext == "hudhud" || ext == "hud" {
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    walk(dir.as_path(), &mut files)?;
    files.sort();

    for file_path in &files {
        file_count += 1;
        match format_file(file_path, write, check) {
            Ok(()) => {}
            Err(CliError::Runtime(msg)) if check && msg.starts_with("File needs formatting") => {
                unformatted.push(file_path.clone());
            }
            Err(e) => {
                eprintln!("Error formatting {}: {}", file_path.display(), e);
            }
        }
    }

    if check && !unformatted.is_empty() {
        eprintln!(
            "{} file(s) need formatting out of {} checked:",
            unformatted.len(),
            file_count
        );
        for f in &unformatted {
            eprintln!("  {}", f.display());
        }
        return Err(CliError::Runtime(format!(
            "{} file(s) need formatting",
            unformatted.len()
        )));
    }

    if file_count == 0 {
        println!("No .hudhud or .hud files found in {}", dir.display());
    } else if check {
        println!("All {} file(s) are properly formatted.", file_count);
    } else if write {
        println!("Formatted {} file(s).", file_count);
    }

    Ok(())
}

pub fn format_file(path: &PathBuf, write: bool, check: bool) -> Result<(), CliError> {
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

    // Format
    let mut formatter = Formatter::new();
    let formatted = formatter.format_program(&ast);

    if check {
        // Check mode: compare formatted output with original source
        if formatted != source {
            return Err(CliError::Runtime(format!(
                "File needs formatting: {}",
                path.display()
            )));
        }
        return Ok(());
    }

    if write {
        // Write formatted code back to file
        fs::write(path, &formatted)
            .map_err(|e| CliError::Io(format!("Failed to write file: {}", e)))?;
        println!("Formatted {}", path.display());
    } else {
        // Print formatted code to stdout
        println!("{}", formatted);
    }

    Ok(())
}

/// Lint a HudHudScript file and report diagnostics
#[allow(dead_code)] // Used from main.rs (multi-binary false positive)
pub fn lint_file(path: &PathBuf) -> Result<(), CliError> {
    let source = fs::read_to_string(path)
        .map_err(|e| CliError::Io(format!("Failed to read file: {}", e)))?;

    let ast = parse(&source).map_err(|e| {
        let unified: hudhudscript_errors::Error = e;
        CliError::ParseCompile(unified.render_full())
    })?;

    let diagnostics = hudhudscript_linter::lint_default(&ast);

    if diagnostics.is_empty() {
        println!("No lint issues found in {}", path.display());
    } else {
        println!("Lint issues in {}:", path.display());
        for d in &diagnostics {
            println!("  {}", d);
        }
        if diagnostics.len() == 1 {
            println!("\n1 issue found.");
        } else {
            println!("\n{} issues found.", diagnostics.len());
        }
    }

    Ok(())
}
