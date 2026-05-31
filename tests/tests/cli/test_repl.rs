//! Tests for hudhudscript-cli repl module
//!
//! Covers: ReplConfig, Repl, LineBuffer, ReplAction, command registration, history

use hudhudscript_cli::repl::*;

// ─── ReplConfig defaults ────────────────────────────────────────────────────

#[test]
fn repl_config_defaults() {
    let cfg = ReplConfig::default();
    assert_eq!(cfg.prompt, "hud> ");
    assert!(cfg.history_file.is_none());
    assert_eq!(cfg.max_history, 1000);
}

#[test]
fn repl_config_builder() {
    let cfg = ReplConfig::new(">> ")
        .history_file("/tmp/hist")
        .max_history(500);

    assert_eq!(cfg.prompt, ">> ");
    assert_eq!(cfg.history_file.unwrap().to_str().unwrap(), "/tmp/hist");
    assert_eq!(cfg.max_history, 500);
}

// ─── LineBuffer basic operations ────────────────────────────────────────────

#[test]
fn linebuffer_insert_and_read() {
    let mut buf = LineBuffer::new();
    buf.insert('h');
    buf.insert('i');
    assert_eq!(buf.as_str(), "hi");
    assert_eq!(buf.cursor, 2);
}

#[test]
fn linebuffer_backspace() {
    let mut buf = LineBuffer::new();
    buf.insert('a');
    buf.insert('b');
    buf.insert('c');
    assert!(buf.backspace());
    assert_eq!(buf.as_str(), "ab");
    assert_eq!(buf.cursor, 2);

    // Backspace at empty returns false
    buf.clear();
    assert!(!buf.backspace());
}

#[test]
fn linebuffer_cursor_movement() {
    let mut buf = LineBuffer::new();
    buf.insert('x');
    buf.insert('y');
    buf.insert('z');
    // cursor at end (3)
    assert_eq!(buf.cursor, 3);

    // Move left twice
    assert!(buf.move_left());
    assert!(buf.move_left());
    assert_eq!(buf.cursor, 1);

    // Insert at cursor position
    buf.insert('W');
    assert_eq!(buf.as_str(), "xWyz");

    // Move right
    assert!(buf.move_right());
    assert_eq!(buf.cursor, 3); // past 'y'

    // Move left at start returns false
    buf.cursor = 0;
    assert!(!buf.move_left());

    // Move right at end returns false
    buf.cursor = buf.content.len();
    assert!(!buf.move_right());
}

#[test]
fn linebuffer_clear() {
    let mut buf = LineBuffer::new();
    buf.insert('a');
    buf.insert('b');
    buf.clear();
    assert_eq!(buf.as_str(), "");
    assert_eq!(buf.cursor, 0);
}

#[test]
fn linebuffer_default_trait() {
    let buf = LineBuffer::default();
    assert_eq!(buf.as_str(), "");
    assert_eq!(buf.cursor, 0);
}

// ─── Repl run_line ──────────────────────────────────────────────────────────

#[test]
fn repl_empty_line_returns_continue_none() {
    let mut repl = Repl::with_defaults();
    assert_eq!(repl.run_line(""), ReplAction::Continue(None));
    assert_eq!(repl.run_line("   "), ReplAction::Continue(None));
}

#[test]
fn repl_normal_line_returns_continue_with_code() {
    let mut repl = Repl::with_defaults();
    let action = repl.run_line("let x = 42");
    assert_eq!(action, ReplAction::Continue(Some("let x = 42".to_string())));
}

#[test]
fn repl_unknown_command_returns_error() {
    let mut repl = Repl::with_defaults();
    let action = repl.run_line(":nonexistent");
    match action {
        ReplAction::Error(msg) => assert!(msg.contains("nonexistent")),
        other => panic!("expected Error, got {:?}", other),
    }
}

#[test]
fn repl_registered_command_invoked() {
    let mut repl = Repl::with_defaults();
    repl.add_command("quit", "Exit the REPL", |_args| ReplAction::Exit);

    let action = repl.run_line(":quit");
    assert_eq!(action, ReplAction::Exit);
}

#[test]
fn repl_command_receives_arguments() {
    let mut repl = Repl::with_defaults();
    repl.add_command("echo", "Echo args", |args| {
        ReplAction::Continue(Some(args.join(" ")))
    });

    let action = repl.run_line(":echo hello world");
    assert_eq!(
        action,
        ReplAction::Continue(Some("hello world".to_string()))
    );
}

// ─── History ────────────────────────────────────────────────────────────────

#[test]
fn repl_history_records_lines() {
    let mut repl = Repl::with_defaults();
    repl.run_line("let a = 1");
    repl.run_line("let b = 2");
    assert_eq!(repl.history_len(), 2);
    assert_eq!(repl.history(), &["let a = 1", "let b = 2"]);
}

#[test]
fn repl_history_skips_consecutive_duplicates() {
    let mut repl = Repl::with_defaults();
    repl.run_line("let x = 1");
    repl.run_line("let x = 1");
    repl.run_line("let x = 1");
    assert_eq!(repl.history_len(), 1);
}

#[test]
fn repl_history_respects_max_history() {
    let config = ReplConfig::new(">> ").max_history(3);
    let mut repl = Repl::new(config);

    for i in 0..5 {
        repl.run_line(&format!("line {}", i));
    }
    assert_eq!(repl.history_len(), 3);
    // Oldest entries should have been evicted
    assert_eq!(repl.history()[0], "line 2");
    assert_eq!(repl.history()[2], "line 4");
}

// ─── list_commands ──────────────────────────────────────────────────────────

#[test]
fn repl_list_commands_sorted() {
    let mut repl = Repl::with_defaults();
    repl.add_command("quit", "Exit", |_| ReplAction::Exit);
    repl.add_command("help", "Show help", |_| ReplAction::Continue(None));
    repl.add_command("clear", "Clear screen", |_| ReplAction::Continue(None));

    let cmds = repl.list_commands();
    let names: Vec<&str> = cmds.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, vec!["clear", "help", "quit"]);
}

// ─── LineBuffer with multibyte characters ───────────────────────────────────

#[test]
fn linebuffer_multibyte_insert_and_navigate() {
    let mut buf = LineBuffer::new();
    buf.insert('m'); // 1 byte
    buf.insert('\u{00FC}'); // u-umlaut, 2 bytes
    buf.insert('\u{4E16}'); // CJK char, 3 bytes
    assert_eq!(buf.as_str(), "m\u{00FC}\u{4E16}");
    assert_eq!(buf.cursor, 6); // 1+2+3

    // Move left past CJK char
    assert!(buf.move_left());
    assert_eq!(buf.cursor, 3); // 1+2

    // Backspace removes umlaut
    assert!(buf.backspace());
    assert_eq!(buf.as_str(), "m\u{4E16}");
}
