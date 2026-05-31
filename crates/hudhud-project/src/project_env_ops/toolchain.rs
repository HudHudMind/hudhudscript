use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::helpers::require_string;

pub fn toolchain_version(args: &[Value16]) -> HudHudResult<Value16> {
    let tool = require_string(args, 0, "project.toolchain_version")?;

    let (cmd, cmd_args): (&str, &[&str]) = match tool.as_str() {
        "python" | "python3" => ("python3", &["--version"]),
        "node" | "nodejs" => ("node", &["--version"]),
        "rustc" | "rust" => ("rustc", &["--version"]),
        "go" | "golang" => ("go", &["version"]),
        "ruby" => ("ruby", &["--version"]),
        "gcc" => ("gcc", &["--version"]),
        "cmake" => ("cmake", &["--version"]),
        "cargo" => ("cargo", &["--version"]),
        "npm" => ("npm", &["--version"]),
        "pip" | "pip3" => ("pip3", &["--version"]),
        other => (other, &["--version"]),
    };

    match std::process::Command::new(cmd).args(cmd_args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let version_str = if stdout.trim().is_empty() {
                stderr.trim().to_string()
            } else {
                stdout.trim().to_string()
            };
            let first_line = version_str.lines().next().unwrap_or("").to_string();
            Ok(Value16::string(first_line))
        }
        Err(_) => Ok(Value16::null()),
    }
}
