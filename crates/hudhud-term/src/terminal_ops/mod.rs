//! Shared Terminal/ANSI builtin — used by both VM and interpreter.
//!
//! Provides Terminal.bold(), dim(), italic(), underline(), strikethrough(),
//! red(), green(), yellow(), blue(), magenta(), cyan(), white(), gray(),
//! strip(), width(), height(), isatty().
//!
//! TUI operations (Issue #1023):
//! terminal.clear(), terminal.move_cursor(row, col), terminal.write(text),
//! terminal.read_key(), terminal.size(), terminal.set_color(fg, bg)

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}
use std::collections::HashMap;
use std::io::Write;

/// Execute a Terminal method on the given arguments.
/// Enum identifying each Terminal operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalMethodId {
    Clear,
    MoveCursor,
    Write,
    ReadKey,
    Size,
    SetColor,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Gray,
    Bold,
    Dim,
    Italic,
    Underline,
    Strikethrough,
    Strip,
    Width,
    Height,
    Isatty,
}

impl std::str::FromStr for TerminalMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "clear" => Ok(Self::Clear),
            "move_cursor" => Ok(Self::MoveCursor),
            "write" => Ok(Self::Write),
            "read_key" => Ok(Self::ReadKey),
            "size" => Ok(Self::Size),
            "set_color" => Ok(Self::SetColor),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "yellow" => Ok(Self::Yellow),
            "blue" => Ok(Self::Blue),
            "magenta" => Ok(Self::Magenta),
            "cyan" => Ok(Self::Cyan),
            "white" => Ok(Self::White),
            "gray" => Ok(Self::Gray),
            "bold" => Ok(Self::Bold),
            "dim" => Ok(Self::Dim),
            "italic" => Ok(Self::Italic),
            "underline" => Ok(Self::Underline),
            "strikethrough" => Ok(Self::Strikethrough),
            "strip" => Ok(Self::Strip),
            "width" => Ok(Self::Width),
            "height" => Ok(Self::Height),
            "isatty" => Ok(Self::Isatty),
            _ => Err(runtime_error(format!("Unknown Terminal method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch for Terminal operations.
pub fn dispatch(method: TerminalMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    // TUI methods that don't require a text argument
    match method {
        TerminalMethodId::Clear => return tui_clear(),
        TerminalMethodId::MoveCursor => return tui_move_cursor(args),
        TerminalMethodId::Write => return tui_write(args),
        TerminalMethodId::ReadKey => return tui_read_key(),
        TerminalMethodId::Size => return tui_size(),
        TerminalMethodId::SetColor => return tui_set_color(args),
        _ => {}
    }

    let text = match args.first().and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None if matches!(
            method,
            TerminalMethodId::Width | TerminalMethodId::Height | TerminalMethodId::Isatty
        ) =>
        {
            String::new()
        }
        _ => {
            return Err(runtime_error(
                format!("Terminal.{:?} requires a string argument", method).to_lowercase(),
            ));
        }
    };

    match method {
        TerminalMethodId::Red => Ok(wrap("31", &text)),
        TerminalMethodId::Green => Ok(wrap("32", &text)),
        TerminalMethodId::Yellow => Ok(wrap("33", &text)),
        TerminalMethodId::Blue => Ok(wrap("34", &text)),
        TerminalMethodId::Magenta => Ok(wrap("35", &text)),
        TerminalMethodId::Cyan => Ok(wrap("36", &text)),
        TerminalMethodId::White => Ok(wrap("37", &text)),
        TerminalMethodId::Gray => Ok(wrap("90", &text)),
        TerminalMethodId::Bold => Ok(wrap("1", &text)),
        TerminalMethodId::Dim => Ok(wrap("2", &text)),
        TerminalMethodId::Italic => Ok(wrap("3", &text)),
        TerminalMethodId::Underline => Ok(wrap("4", &text)),
        TerminalMethodId::Strikethrough => Ok(wrap("9", &text)),
        TerminalMethodId::Strip => {
            let mut result = String::new();
            let mut chars = text.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '\x1b' {
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next == 'm' {
                            break;
                        }
                    }
                } else {
                    result.push(ch);
                }
            }
            Ok(Value16::string(result))
        }
        TerminalMethodId::Width => Ok(Value16::number(get_terminal_width() as f64)),
        TerminalMethodId::Height => Ok(Value16::number(get_terminal_height() as f64)),
        TerminalMethodId::Isatty => Ok(Value16::bool_(get_is_tty())),
        _ => unreachable!(),
    }
}

/// Execute a Terminal method (kept for backward compat).

fn wrap(code: &str, text: &str) -> Value16 {
    Value16::string(format!("\x1b[{}m{}\x1b[0m", code, text))
}

#[cfg(unix)]
fn get_terminal_width() -> u16 {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            return ws.ws_col;
        }
    }
    80
}

#[cfg(not(unix))]
fn get_terminal_width() -> u16 {
    80
}

#[cfg(unix)]
fn get_terminal_height() -> u16 {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_row > 0 {
            return ws.ws_row;
        }
    }
    24
}

#[cfg(not(unix))]
fn get_terminal_height() -> u16 {
    24
}

#[cfg(unix)]
fn get_is_tty() -> bool {
    unsafe { libc::isatty(libc::STDOUT_FILENO) != 0 }
}

#[cfg(not(unix))]
fn get_is_tty() -> bool {
    false
}

mod tui;
use tui::*;
