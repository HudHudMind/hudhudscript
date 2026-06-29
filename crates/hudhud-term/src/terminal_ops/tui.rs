use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::collections::HashMap;
use std::io::Write;
use super::{runtime_error, get_terminal_width, get_terminal_height, get_is_tty};

// ── TUI Operations (Issue #1023) ───────────────────────────────────────────

/// terminal.clear() — clear the entire screen and move cursor to top-left.
pub(crate) fn tui_clear() -> HudHudResult<Value16> {
    let mut stdout = std::io::stdout();
    // ESC[2J = clear entire screen, ESC[H = move cursor to 1,1
    write!(stdout, "\x1b[2J\x1b[H")
        .map_err(|e| runtime_error(format!("terminal.clear error: {}", e)))?;
    stdout
        .flush()
        .map_err(|e| runtime_error(format!("terminal.clear flush error: {}", e)))?;
    Ok(Value16::null())
}

/// terminal.move_cursor(row, col) — move cursor to 1-based position.
pub(crate) fn tui_move_cursor(args: &[Value16]) -> HudHudResult<Value16> {
    let row = args.first().and_then(|v| v.as_number()).ok_or_else(|| {
        runtime_error("terminal.move_cursor requires row (number) as first argument")
    })?;
    let col = args.get(1).and_then(|v| v.as_number()).ok_or_else(|| {
        runtime_error("terminal.move_cursor requires col (number) as second argument")
    })?;
    let row = row.max(1.0) as u32;
    let col = col.max(1.0) as u32;
    let mut stdout = std::io::stdout();
    // ESC[row;colH — ANSI CUP (Cursor Position), 1-based
    write!(stdout, "\x1b[{};{}H", row, col)
        .map_err(|e| runtime_error(format!("terminal.move_cursor error: {}", e)))?;
    stdout
        .flush()
        .map_err(|e| runtime_error(format!("terminal.move_cursor flush error: {}", e)))?;
    Ok(Value16::null())
}

/// terminal.write(text) — write text at the current cursor position (no newline).
pub(crate) fn tui_write(args: &[Value16]) -> HudHudResult<Value16> {
    let text = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("terminal.write requires a string argument"))?;
    let mut stdout = std::io::stdout();
    write!(stdout, "{}", text)
        .map_err(|e| runtime_error(format!("terminal.write error: {}", e)))?;
    stdout
        .flush()
        .map_err(|e| runtime_error(format!("terminal.write flush error: {}", e)))?;
    Ok(Value16::null())
}

/// terminal.read_key() — read a single keypress (blocking). Returns a string
/// describing the key: single chars as-is, special keys as names like "Enter",
/// "Escape", "ArrowUp", "ArrowDown", "ArrowRight", "ArrowLeft", "Backspace", etc.
pub(crate) fn tui_read_key() -> HudHudResult<Value16> {
    let key =
        read_single_key().map_err(|e| runtime_error(format!("terminal.read_key error: {}", e)))?;
    Ok(Value16::string(key))
}

/// terminal.size() — returns {rows, cols} object with terminal dimensions.
pub(crate) fn tui_size() -> HudHudResult<Value16> {
    let mut map = hudhudscript_bytecode::ObjMap::default();
    map.insert(
        "rows".to_string(),
        Value16::number(get_terminal_height() as f64),
    );
    map.insert(
        "cols".to_string(),
        Value16::number(get_terminal_width() as f64),
    );
    Ok(Value16::object(map))
}

/// terminal.set_color(fg, bg) — set foreground (and optional background) color.
/// Color names: "black", "red", "green", "yellow", "blue", "magenta", "cyan",
/// "white", "default". Pass null/empty for no change.
pub(crate) fn tui_set_color(args: &[Value16]) -> HudHudResult<Value16> {
    let fg = args.first().and_then(|v| v.as_str()).unwrap_or("");
    let bg = args.get(1).and_then(|v| v.as_str()).unwrap_or("");

    let mut seq = String::new();
    if let Some(code) = color_name_to_fg(fg) {
        seq.push_str(&format!("\x1b[{}m", code));
    }
    if let Some(code) = color_name_to_bg(bg) {
        seq.push_str(&format!("\x1b[{}m", code));
    }

    if !seq.is_empty() {
        let mut stdout = std::io::stdout();
        write!(stdout, "{}", seq)
            .map_err(|e| runtime_error(format!("terminal.set_color error: {}", e)))?;
        stdout
            .flush()
            .map_err(|e| runtime_error(format!("terminal.set_color flush error: {}", e)))?;
    }
    Ok(Value16::null())
}

