//! Real unit tests for hudhud-print — print capture mechanism

#[test]
fn start_capture_initializes_buffer() {
    hudhud_print::print_ops::start_capture();
    let output = hudhud_print::print_ops::stop_capture();
    assert!(output.is_some(), "capture should be active after start");
    assert_eq!(output.unwrap(), "", "fresh capture should be empty");
}

#[test]
fn stop_capture_returns_none_when_not_active() {
    let output = hudhud_print::print_ops::stop_capture();
    assert!(
        output.is_none(),
        "stop_capture without start should return None"
    );
}

#[test]
fn print_line_captures_output() {
    hudhud_print::print_ops::start_capture();
    hudhud_print::print_ops::print_line("hello");
    hudhud_print::print_ops::print_line("world");
    let output = hudhud_print::print_ops::stop_capture().unwrap();
    assert_eq!(output, "hello\nworld\n");
}

#[test]
fn print_line_empty_string_still_adds_newline() {
    hudhud_print::print_ops::start_capture();
    hudhud_print::print_ops::print_line("");
    let output = hudhud_print::print_ops::stop_capture().unwrap();
    assert_eq!(output, "\n");
}

#[test]
fn multiple_start_capture_clears_previous() {
    hudhud_print::print_ops::start_capture();
    hudhud_print::print_ops::print_line("first");
    // Start a new capture without stopping — replaces buffer
    hudhud_print::print_ops::start_capture();
    hudhud_print::print_ops::print_line("second");
    let output = hudhud_print::print_ops::stop_capture().unwrap();
    assert_eq!(output, "second\n");
}

#[test]
fn unicode_text_in_capture() {
    hudhud_print::print_ops::start_capture();
    hudhud_print::print_ops::print_line("Merhaba Dünya 🌍");
    hudhud_print::print_ops::print_line("こんにちは");
    let output = hudhud_print::print_ops::stop_capture().unwrap();
    assert!(output.contains("Merhaba Dünya 🌍"));
    assert!(output.contains("こんにちは"));
}
