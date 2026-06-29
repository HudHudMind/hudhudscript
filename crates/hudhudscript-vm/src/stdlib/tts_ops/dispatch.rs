use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use super::{
    engine_ops::is_binary_available,
    synthesis_ops::{
        empty_opts, error_obj, espeak_save_args, espeak_speak_args, extract_options,
        require_string, run_command,
    },
};

pub fn tts_speak(args: &[Value16]) -> SharedResult<Value16> {
    if args.is_empty() {
        return Err(runtime_error(
            "tts.speak() requires at least 1 argument: text",
        ));
    }
    let text = require_string(&args[0], "tts.speak text")?;
    let opts = if args.len() > 1 {
        extract_options(&args[1])
    } else {
        empty_opts()
    };

    if is_binary_available("espeak-ng") {
        let cmd_args = espeak_speak_args(&text, &opts);
        return run_command("espeak-ng", &cmd_args);
    }

    if is_binary_available("piper") {
        let mut cmd_args = vec!["--output-raw".to_string()];
        if let Some(ref v) = opts.voice {
            cmd_args.push("--model".to_string());
            cmd_args.push(v.clone());
        }
        let piper = Command::new("piper")
            .args(&cmd_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match piper {
            Ok(mut child) => {
                if let Some(ref mut stdin) = child.stdin {
                    let _ = stdin.write_all(text.as_bytes());
                }
                drop(child.stdin.take());

                if !is_binary_available("aplay") {
                    let _ = child.wait();
                    return Ok(error_obj(
                        "piper available but no audio player (aplay) found".to_string(),
                    ));
                }

                let output = match child.wait_with_output() {
                    Ok(o) => o,
                    Err(e) => return Ok(error_obj(format!("piper failed: {}", e))),
                };

                let aplay = Command::new("aplay")
                    .args(["-r", "22050", "-f", "S16_LE", "-t", "raw", "-c", "1"])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .spawn();

                match aplay {
                    Ok(mut aplay_child) => {
                        if let Some(ref mut stdin) = aplay_child.stdin {
                            let _ = stdin.write_all(&output.stdout);
                        }
                        drop(aplay_child.stdin.take());
                        let aplay_out = aplay_child.wait_with_output();
                        let mut obj = hudhudscript_bytecode::ObjMap::default();
                        let success = aplay_out
                            .as_ref()
                            .map(|o| o.status.success())
                            .unwrap_or(false);
                        obj.insert("ok".to_string(), Value16::boolean(success));
                        obj.insert("code".to_string(), Value16::number(0.0));
                        if !success {
                            if let Ok(ref o) = aplay_out {
                                obj.insert(
                                    "error".to_string(),
                                    Value16::string(String::from_utf8_lossy(&o.stderr).to_string()),
                                );
                            }
                        }
                        Ok(Value16::object(obj))
                    }
                    Err(e) => Ok(error_obj(format!(
                        "piper succeeded but aplay failed: {}",
                        e
                    ))),
                }
            }
            Err(e) => Ok(error_obj(format!("Failed to execute piper: {}", e))),
        }
    } else if is_binary_available("festival") {
        let child = Command::new("festival")
            .arg("--tts")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();

        match child {
            Ok(mut c) => {
                if let Some(ref mut stdin) = c.stdin {
                    let _ = stdin.write_all(text.as_bytes());
                }
                drop(c.stdin.take());
                let output = c.wait_with_output();
                let mut obj = hudhudscript_bytecode::ObjMap::default();
                let success = output.as_ref().map(|o| o.status.success()).unwrap_or(false);
                obj.insert("ok".to_string(), Value16::boolean(success));
                obj.insert(
                    "code".to_string(),
                    Value16::number(if success { 0.0 } else { -1.0 }),
                );
                if !success {
                    if let Ok(ref o) = output {
                        obj.insert(
                            "error".to_string(),
                            Value16::string(String::from_utf8_lossy(&o.stderr).to_string()),
                        );
                    }
                }
                Ok(Value16::object(obj))
            }
            Err(e) => Ok(error_obj(format!("Failed to execute festival: {}", e))),
        }
    } else {
        Ok(error_obj(
            "No TTS engine found. Install espeak-ng, piper, or festival.".to_string(),
        ))
    }
}

pub fn tts_save(args: &[Value16]) -> SharedResult<Value16> {
    if args.len() < 2 {
        return Err(runtime_error(
            "tts.save() requires at least 2 arguments: text, output_path",
        ));
    }
    let text = require_string(&args[0], "tts.save text")?;
    let output_path = require_string(&args[1], "tts.save output_path")?;
    let opts = if args.len() > 2 {
        extract_options(&args[2])
    } else {
        empty_opts()
    };

    if is_binary_available("espeak-ng") {
        let cmd_args = espeak_save_args(&text, &output_path, &opts);
        return run_command("espeak-ng", &cmd_args);
    }

    if is_binary_available("piper") {
        let mut cmd_args = vec!["--output_file".to_string(), output_path.clone()];
        if let Some(ref v) = opts.voice {
            cmd_args.push("--model".to_string());
            cmd_args.push(v.clone());
        }

        let child = Command::new("piper")
            .args(&cmd_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();

        match child {
            Ok(mut c) => {
                if let Some(ref mut stdin) = c.stdin {
                    let _ = stdin.write_all(text.as_bytes());
                }
                drop(c.stdin.take());
                let output = c.wait_with_output();
                let mut obj = hudhudscript_bytecode::ObjMap::default();
                let success = output.as_ref().map(|o| o.status.success()).unwrap_or(false);
                obj.insert("ok".to_string(), Value16::boolean(success));
                obj.insert(
                    "code".to_string(),
                    Value16::number(if success { 0.0 } else { -1.0 }),
                );
                obj.insert("path".to_string(), Value16::string(output_path));
                if !success {
                    if let Ok(ref o) = output {
                        obj.insert(
                            "error".to_string(),
                            Value16::string(String::from_utf8_lossy(&o.stderr).to_string()),
                        );
                    }
                }
                return Ok(Value16::object(obj));
            }
            Err(e) => return Ok(error_obj(format!("Failed to execute piper: {}", e))),
        }
    }

    if is_binary_available("text2wave") {
        let child = Command::new("text2wave")
            .args(["-o", &output_path])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();

        match child {
            Ok(mut c) => {
                if let Some(ref mut stdin) = c.stdin {
                    let _ = stdin.write_all(text.as_bytes());
                }
                drop(c.stdin.take());
                let output = c.wait_with_output();
                let mut obj = hudhudscript_bytecode::ObjMap::default();
                let success = output.as_ref().map(|o| o.status.success()).unwrap_or(false);
                obj.insert("ok".to_string(), Value16::boolean(success));
                obj.insert(
                    "code".to_string(),
                    Value16::number(if success { 0.0 } else { -1.0 }),
                );
                obj.insert("path".to_string(), Value16::string(output_path));
                if !success {
                    if let Ok(ref o) = output {
                        obj.insert(
                            "error".to_string(),
                            Value16::string(String::from_utf8_lossy(&o.stderr).to_string()),
                        );
                    }
                }
                return Ok(Value16::object(obj));
            }
            Err(e) => {
                return Ok(error_obj(format!("Failed to execute text2wave: {}", e)));
            }
        }
    }

    Ok(error_obj(
        "No TTS engine found. Install espeak-ng, piper, or festival.".to_string(),
    ))
}

pub fn tts_ssml(args: &[Value16]) -> SharedResult<Value16> {
    if args.is_empty() {
        return Err(runtime_error("tts.ssml() requires 1 argument: ssml_text"));
    }
    let ssml_text = require_string(&args[0], "tts.ssml ssml_text")?;

    if is_binary_available("espeak-ng") {
        let cmd_args = vec!["-m".to_string(), ssml_text];
        return run_command("espeak-ng", &cmd_args);
    }

    if is_binary_available("festival") {
        let child = Command::new("festival")
            .args(["--language", "ssml"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn();

        match child {
            Ok(mut c) => {
                if let Some(ref mut stdin) = c.stdin {
                    let _ = stdin.write_all(ssml_text.as_bytes());
                }
                drop(c.stdin.take());
                let output = c.wait_with_output();
                let mut obj = hudhudscript_bytecode::ObjMap::default();
                let success = output.as_ref().map(|o| o.status.success()).unwrap_or(false);
                obj.insert("ok".to_string(), Value16::boolean(success));
                obj.insert(
                    "code".to_string(),
                    Value16::number(if success { 0.0 } else { -1.0 }),
                );
                if !success {
                    if let Ok(ref o) = output {
                        obj.insert(
                            "error".to_string(),
                            Value16::string(String::from_utf8_lossy(&o.stderr).to_string()),
                        );
                    }
                }
                return Ok(Value16::object(obj));
            }
            Err(e) => {
                return Ok(error_obj(format!("Failed to execute festival: {}", e)));
            }
        }
    }

    Ok(error_obj(
        "No TTS engine with SSML support found. Install espeak-ng or festival.".to_string(),
    ))
}