/// Map color name to ANSI foreground code.
pub(crate) fn color_name_to_fg(name: &str) -> Option<u8> {
    match name {
        "black" => Some(30),
        "red" => Some(31),
        "green" => Some(32),
        "yellow" => Some(33),
        "blue" => Some(34),
        "magenta" => Some(35),
        "cyan" => Some(36),
        "white" => Some(37),
        "default" => Some(39),
        _ => None,
    }
}

/// Map color name to ANSI background code.
pub(crate) fn color_name_to_bg(name: &str) -> Option<u8> {
    match name {
        "black" => Some(40),
        "red" => Some(41),
        "green" => Some(42),
        "yellow" => Some(43),
        "blue" => Some(44),
        "magenta" => Some(45),
        "cyan" => Some(46),
        "white" => Some(47),
        "default" => Some(49),
        _ => None,
    }
}

// ── Raw key reading ────────────────────────────────────────────────────────

/// Read a single keypress in raw mode (Unix) or fall back to line-buffered read.
#[cfg(unix)]
pub(crate) fn read_single_key() -> Result<String, String> {
    use std::io::Read;
    use std::os::unix::io::AsRawFd;

    let stdin_fd = std::io::stdin().as_raw_fd();
    let mut old_termios: libc::termios = unsafe { std::mem::zeroed() };

    // Save current terminal attributes
    if unsafe { libc::tcgetattr(stdin_fd, &mut old_termios) } != 0 {
        return Err("failed to get terminal attributes".to_string());
    }

    // Set raw mode: disable canonical mode and echo
    let mut raw = old_termios;
    raw.c_lflag &= !(libc::ICANON | libc::ECHO);
    raw.c_cc[libc::VMIN] = 1; // minimum 1 byte
    raw.c_cc[libc::VTIME] = 0; // no timeout

    if unsafe { libc::tcsetattr(stdin_fd, libc::TCSANOW, &raw) } != 0 {
        return Err("failed to set raw mode".to_string());
    }

    let result = (|| {
        let mut buf = [0u8; 1];
        std::io::stdin()
            .read_exact(&mut buf)
            .map_err(|e| format!("read error: {}", e))?;

        let key = match buf[0] {
            0x1b => {
                // Escape sequence — try to read more
                let mut seq = [0u8; 2];
                // Set a short timeout for the rest of the escape sequence
                let mut peek_termios = raw;
                peek_termios.c_cc[libc::VMIN] = 0;
                peek_termios.c_cc[libc::VTIME] = 1; // 100ms timeout
                unsafe { libc::tcsetattr(stdin_fd, libc::TCSANOW, &peek_termios) };

                let n = std::io::stdin()
                    .read(&mut seq)
                    .map_err(|e| format!("read error: {}", e))?;

                if n == 0 {
                    "Escape".to_string()
                } else if n >= 2 && seq[0] == b'[' {
                    match seq[1] {
                        b'A' => "ArrowUp".to_string(),
                        b'B' => "ArrowDown".to_string(),
                        b'C' => "ArrowRight".to_string(),
                        b'D' => "ArrowLeft".to_string(),
                        b'H' => "Home".to_string(),
                        b'F' => "End".to_string(),
                        b'3' => {
                            // Could be Delete (ESC[3~) — consume the tilde
                            let mut tilde = [0u8; 1];
                            let _ = std::io::stdin().read(&mut tilde);
                            "Delete".to_string()
                        }
                        other => format!("Escape[{}", other as char),
                    }
                } else {
                    format!("Escape+{}", seq[0] as char)
                }
            }
            0x0d | 0x0a => "Enter".to_string(),
            0x09 => "Tab".to_string(),
            0x7f => "Backspace".to_string(),
            0x08 => "Backspace".to_string(),
            b if b < 0x20 => format!("Ctrl+{}", (b + 0x60) as char),
            b => (b as char).to_string(),
        };
        Ok(key)
    })();

    // Restore original terminal attributes
    unsafe { libc::tcsetattr(stdin_fd, libc::TCSANOW, &old_termios) };

    result
}

#[cfg(not(unix))]
pub(crate) fn read_single_key() -> Result<String, String> {
    // Fallback: read a line and return the first character
    use std::io::Read;
    let mut buf = [0u8; 1];
    std::io::stdin()
        .read_exact(&mut buf)
        .map_err(|e| format!("read error: {}", e))?;
    Ok((buf[0] as char).to_string())
}
