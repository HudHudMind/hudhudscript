//! zip creation and extraction operations.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::process::Command;

use super::utils::{require_str, require_string_array, run_command, runtime_error};

pub fn create_zip(args: &[Value16]) -> HudHudResult<Value16> {
    let output_path = require_str(args, 0, "archive.create_zip")?.to_string();
    let files = require_string_array(args, 1, "archive.create_zip")?;

    if files.is_empty() {
        return Err(runtime_error(
            "archive.create_zip: input_files array must not be empty",
        ));
    }

    let mut cmd = Command::new("zip");
    cmd.arg("-r").arg(&output_path);
    for f in &files {
        cmd.arg(f);
    }

    run_command(cmd, "archive.create_zip")?;
    Ok(Value16::string(output_path))
}

pub fn extract_zip(args: &[Value16]) -> HudHudResult<Value16> {
    let archive_path = require_str(args, 0, "archive.extract_zip")?.to_string();
    let output_dir = require_str(args, 1, "archive.extract_zip")?.to_string();

    std::fs::create_dir_all(&output_dir).map_err(|e| {
        runtime_error(format!(
            "archive.extract_zip: cannot create output dir: {}",
            e
        ))
    })?;

    let cmd = Command::new("unzip")
        .arg("-o")
        .arg(&archive_path)
        .arg("-d")
        .arg(&output_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|c| c.wait_with_output());

    match cmd {
        Ok(output) if output.status.success() => Ok(Value16::string(output_dir)),
        Ok(output) => Err(runtime_error(format!(
            "archive.extract_zip failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))),
        Err(e) => Err(runtime_error(format!("archive.extract_zip: {}", e))),
    }
}
