//! TUI lifecycle and draw builtin implementations.
//! TUI.md: tui.draw() drains command queue inside term.draw().

use std::io::stdout;
use std::sync::Mutex;

use crossterm::{
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, size},
    cursor::{Hide, Show},
};
use ratatui::{Terminal, backend::CrosstermBackend, layout::{Layout, Constraint, Direction}};
use hudhudscript_bytecode::Value16;
use hudhudscript_bytecode::error::compile_codes;

use crate::widgets;

type CompileResult<T> = Result<T, hudhudscript_errors::Error>;

static TERMINAL: Mutex<Option<Terminal<CrosstermBackend<std::io::Stdout>>>> = Mutex::new(None);
static PANIC_HOOK_SET: std::sync::OnceLock<()> = std::sync::OnceLock::new();

fn install_panic_hook() {
    PANIC_HOOK_SET.get_or_init(|| {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = disable_raw_mode();
            let _ = execute!(stdout(), LeaveAlternateScreen, Show);
            hook(info);
        }));
    });
}

pub fn tui_init(_args: &[Value16]) -> CompileResult<Value16> {
    install_panic_hook();
    enable_raw_mode().map_err(|e| compile_codes::runtime_error(format!("raw mode: {}", e)))?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide)
        .map_err(|e| compile_codes::runtime_error(format!("alt screen: {}", e)))?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)
        .map_err(|e| compile_codes::runtime_error(format!("terminal: {}", e)))?;
    *TERMINAL.lock().unwrap() = Some(terminal);
    Ok(Value16::bool_(true))
}

pub fn tui_restore(_args: &[Value16]) -> CompileResult<Value16> {
    if let Some(_term) = TERMINAL.lock().unwrap().take() {
        let mut stdout = stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, Show);
        let _ = disable_raw_mode();
    }
    Ok(Value16::bool_(true))
}

pub fn tui_size(_args: &[Value16]) -> CompileResult<Value16> {
    let (w, h) = size().map_err(|e| compile_codes::runtime_error(format!("size: {}", e)))?;
    let mut obj = std::collections::HashMap::new();
    obj.insert("width".to_string(), Value16::int(w as i64));
    obj.insert("height".to_string(), Value16::int(h as i64));
    Ok(Value16::object(obj))
}

pub fn tui_clear(_args: &[Value16]) -> CompileResult<Value16> {
    // Clear by queuing a full-screen clear command
    let (w, h) = size().unwrap_or((80, 24));
    widgets::COMMAND_QUEUE.with(|q| {
        q.borrow_mut().push(widgets::RenderCommand::Block(
            String::new(), ratatui::widgets::Borders::NONE,
            ratatui::style::Style::default(), ratatui::style::Style::default(),
            ratatui::layout::Rect { x: 0, y: 0, width: w, height: h },
        ));
    });
    Ok(Value16::null())
}

/// TUI.md fix: drain command queue inside term.draw().
pub fn tui_draw(_args: &[Value16]) -> CompileResult<Value16> {
    let mut guard = TERMINAL.lock().unwrap();
    let term = guard.as_mut()
        .ok_or_else(|| compile_codes::runtime_error("tui.init() not called"))?;

    term.draw(|frame| {
        widgets::execute_commands(frame);
    }).map_err(|e| compile_codes::runtime_error(format!("draw: {}", e)))?;

    Ok(Value16::null())
}

pub fn tui_split(args: &[Value16]) -> CompileResult<Value16> {
    let cfg = args.first().and_then(|v| v.as_object())
        .ok_or_else(|| compile_codes::runtime_error("split: expected object"))?;

    let direction = match cfg.get("direction").and_then(|v| v.as_string()).as_deref() {
        Some("horizontal") => Direction::Horizontal,
        _ => Direction::Vertical,
    };

    let constraints: Vec<Constraint> = cfg.get("constraints")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|c| {
            let obj = c.as_object()?;
            let val = obj.get("value").and_then(|v| v.as_int())? as u16;
            match obj.get("type").and_then(|v| v.as_string())?.as_str() {
                "length" => Some(Constraint::Length(val)),
                "percentage" => Some(Constraint::Percentage(val)),
                "min" => Some(Constraint::Min(val)),
                "max" => Some(Constraint::Max(val)),
                "ratio" => Some(Constraint::Ratio(val as u32, 1)),
                _ => None,
            }
        }).collect())
        .unwrap_or_default();

    let area = cfg.get("area").and_then(|v| v.as_object())
        .map(|a| ratatui::layout::Rect {
            x: a.get("x").and_then(|v| v.as_int()).unwrap_or(0) as u16,
            y: a.get("y").and_then(|v| v.as_int()).unwrap_or(0) as u16,
            width: a.get("width").and_then(|v| v.as_int()).unwrap_or(0) as u16,
            height: a.get("height").and_then(|v| v.as_int()).unwrap_or(0) as u16,
        }).unwrap_or(ratatui::layout::Rect { x: 0, y: 0, width: 80, height: 24 });

    let chunks = Layout::default().direction(direction).constraints(constraints).split(area);
    let result: Vec<Value16> = chunks.iter().map(|r| {
        let mut o = std::collections::HashMap::new();
        o.insert("x".to_string(), Value16::int(r.x as i64));
        o.insert("y".to_string(), Value16::int(r.y as i64));
        o.insert("width".to_string(), Value16::int(r.width as i64));
        o.insert("height".to_string(), Value16::int(r.height as i64));
        Value16::object(o)
    }).collect();
    Ok(Value16::array(result))
}
