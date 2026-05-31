//! Compress / decompress operations (gzip, deflate).

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::utils::{require_str, runtime_error, shell_pipe};

pub fn compress(args: &[Value16]) -> HudHudResult<Value16> {
    use base64::Engine;
    let data = require_str(args, 0, "archive.compress")?.to_string();
    let algorithm = if args.len() > 1 {
        require_str(args, 1, "archive.compress")?.to_string()
    } else {
        "gzip".to_string()
    };

    let compressed = match algorithm.to_lowercase().as_str() {
        "gzip" | "gz" => shell_pipe("gzip", &["-c"], data.as_bytes(), "archive.compress (gzip)")?,
        "deflate" => shell_pipe(
            "python3",
            &[
                "-c",
                "import sys,zlib;sys.stdout.buffer.write(zlib.compress(sys.stdin.buffer.read()))",
            ],
            data.as_bytes(),
            "archive.compress (deflate)",
        )?,
        other => {
            return Err(runtime_error(format!(
                "archive.compress: unsupported algorithm '{}' (supported: gzip, deflate)",
                other
            )));
        }
    };

    let encoded = base64::engine::general_purpose::STANDARD.encode(&compressed);
    Ok(Value16::string(encoded))
}

pub fn decompress(args: &[Value16]) -> HudHudResult<Value16> {
    use base64::Engine;
    let compressed_b64 = require_str(args, 0, "archive.decompress")?.to_string();
    let algorithm = if args.len() > 1 {
        require_str(args, 1, "archive.decompress")?.to_string()
    } else {
        "gzip".to_string()
    };

    let compressed = base64::engine::general_purpose::STANDARD
        .decode(compressed_b64.as_bytes())
        .map_err(|e| runtime_error(format!("archive.decompress base64 error: {}", e)))?;

    let decompressed = match algorithm.to_lowercase().as_str() {
        "gzip" | "gz" => shell_pipe("gzip", &["-dc"], &compressed, "archive.decompress (gzip)")?,
        "deflate" => shell_pipe(
            "python3",
            &[
                "-c",
                "import sys,zlib;sys.stdout.buffer.write(zlib.decompress(sys.stdin.buffer.read()))",
            ],
            &compressed,
            "archive.decompress (deflate)",
        )?,
        other => {
            return Err(runtime_error(format!(
                "archive.decompress: unsupported algorithm '{}' (supported: gzip, deflate)",
                other
            )));
        }
    };

    let text = String::from_utf8(decompressed)
        .map_err(|e| runtime_error(format!("archive.decompress UTF-8 error: {}", e)))?;
    Ok(Value16::string(text))
}
