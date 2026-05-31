//! Shared stdin reading builtin — used by both VM and interpreter.
//! INPUT0001-0005: single source for all terminal input.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StdinMethodId {
    Read,    // prompt + line, alias for input()
    Line,    // line without prompt
    Confirm, // y/N prompt
    Password,// real hidden input
    Number,  // parse as f64
    Int,     // parse as i64
    All,     // read to EOF
    Lines,   // all lines as array
}

impl std::str::FromStr for StdinMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "read" => Ok(Self::Read),
            "line" => Ok(Self::Line),
            "confirm" => Ok(Self::Confirm),
            "password" | "hidden" => Ok(Self::Password),
            "number" => Ok(Self::Number),
            "int" => Ok(Self::Int),
            "all" => Ok(Self::All),
            "lines" => Ok(Self::Lines),
            _ => Err(runtime_error(format!("Unknown stdin method: {}", s))),
        }
    }
}

fn strip_newline(s: &mut String) {
    if s.ends_with('\n') { s.pop(); if s.ends_with('\r') { s.pop(); } }
}

fn write_prompt(args: &[Value16]) {
    if let Some(prompt) = args.first().and_then(|v| v.as_str()) {
        print!("{}", prompt);
        use std::io::Write;
        std::io::stdout().flush().ok();
    }
}

/// INPUT0004: Shared read-line helper. Returns None on EOF.
fn read_line_eof() -> HudHudResult<Option<String>> {
    let mut line = String::new();
    let n = std::io::stdin()
        .read_line(&mut line)
        .map_err(|e| runtime_error(format!("stdin: {}", e)))?;
    if n == 0 { return Ok(None); }  // INPUT0002: EOF → None
    strip_newline(&mut line);
    Ok(Some(line))
}

fn read_with_prompt(args: &[Value16]) -> HudHudResult<Value16> {
    write_prompt(args);
    match read_line_eof()? {
        None => Ok(Value16::null()),
        Some(s) => Ok(Value16::string(s)),
    }
}

/// Zero-cost enum dispatch for stdin operations.
pub fn dispatch(method: StdinMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        StdinMethodId::Read => read_with_prompt(args),
        StdinMethodId::Line => {
            match read_line_eof()? {
                None => Ok(Value16::null()),
                Some(s) => Ok(Value16::string(s)),
            }
        }
        StdinMethodId::Confirm => {
            let prompt = args.first().and_then(|v| v.as_str()).unwrap_or("Confirm?");
            print!("{} [y/N] ", prompt);
            use std::io::Write;
            std::io::stdout().flush().ok();
            match read_line_eof()? {
                None => Ok(Value16::bool_(false)),
                Some(s) => {
                    let a = s.trim().to_lowercase();
                    Ok(Value16::bool_(matches!(a.as_str(), "y" | "yes" | "evet" | "e")))
                }
            }
        }
        StdinMethodId::Password => {
            // INPUT0003: Real hidden input via rpassword
            if let Some(prompt) = args.first().and_then(|v| v.as_str()) {
                print!("{}", prompt);
                use std::io::Write;
                std::io::stdout().flush().ok();
            }
            match rpassword::read_password() {
                Ok(pw) => Ok(Value16::string(pw)),
                Err(e) => {
                    // TTY yoksa (pipe/CI) — net hata, fallback YOK
                    Err(runtime_error(format!("stdin.password: no TTY available: {}", e)))
                }
            }
        }
        StdinMethodId::Number => {
            write_prompt(args);
            match read_line_eof()? {
                None => Ok(Value16::null()),
                Some(s) => s.trim().parse::<f64>()
                    .map(Value16::number)
                    .map_err(|_| runtime_error(format!("stdin.number: '{}' is not a number", s.trim()))),
            }
        }
        StdinMethodId::Int => {
            write_prompt(args);
            match read_line_eof()? {
                None => Ok(Value16::null()),
                Some(s) => s.trim().parse::<i64>()
                    .map(Value16::int)
                    .map_err(|_| runtime_error(format!("stdin.int: '{}' is not an integer", s.trim()))),
            }
        }
        StdinMethodId::All => {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)
                .map_err(|e| runtime_error(format!("stdin.all: {}", e)))?;
            Ok(Value16::string(buf))
        }
        StdinMethodId::Lines => {
            use std::io::BufRead;
            let lines: Vec<Value16> = std::io::stdin().lock().lines()
                .map_while(Result::ok)
                .map(Value16::string)
                .collect();
            Ok(Value16::array(lines))
        }
    }
}
