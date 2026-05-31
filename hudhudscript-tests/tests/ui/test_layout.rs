use hudhudscript_ui_core::layout::{Constraint, Direction, Layout};
use hudhudscript_ui_core::widget::Rect;

#[test]
fn test_empty_constraints() {
    let layout = Layout::new();
    let area = Rect::new(0, 0, 80, 24);
    let rects = layout.split(area);
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0], area);
}

#[test]
fn test_fixed_vertical() {
    let layout = Layout::new()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fixed(5), Constraint::Fixed(10)]);
    let area = Rect::new(0, 0, 80, 24);
    let rects = layout.split(area);
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0], Rect::new(0, 0, 80, 5));
    assert_eq!(rects[1], Rect::new(0, 5, 80, 10));
}

#[test]
fn test_fixed_horizontal() {
    let layout = Layout::new()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Fixed(20), Constraint::Fixed(30)]);
    let area = Rect::new(0, 0, 80, 24);
    let rects = layout.split(area);
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0], Rect::new(0, 0, 20, 24));
    assert_eq!(rects[1], Rect::new(20, 0, 30, 24));
}

#[test]
fn test_percentage() {
    let layout = Layout::new()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)]);
    let area = Rect::new(0, 0, 80, 24);
    let rects = layout.split(area);
    assert_eq!(rects[0].width, 40);
    assert_eq!(rects[1].width, 40);
}

#[test]
fn test_fill() {
    let layout = Layout::new()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Fixed(3),
            Constraint::Fill,
            Constraint::Fixed(1),
        ]);
    let area = Rect::new(0, 0, 80, 24);
    let rects = layout.split(area);
    assert_eq!(rects[0].height, 3);
    assert_eq!(rects[1].height, 20); // 24 - 3 - 1
    assert_eq!(rects[2].height, 1);
}

#[test]
fn test_min_constraint() {
    let layout = Layout::new()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(5), Constraint::Fixed(10)]);
    let area = Rect::new(0, 0, 80, 24);
    let rects = layout.split(area);
    // Min(5) gets 5 + remaining (24 - 5 - 10 = 9) = 14
    assert_eq!(rects[0].height, 14);
    assert_eq!(rects[1].height, 10);
}

#[test]
fn test_max_constraint() {
    let layout = Layout::new()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Max(10), Constraint::Fixed(5)]);
    let area = Rect::new(0, 0, 80, 24);
    let rects = layout.split(area);
    // Max(10) gets min(remaining=24, 10) = 10 initially, then no fill
    assert_eq!(rects[0].height, 10);
    assert_eq!(rects[1].height, 5);
}

#[test]
fn test_overflow_clamp() {
    let layout = Layout::new()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fixed(100), Constraint::Fixed(100)]);
    let area = Rect::new(0, 0, 80, 24);
    let rects = layout.split(area);
    assert_eq!(rects[0].height, 24);
    assert_eq!(rects[1].height, 0);
}

#[test]
fn test_layout_with_offset() {
    let layout = Layout::new()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Fixed(10), Constraint::Fill]);
    let area = Rect::new(5, 3, 40, 20);
    let rects = layout.split(area);
    assert_eq!(rects[0], Rect::new(5, 3, 10, 20));
    assert_eq!(rects[1], Rect::new(15, 3, 30, 20));
}

#[test]
fn test_multiple_fills() {
    let layout = Layout::new()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fill, Constraint::Fill, Constraint::Fill]);
    let area = Rect::new(0, 0, 80, 24);
    let rects = layout.split(area);
    // 24 / 3 = 8 each
    assert_eq!(rects[0].height, 8);
    assert_eq!(rects[1].height, 8);
    assert_eq!(rects[2].height, 8);
}

#[test]
fn test_multiple_fills_uneven() {
    let layout = Layout::new()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fill, Constraint::Fill]);
    let area = Rect::new(0, 0, 80, 25);
    let rects = layout.split(area);
    // 25 / 2 = 12 each, 1 extra goes to first
    assert_eq!(rects[0].height, 13);
    assert_eq!(rects[1].height, 12);
}

#[test]
fn test_layout_default() {
    let layout = Layout::default();
    assert_eq!(layout.get_direction(), Direction::Vertical);
    assert!(layout.get_constraints().is_empty());
}

#[test]
fn test_direction_default() {
    let d = Direction::default();
    assert_eq!(d, Direction::Vertical);
}

#[test]
fn test_percentage_over_100_clamped() {
    let layout = Layout::new()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(150)]);
    let area = Rect::new(0, 0, 80, 24);
    let rects = layout.split(area);
    // Percentage clamped to 100 → 80
    assert_eq!(rects[0].width, 80);
}

#[test]
fn test_min_with_fill_shares_leftover() {
    let layout = Layout::new()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Fixed(5),
            Constraint::Min(3),
            Constraint::Fill,
        ]);
    let area = Rect::new(0, 0, 80, 24);
    let rects = layout.split(area);
    assert_eq!(rects[0].height, 5);
    // Remaining = 24 - 5 = 19, Min(3) gets 3, then leftover=16 split between Min and Fill
    // Min gets 3 + 8 = 11, Fill gets 8
    let total_remaining: u16 = rects[1].height + rects[2].height;
    assert_eq!(total_remaining, 19);
}

#[test]
fn test_layout_serialization() {
    let layout = Layout::new()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Fixed(10), Constraint::Fill]);
    let json = serde_json::to_string(&layout).unwrap();
    assert!(json.contains("Horizontal"));
    assert!(json.contains("Fixed"));
}
