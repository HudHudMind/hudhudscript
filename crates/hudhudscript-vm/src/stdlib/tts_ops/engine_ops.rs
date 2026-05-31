use hudhudscript_bytecode::shared_value::SharedResult;
use hudhudscript_bytecode::Value16;
use std::process::{Command, Stdio};

use super::ENGINES;

pub(crate) fn is_binary_available(bin: &str) -> bool {
    Command::new("which")
        .arg(bin)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub(crate) fn find_engine() -> Option<&'static str> {
    for &(_name, bin) in ENGINES {
        if is_binary_available(bin) {
            return Some(bin);
        }
    }
    None
}

pub fn tts_engines(_args: &[Value16]) -> SharedResult<Value16> {
    let available: Vec<Value16> = ENGINES
        .iter()
        .filter(|&&(_name, bin)| is_binary_available(bin))
        .map(|&(name, _bin)| Value16::string(name.to_string()))
        .collect();
    Ok(Value16::array(available))
}

pub fn tts_is_available(_args: &[Value16]) -> SharedResult<Value16> {
    Ok(Value16::boolean(find_engine().is_some()))
}
