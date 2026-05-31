//! TUI widget builtins — paragraph, block, list, gauge.
//! TUI.md: Command buffer pattern — widgets push to queue, tui.draw() drains.

use std::cell::RefCell;
use std::collections::HashMap;

use ratatui::{
    widgets::{Paragraph, Block, Borders, List, ListItem, Gauge},
    style::{Style, Color, Modifier},
    layout::Rect,
};
use hudhudscript_bytecode::Value16;
use hudhudscript_bytecode::error::compile_codes;

type CompileResult<T> = Result<T, hudhudscript_errors::Error>;

/// A render command queued by widget builtins, executed by tui.draw().
pub(crate) enum RenderCommand {
    Paragraph(String, Style, Rect),
    Block(String, Borders, Style, Style, Rect),
    List(Vec<String>, Style, Rect),
    Gauge(f64, String, Style, Rect),
}

thread_local! {
    /// Command queue — widgets push, tui.draw() drains inside term.draw().
    pub(crate) static COMMAND_QUEUE: RefCell<Vec<RenderCommand>> = RefCell::new(Vec::new());
}

pub(crate) fn drain_queue() -> Vec<RenderCommand> {
    COMMAND_QUEUE.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

fn parse_color(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "black" => Color::Black, "red" => Color::Red,
        "green" => Color::Green, "yellow" => Color::Yellow,
        "blue" => Color::Blue, "magenta" => Color::Magenta,
        "cyan" => Color::Cyan, "white" => Color::White,
        "gray" | "grey" => Color::Gray, "darkgray" => Color::DarkGray,
        "lightred" => Color::LightRed, "lightgreen" => Color::LightGreen,
        "lightblue" => Color::LightBlue, "lightyellow" => Color::LightYellow,
        "lightcyan" => Color::LightCyan, "lightmagenta" => Color::LightMagenta,
        _ => Color::Reset,
    }
}

fn parse_style(cfg: &HashMap<String, Value16>) -> Style {
    let mut style = Style::default();
    if let Some(fg) = cfg.get("fg").and_then(|v| v.as_string()) { style = style.fg(parse_color(&fg)); }
    if let Some(bg) = cfg.get("bg").and_then(|v| v.as_string()) { style = style.bg(parse_color(&bg)); }
    if cfg.get("bold").and_then(|v| v.as_bool()).unwrap_or(false) { style = style.add_modifier(Modifier::BOLD); }
    if cfg.get("italic").and_then(|v| v.as_bool()).unwrap_or(false) { style = style.add_modifier(Modifier::ITALIC); }
    if cfg.get("underline").and_then(|v| v.as_bool()).unwrap_or(false) { style = style.add_modifier(Modifier::UNDERLINED); }
    style
}

fn parse_area(cfg: &HashMap<String, Value16>) -> Rect {
    Rect {
        x: cfg.get("x").and_then(|v| v.as_int()).unwrap_or(0) as u16,
        y: cfg.get("y").and_then(|v| v.as_int()).unwrap_or(0) as u16,
        width: cfg.get("width").and_then(|v| v.as_int()).unwrap_or(0) as u16,
        height: cfg.get("height").and_then(|v| v.as_int()).unwrap_or(0) as u16,
    }
}

fn get_config(args: &[Value16]) -> CompileResult<HashMap<String, Value16>> {
    args.first()
        .and_then(|v| v.as_object())
        .cloned()
        .ok_or_else(|| compile_codes::runtime_error("expected object argument"))
}

/// Queue paragraph for rendering.
pub fn tui_paragraph(args: &[Value16]) -> CompileResult<Value16> {
    let cfg = get_config(args)?;
    let text = cfg.get("text").and_then(|v| v.as_string()).unwrap_or_default();
    let style = cfg.get("style").and_then(|v| v.as_object()).map(|o| parse_style(&o)).unwrap_or_default();
    let area = cfg.get("area").and_then(|v| v.as_object()).map(|a| parse_area(&a)).unwrap_or_default();
    COMMAND_QUEUE.with(|q| q.borrow_mut().push(RenderCommand::Paragraph(text, style, area)));
    Ok(Value16::null())
}

