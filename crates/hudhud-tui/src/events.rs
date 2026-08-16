//! TUI event handling builtin — keyboard + mouse + resize polling.

use std::collections::HashMap;
use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use hudhudscript_bytecode::error::compile_codes;
use hudhudscript_bytecode::Value16;

type CompileResult<T> = Result<T, hudhudscript_errors::Error>;

fn key_code_to_string(code: KeyCode) -> String {
    match code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "enter".into(),
        KeyCode::Esc => "esc".into(),
        KeyCode::Backspace => "backspace".into(),
        KeyCode::Tab => "tab".into(),
        KeyCode::Up => "up".into(),
        KeyCode::Down => "down".into(),
        KeyCode::Left => "left".into(),
        KeyCode::Right => "right".into(),
        KeyCode::Home => "home".into(),
        KeyCode::End => "end".into(),
        KeyCode::PageUp => "pageup".into(),
        KeyCode::PageDown => "pagedown".into(),
        KeyCode::Delete => "delete".into(),
        KeyCode::Insert => "insert".into(),
        KeyCode::F(n) => format!("f{}", n),
        _ => "unknown".into(),
    }
}

/// TUI0005: poll for terminal events with timeout (ms).
/// Returns null on timeout, or an object with { type, code, ctrl, alt, shift } for key events,
/// { type, kind, x, y } for mouse, { type, width, height } for resize.
pub fn tui_poll_event(args: &[Value16]) -> CompileResult<Value16> {
    let timeout_ms = args.first().and_then(|v| v.as_int()).unwrap_or(0) as u64;
    let timeout = Duration::from_millis(timeout_ms);

    if !poll(timeout).map_err(|e| compile_codes::runtime_error(format!("poll: {}", e)))? {
        return Ok(Value16::null());
    }

    let event = read().map_err(|e| compile_codes::runtime_error(format!("read: {}", e)))?;
    let mut obj = hudhudscript_bytecode::ObjMap::default();

    match event {
        Event::Key(KeyEvent {
            code, modifiers, ..
        }) => {
            obj.insert("type".to_string(), Value16::string("key".to_string()));
            obj.insert(
                "code".to_string(),
                Value16::string(key_code_to_string(code)),
            );
            obj.insert(
                "ctrl".to_string(),
                Value16::bool_(modifiers.contains(KeyModifiers::CONTROL)),
            );
            obj.insert(
                "alt".to_string(),
                Value16::bool_(modifiers.contains(KeyModifiers::ALT)),
            );
            obj.insert(
                "shift".to_string(),
                Value16::bool_(modifiers.contains(KeyModifiers::SHIFT)),
            );
        }
        Event::Mouse(MouseEvent {
            kind, column, row, ..
        }) => {
            obj.insert("type".to_string(), Value16::string("mouse".to_string()));
            obj.insert(
                "kind".to_string(),
                Value16::string(format!("{:?}", kind).to_lowercase()),
            );
            obj.insert("x".to_string(), Value16::int(column as i64));
            obj.insert("y".to_string(), Value16::int(row as i64));
        }
        Event::Resize(w, h) => {
            obj.insert("type".to_string(), Value16::string("resize".to_string()));
            obj.insert("width".to_string(), Value16::int(w as i64));
            obj.insert("height".to_string(), Value16::int(h as i64));
        }
        _ => return Ok(Value16::null()),
    }
    Ok(Value16::object(obj))
}
