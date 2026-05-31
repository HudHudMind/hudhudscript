//! tar.gz creation and extraction operations.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::process::Command;

use super::utils::{require_str, require_string_array, run_command, runtime_error};

pub fn create_tar_gz(args: &[Value16]) -> HudHudResult<Value16> {
    let output_path = require_str(args, 0, "archive.create_tar_gz")?.to_string();
    let files = require_string_array(args, 1, "archive.create_tar_gz")?;

    if files.is_empty() {
        return Err(runtime_error(
            "archive.create_tar_gz: input_files array must not be empty",
        ));
    }

    let mut cmd = Command::new("tar");
    cmd.arg("czf").arg(&output_path);
    for f in &files {
        cmd.arg(f);
    }

    run_command(cmd, "archive.create_tar_gz")?;
    Ok(Value16::string(output_path))
}

pub fn extract_tar_gz(args: &[Value16]) -> HudHudResult<Value16> {
    let archive_path = require_str(args, 0, "archive.extract_tar_gz")?.to_string();
    let output_dir = require_str(args, 1, "archive.extract_tar_gz")?.to_string();

    std::fs::create_dir_all(&output_dir).map_err(|e| {
        runtime_error(format!(
            "archive.extract_tar_gz: cannot create output dir: {}",
            e
        ))
    })?;

    let cmd = Command::new("tar")
        .arg("xzf")
        .arg(&archive_path)
        .arg("-C")
        .arg(&output_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|c| c.wait_with_output());

    match cmd {
        Ok(output) if output.status.success() => Ok(Value16::string(output_dir)),
        Ok(output) => Err(runtime_error(format!(
            "archive.extract_tar_gz failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))),
        Err(e) => Err(runtime_error(format!("archive.extract_tar_gz: {}", e))),
    }
}
