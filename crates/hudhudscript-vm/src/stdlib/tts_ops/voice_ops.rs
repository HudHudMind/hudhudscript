use hudhudscript_bytecode::shared_value::SharedResult;
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;
use std::process::Command;

use super::engine_ops::is_binary_available;

pub fn tts_voices(_args: &[Value16]) -> SharedResult<Value16> {
    if is_binary_available("espeak-ng") {
        let result = Command::new("espeak-ng").arg("--voices").output();

        if let Ok(output) = result {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut voices = Vec::new();

                for line in stdout.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let mut voice = hudhudscript_bytecode::ObjMap::default();
                        voice.insert(
                            "language".to_string(),
                            Value16::string(parts[1].to_string()),
                        );
                        voice.insert("gender".to_string(), Value16::string(parts[2].to_string()));
                        voice.insert("name".to_string(), Value16::string(parts[3].to_string()));
                        voices.push(Value16::object(voice));
                    }
                }

                return Ok(Value16::array(voices));
            }
        }
    }

    Ok(Value16::array(Vec::new()))
}