/// Queue block for rendering.
pub fn tui_block(args: &[Value16]) -> CompileResult<Value16> {
    let cfg = get_config(args)?;
    let title = cfg.get("title").and_then(|v| v.as_string()).unwrap_or_default();
    let borders = match cfg.get("borders").and_then(|v| v.as_string()).as_deref() {
        Some("all") => Borders::ALL, Some("top") => Borders::TOP,
        Some("bottom") => Borders::BOTTOM, Some("left") => Borders::LEFT,
        Some("right") => Borders::RIGHT, _ => Borders::ALL,
    };
    let border_style = cfg.get("border_style").and_then(|v| v.as_object()).map(|o| parse_style(&o)).unwrap_or_default();
    let style = cfg.get("style").and_then(|v| v.as_object()).map(|o| parse_style(&o)).unwrap_or_default();
    let area = cfg.get("area").and_then(|v| v.as_object()).map(|a| parse_area(&a)).unwrap_or_default();
    COMMAND_QUEUE.with(|q| q.borrow_mut().push(RenderCommand::Block(title, borders, border_style, style, area)));
    Ok(Value16::null())
}

/// Queue list for rendering.
pub fn tui_list(args: &[Value16]) -> CompileResult<Value16> {
    let cfg = get_config(args)?;
    let items: Vec<String> = cfg.get("items")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(|v| v.as_string().unwrap_or_default()).collect())
        .unwrap_or_default();
    let style = cfg.get("style").and_then(|v| v.as_object()).map(|o| parse_style(&o)).unwrap_or_default();
    let area = cfg.get("area").and_then(|v| v.as_object()).map(|a| parse_area(&a)).unwrap_or_default();
    COMMAND_QUEUE.with(|q| q.borrow_mut().push(RenderCommand::List(items, style, area)));
    Ok(Value16::null())
}

/// Queue gauge for rendering.
pub fn tui_gauge(args: &[Value16]) -> CompileResult<Value16> {
    let cfg = get_config(args)?;
    let value = cfg.get("value").and_then(|v| v.as_int()).unwrap_or(0) as u16;
    let max = cfg.get("max").and_then(|v| v.as_int()).unwrap_or(100) as u16;
    let label = cfg.get("label").and_then(|v| v.as_string()).unwrap_or_default();
    let style = cfg.get("style").and_then(|v| v.as_object()).map(|o| parse_style(&o)).unwrap_or_default();
    let area = cfg.get("area").and_then(|v| v.as_object()).map(|a| parse_area(&a)).unwrap_or_default();
    let ratio = if max > 0 { value as f64 / max as f64 } else { 0.0 };
    COMMAND_QUEUE.with(|q| q.borrow_mut().push(RenderCommand::Gauge(ratio, label, style, area)));
    Ok(Value16::null())
}

/// Execute all queued commands on a frame. Called from tui_draw inside term.draw().
pub fn execute_commands(frame: &mut ratatui::Frame) {
    for cmd in drain_queue() {
        match cmd {
            RenderCommand::Paragraph(text, style, area) => {
                frame.render_widget(Paragraph::new(text).style(style), area);
            }
            RenderCommand::Block(title, borders, border_style, style, area) => {
                let block = Block::default().title(title.as_str()).borders(borders)
                    .border_style(border_style).style(style);
                frame.render_widget(block, area);
            }
            RenderCommand::List(items, style, area) => {
                let list_items: Vec<ListItem> = items.iter().map(|i| ListItem::new(i.as_str())).collect();
                frame.render_widget(List::new(list_items).style(style), area);
            }
            RenderCommand::Gauge(ratio, label, style, area) => {
                frame.render_widget(Gauge::default().ratio(ratio).label(label).style(style), area);
            }
        }
    }
}
