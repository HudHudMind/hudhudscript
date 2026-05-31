use hudhudscript_ui_core::widget::{Alignment, Color, Margin, Rect, RenderCommand, TextStyle};

#[test]
fn test_rect_basics() {
    let r = Rect::new(5, 10, 20, 15);
    assert_eq!(r.right(), 25);
    assert_eq!(r.bottom(), 25);
    assert_eq!(r.area(), 300);
    assert!(!r.is_empty());
}

#[test]
fn test_rect_empty() {
    let r = Rect::new(0, 0, 0, 5);
    assert!(r.is_empty());
}

#[test]
fn test_rect_inner() {
    let r = Rect::new(0, 0, 20, 10);
    let m = Margin::all(1);
    let inner = r.inner(&m);
    assert_eq!(inner, Rect::new(1, 1, 18, 8));
}

#[test]
fn test_margin_symmetric() {
    let m = Margin::symmetric(2, 3);
    assert_eq!(m.top, 2);
    assert_eq!(m.bottom, 2);
    assert_eq!(m.left, 3);
    assert_eq!(m.right, 3);
}

#[test]
fn test_text_style_builder() {
    let style = TextStyle::default()
        .fg(Color::Red)
        .bg(Color::Black)
        .bold()
        .underline();
    assert_eq!(style.fg, Some(Color::Red));
    assert_eq!(style.bg, Some(Color::Black));
    assert!(style.bold);
    assert!(style.underline);
    assert!(!style.italic);
}

#[test]
fn test_render_command_serialization() {
    let cmd = RenderCommand::DrawText {
        x: 0,
        y: 0,
        text: "Hello".to_string(),
        style: TextStyle::default(),
    };
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("Hello"));
}

#[test]
fn test_rect_saturating() {
    let r = Rect::new(0, 0, 5, 5);
    let m = Margin::all(10); // bigger than rect
    let inner = r.inner(&m);
    assert_eq!(inner.width, 0);
    assert_eq!(inner.height, 0);
}

#[test]
fn test_text_style_italic() {
    let style = TextStyle::default().italic();
    assert!(style.italic);
    assert!(!style.bold);
    assert!(!style.underline);
}

#[test]
fn test_margin_new() {
    let m = Margin::new(1, 2, 3, 4);
    assert_eq!(m.top, 1);
    assert_eq!(m.right, 2);
    assert_eq!(m.bottom, 3);
    assert_eq!(m.left, 4);
}

#[test]
fn test_rect_default() {
    let r = Rect::default();
    assert_eq!(r.x, 0);
    assert_eq!(r.y, 0);
    assert_eq!(r.width, 0);
    assert_eq!(r.height, 0);
    assert!(r.is_empty());
    assert_eq!(r.area(), 0);
}

#[test]
fn test_color_serialization() {
    let indexed = Color::Indexed(42);
    let json = serde_json::to_string(&indexed).unwrap();
    assert!(json.contains("42"));

    let rgb = Color::Rgb(255, 128, 0);
    let json2 = serde_json::to_string(&rgb).unwrap();
    assert!(json2.contains("255"));
}

#[test]
fn test_render_command_clear_serialization() {
    let cmd = RenderCommand::Clear {
        rect: Rect::new(0, 0, 10, 5),
    };
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("Clear"));
}

#[test]
fn test_render_command_fill_serialization() {
    let cmd = RenderCommand::Fill {
        rect: Rect::new(0, 0, 5, 3),
        ch: '#',
        style: TextStyle::default(),
    };
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("Fill"));
}

#[test]
fn test_alignment_default() {
    let a = Alignment::default();
    assert_eq!(a, Alignment::Left);
}

#[test]
fn test_rect_inner_asymmetric_margin() {
    let r = Rect::new(10, 10, 30, 20);
    let m = Margin::new(2, 3, 4, 5);
    let inner = r.inner(&m);
    assert_eq!(inner.x, 15); // 10 + 5
    assert_eq!(inner.y, 12); // 10 + 2
    assert_eq!(inner.width, 22); // 30 - 5 - 3
    assert_eq!(inner.height, 14); // 20 - 2 - 4
}
