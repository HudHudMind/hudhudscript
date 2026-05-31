//! Shared print capture mechanism for HudHudScript
//!
//! Provides a thread-local capture buffer so that both the interpreter and VM
//! can route print output through the same mechanism, enabling test capture.

use std::cell::RefCell;

// Thread-local buffer for capturing print output (used by tests and Python bindings)
thread_local! {
    static PRINT_CAPTURE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Enable print capturing for the current thread.
/// All subsequent `print_line` calls will append to the internal buffer instead of stdout.
pub fn start_capture() {
    PRINT_CAPTURE.with(|c| *c.borrow_mut() = Some(String::new()));
}

/// Disable print capturing and return the captured output.
/// Returns `None` if capturing was not active.
pub fn stop_capture() -> Option<String> {
    PRINT_CAPTURE.with(|c| c.borrow_mut().take())
}

/// Write a line to the capture buffer if active, otherwise to stdout.
pub fn print_line(line: &str) {
    let captured = PRINT_CAPTURE.with(|c| {
        let mut borrow = c.borrow_mut();
        if let Some(ref mut buf) = *borrow {
            buf.push_str(line);
            buf.push('\n');
            true
        } else {
            false
        }
    });
    if !captured {
        println!("{}", line);
    }
}

/// Write to stdout WITHOUT newline (capture-aware).
pub fn print_str(text: &str) {
    let captured = PRINT_CAPTURE.with(|c| {
        let mut borrow = c.borrow_mut();
        if let Some(ref mut buf) = *borrow {
            buf.push_str(text);
            true
        } else {
            false
        }
    });
    if !captured {
        print!("{}", text);
        use std::io::Write;
        std::io::stdout().flush().ok();
    }
}

/// Write to stderr (no newline). Not captured — always goes to stderr.
pub fn eprint_str(line: &str) {
    eprint!("{}", line);
}

/// Write to stderr with newline. Not captured — always goes to stderr.
pub fn eprintln_str(line: &str) {
    eprintln!("{}", line);
}
