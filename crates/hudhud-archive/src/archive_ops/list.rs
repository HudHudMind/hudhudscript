//! Archive listing operation.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::process::Command;

use super::utils::{require_str, runtime_error};

pub fn list_archive(args: &[Value16]) -> HudHudResult<Value16> {
    let archive_path = require_str(args, 0, "archive.list")?;

    let output = if archive_path.ends_with(".tar.gz") || archive_path.ends_with(".tgz") {
        Command::new("tar")
            .arg("tzf")
            .arg(archive_path)
            .output()
            .map_err(|e| runtime_error(format!("archive.list (tar): {}", e)))?
    } else if archive_path.ends_with(".zip") {
        Command::new("unzip")
            .arg("-l")
            .arg(archive_path)
            .output()
            .map_err(|e| runtime_error(format!("archive.list (unzip): {}", e)))?
    } else {
        return Err(runtime_error(format!(
            "archive.list: unsupported format for '{}' (supported: .tar.gz, .tgz, .zip)",
            archive_path
        )));
    };

    if !output.status.success() {
        return Err(runtime_error(format!(
            "archive.list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let entries: Vec<Value16> = if archive_path.ends_with(".zip") {
        // unzip -l layout: 3-line header + file rows + 2-line footer.
        let lines: Vec<&str> = stdout.lines().collect();
        if lines.len() > 5 {
            lines[3..lines.len() - 2]
                .iter()
                .filter_map(|line| {
                    let trimmed = line.trim();
                    let parts: Vec<&str> = trimmed.splitn(4, char::is_whitespace).collect();
                    parts.last().map(|s| {
                        let name = s.trim();
                        Value16::string(name.to_string())
                    })
                })
                .filter(|v| v.as_str().map(|s| !s.is_empty()).unwrap_or(false))
                .collect()
        } else {
            vec![]
        }
    } else {
        stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| Value16::string(l.to_string()))
            .collect()
    };

    Ok(Value16::array(entries))
}
