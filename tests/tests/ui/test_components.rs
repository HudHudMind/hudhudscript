use hudhudscript_ui_core::components::input::InputWidget;
use hudhudscript_ui_core::components::list::ListWidget;
use hudhudscript_ui_core::components::progress::ProgressWidget;
use hudhudscript_ui_core::components::table::TableWidget;
use hudhudscript_ui_core::components::text::TextWidget;
use hudhudscript_ui_core::{
    event::{Event, EventResult, KeyCode, KeyEvent},
    widget::{Alignment, Color, Rect, RenderCommand, TextStyle},
};

#[test]
fn test_text_widget_render() {
    let tw = TextWidget::new("Hello, World!");
    let cmds = tw.render(Rect::new(0, 0, 80, 1));
    assert_eq!(cmds.len(), 1);
    match &cmds[0] {
        RenderCommand::DrawText { text, x, y, .. } => {
            assert_eq!(text, "Hello, World!");
            assert_eq!(*x, 0);
            assert_eq!(*y, 0);
        }
        _ => panic!("Expected DrawText"),
    }
}

#[test]
fn test_text_widget_center() {
    let tw = TextWidget::new("Hi").alignment(Alignment::Center);
    let cmds = tw.render(Rect::new(0, 0, 10, 1));
    match &cmds[0] {
        RenderCommand::DrawText { x, .. } => assert_eq!(*x, 4), // (10-2)/2
        _ => panic!("Expected DrawText"),
    }
}

#[test]
fn test_text_widget_empty_area() {
    let tw = TextWidget::new("Hello");
    let cmds = tw.render(Rect::new(0, 0, 0, 0));
    assert!(cmds.is_empty());
}

#[test]
fn test_input_widget_typing() {
    let mut input = InputWidget::new();
    input.insert_char('H');
    input.insert_char('i');
    assert_eq!(input.value, "Hi");
    assert_eq!(input.cursor, 2);
}

#[test]
fn test_input_widget_backspace() {
    let mut input = InputWidget::new().value("Hello");
    input.delete_backward();
    assert_eq!(input.value, "Hell");
    assert_eq!(input.cursor, 4);
}

#[test]
fn test_input_widget_cursor_movement() {
    let mut input = InputWidget::new().value("ABC");
    assert_eq!(input.cursor, 3);
    input.move_left();
    assert_eq!(input.cursor, 2);
    input.move_left();
    assert_eq!(input.cursor, 1);
    input.move_right();
    assert_eq!(input.cursor, 2);
    input.move_home();
    assert_eq!(input.cursor, 0);
    input.move_end();
    assert_eq!(input.cursor, 3);
}

#[test]
fn test_input_widget_delete_forward() {
    let mut input = InputWidget::new().value("Hello");
    input.move_home();
    input.delete_forward();
    assert_eq!(input.value, "ello");
}

#[test]
fn test_input_event_handling() {
    let mut input = InputWidget::new();
    let result = input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Char('x'))));
    assert_eq!(result, EventResult::Consumed);
    assert_eq!(input.value, "x");

    let result = input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Enter)));
    assert_eq!(result, EventResult::Action("submit".to_string()));
}

#[test]
fn test_list_widget_navigation() {
    let mut list = ListWidget::new(vec!["One".into(), "Two".into(), "Three".into()]);
    assert_eq!(list.selected, Some(0));

    list.select_next();
    assert_eq!(list.selected, Some(1));

    list.select_next();
    assert_eq!(list.selected, Some(2));

    // At end, stay at last
    list.select_next();
    assert_eq!(list.selected, Some(2));

    list.select_previous();
    assert_eq!(list.selected, Some(1));
}

#[test]
fn test_list_widget_selected_item() {
    let list = ListWidget::new(vec!["Apple".into(), "Banana".into()]);
    assert_eq!(list.selected_item(), Some("Apple"));
}

#[test]
fn test_list_widget_empty() {
    let mut list = ListWidget::new(vec![]);
    assert_eq!(list.selected, None);
    list.select_next();
    assert_eq!(list.selected, None);
}

#[test]
fn test_list_event_handling() {
    let mut list = ListWidget::new(vec!["Item".into()]);
    let r = list.handle_event(&Event::Key(KeyEvent::new(KeyCode::Down)));
    assert_eq!(r, EventResult::Consumed);

    let r = list.handle_event(&Event::Key(KeyEvent::new(KeyCode::Enter)));
    assert_eq!(r, EventResult::Action("select:Item".to_string()));
}

#[test]
fn test_table_widget() {
    let table = TableWidget::new(vec!["Name".into(), "Age".into()], vec![20, 5]).rows(vec![
        vec!["Alice".into(), "30".into()],
        vec!["Bob".into(), "25".into()],
    ]);
    assert_eq!(table.selected_row, Some(0));
    assert_eq!(table.rows.len(), 2);
}

#[test]
fn test_table_navigation() {
    let mut table = TableWidget::new(vec!["A".into()], vec![10]).rows(vec![
        vec!["1".into()],
        vec!["2".into()],
        vec!["3".into()],
    ]);
    table.select_next();
    assert_eq!(table.selected_row, Some(1));
    table.select_previous();
    assert_eq!(table.selected_row, Some(0));
}

#[test]
fn test_table_render() {
    let table = TableWidget::new(vec!["Col".into()], vec![10]).rows(vec![vec!["Data".into()]]);
    let cmds = table.render(Rect::new(0, 0, 40, 10));
    assert!(!cmds.is_empty());
}

#[test]
fn test_progress_widget() {
    let pw = ProgressWidget::new(0.5);
    assert_eq!(pw.percentage(), 50);
}

#[test]
fn test_progress_clamp() {
    let pw = ProgressWidget::new(1.5);
    assert_eq!(pw.progress, 1.0);

    let pw2 = ProgressWidget::new(-0.5);
    assert_eq!(pw2.progress, 0.0);
}

#[test]
fn test_progress_render() {
    let pw = ProgressWidget::new(0.75).label("Loading");
    let cmds = pw.render(Rect::new(0, 0, 40, 2));
    assert!(cmds.len() >= 2); // label + bar (+ possibly percentage)
}

#[test]
fn test_progress_set() {
    let mut pw = ProgressWidget::new(0.0);
    pw.set_progress(0.42);
    assert_eq!(pw.percentage(), 42);
}

#[test]
fn test_text_widget_right_alignment() {
    let tw = TextWidget::new("Hi").alignment(Alignment::Right);
    let cmds = tw.render(Rect::new(0, 0, 10, 1));
    match &cmds[0] {
        RenderCommand::DrawText { x, .. } => assert_eq!(*x, 8), // 10-2
        _ => panic!("Expected DrawText"),
    }
}

#[test]
fn test_text_widget_multiline() {
    let tw = TextWidget::new("Line1\nLine2\nLine3");
    let cmds = tw.render(Rect::new(0, 0, 80, 2));
    // Only 2 lines fit (height=2)
    assert_eq!(cmds.len(), 2);
}

#[test]
fn test_text_widget_truncation() {
    let tw = TextWidget::new("Hello, World!");
    let cmds = tw.render(Rect::new(0, 0, 5, 1));
    match &cmds[0] {
        RenderCommand::DrawText { text, .. } => assert_eq!(text, "Hello"),
        _ => panic!("Expected DrawText"),
    }
}

#[test]
fn test_text_widget_with_style() {
    let style = TextStyle::default().fg(Color::Red).bold();
    let tw = TextWidget::new("Styled").style(style.clone());
    assert_eq!(tw.style, style);
}

#[test]
fn test_input_widget_placeholder() {
    let input = InputWidget::new().hint_text("Enter name");
    assert_eq!(input.hint_text, "Enter name");
}

#[test]
fn test_input_widget_render_with_placeholder() {
    let input = InputWidget::new().hint_text("Type here");
    let cmds = input.render(Rect::new(0, 0, 40, 1));
    // Should have at least a Clear and DrawText for the placeholder
    assert!(cmds.len() >= 2);
    // The DrawText should have dim style for placeholder
    let draw_text = cmds
        .iter()
        .find(|c| matches!(c, RenderCommand::DrawText { .. }));
    assert!(draw_text.is_some());
}

#[test]
fn test_input_widget_render_empty_area() {
    let input = InputWidget::new();
    let cmds = input.render(Rect::new(0, 0, 0, 0));
    assert!(cmds.is_empty());
}

#[test]
fn test_input_widget_event_non_key_ignored() {
    let mut input = InputWidget::new();
    let result = input.handle_event(&Event::Tick);
    assert_eq!(result, EventResult::Ignored);
}

#[test]
fn test_input_widget_backspace_at_start() {
    let mut input = InputWidget::new().value("Hi");
    input.move_home();
    input.delete_backward();
    // At position 0, backspace does nothing
    assert_eq!(input.value, "Hi");
    assert_eq!(input.cursor, 0);
}

#[test]
fn test_input_widget_delete_forward_at_end() {
    let mut input = InputWidget::new().value("Hi");
    input.delete_forward();
    // Cursor at end, delete forward does nothing
    assert_eq!(input.value, "Hi");
}

#[test]
fn test_input_widget_move_left_at_start() {
    let mut input = InputWidget::new().value("Hi");
    input.move_home();
    input.move_left();
    assert_eq!(input.cursor, 0);
}

#[test]
fn test_input_widget_move_right_at_end() {
    let mut input = InputWidget::new().value("Hi");
    input.move_right();
    assert_eq!(input.cursor, 2); // stays at end
}

#[test]
fn test_input_widget_event_keys() {
    let mut input = InputWidget::new();
    // Backspace
    input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Char('a'))));
    assert_eq!(input.value, "a");
    input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Backspace)));
    assert_eq!(input.value, "");

    // Delete
    input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Char('x'))));
    input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Home)));
    input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Delete)));
    assert_eq!(input.value, "");

    // Arrow keys
    input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Char('a'))));
    input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Char('b'))));
    let r = input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Left)));
    assert_eq!(r, EventResult::Consumed);
    let r = input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Right)));
    assert_eq!(r, EventResult::Consumed);
    let r = input.handle_event(&Event::Key(KeyEvent::new(KeyCode::End)));
    assert_eq!(r, EventResult::Consumed);

    // Unknown key
    let r = input.handle_event(&Event::Key(KeyEvent::new(KeyCode::F(1))));
    assert_eq!(r, EventResult::Ignored);
}

#[test]
fn test_input_widget_default() {
    let input = InputWidget::default();
    assert_eq!(input.value, "");
    assert_eq!(input.cursor, 0);
}

#[test]
fn test_list_widget_ensure_visible() {
    let mut list = ListWidget::new((0..20).map(|i| format!("Item {}", i)).collect());
    list.selected = Some(15);
    list.ensure_visible(5);
    // scroll_offset should be adjusted so item 15 is visible
    assert!(list.scroll_offset <= 15);
    assert!(list.scroll_offset + 5 > 15);
}

#[test]
fn test_list_widget_render_empty_area() {
    let list = ListWidget::new(vec!["a".into()]);
    let cmds = list.render(Rect::new(0, 0, 0, 0));
    assert!(cmds.is_empty());
}

#[test]
fn test_list_widget_event_non_key_ignored() {
    let mut list = ListWidget::new(vec!["a".into()]);
    let r = list.handle_event(&Event::Tick);
    assert_eq!(r, EventResult::Ignored);
}

#[test]
fn test_list_widget_event_unknown_key_ignored() {
    let mut list = ListWidget::new(vec!["a".into()]);
    let r = list.handle_event(&Event::Key(KeyEvent::new(KeyCode::Tab)));
    assert_eq!(r, EventResult::Ignored);
}

#[test]
fn test_list_widget_enter_empty_list() {
    let mut list = ListWidget::new(vec![]);
    let r = list.handle_event(&Event::Key(KeyEvent::new(KeyCode::Enter)));
    assert_eq!(r, EventResult::Ignored);
}

#[test]
fn test_list_widget_select_previous_at_start() {
    let mut list = ListWidget::new(vec!["a".into(), "b".into()]);
    list.select_previous();
    assert_eq!(list.selected, Some(0));
}

#[test]
fn test_table_widget_empty_render() {
    let table = TableWidget::new(vec!["Col".into()], vec![10]);
    let cmds = table.render(Rect::new(0, 0, 0, 0));
    assert!(cmds.is_empty());
}

#[test]
fn test_table_widget_event_handling() {
    let mut table =
        TableWidget::new(vec!["A".into()], vec![10]).rows(vec![vec!["1".into()], vec!["2".into()]]);

    let r = table.handle_event(&Event::Key(KeyEvent::new(KeyCode::Down)));
    assert_eq!(r, EventResult::Consumed);
    assert_eq!(table.selected_row, Some(1));

    let r = table.handle_event(&Event::Key(KeyEvent::new(KeyCode::Up)));
    assert_eq!(r, EventResult::Consumed);
    assert_eq!(table.selected_row, Some(0));

    let r = table.handle_event(&Event::Key(KeyEvent::new(KeyCode::Enter)));
    assert_eq!(r, EventResult::Action("select_row:0".to_string()));

    let r = table.handle_event(&Event::Key(KeyEvent::new(KeyCode::Tab)));
    assert_eq!(r, EventResult::Ignored);

    let r = table.handle_event(&Event::Tick);
    assert_eq!(r, EventResult::Ignored);
}

#[test]
fn test_table_select_empty() {
    let mut table = TableWidget::new(vec!["A".into()], vec![10]);
    table.select_next();
    assert_eq!(table.selected_row, None);
    table.select_previous();
    assert_eq!(table.selected_row, None);
}

#[test]
fn test_table_enter_no_selection() {
    let mut table = TableWidget::new(vec!["A".into()], vec![10]);
    let r = table.handle_event(&Event::Key(KeyEvent::new(KeyCode::Enter)));
    assert_eq!(r, EventResult::Ignored);
}

#[test]
fn test_table_select_boundary() {
    let mut table =
        TableWidget::new(vec!["A".into()], vec![10]).rows(vec![vec!["1".into()], vec!["2".into()]]);
    table.select_next(); // 1
    table.select_next(); // stays at 1 (last)
    assert_eq!(table.selected_row, Some(1));
    table.select_previous(); // 0
    table.select_previous(); // stays at 0
    assert_eq!(table.selected_row, Some(0));
}

#[test]
fn test_progress_widget_no_percentage() {
    let pw = ProgressWidget::new(0.5).show_percentage(false);
    let cmds = pw.render(Rect::new(0, 0, 40, 1));
    // Should still render the bar
    assert!(!cmds.is_empty());
}

#[test]
fn test_progress_widget_with_label_short_height() {
    let pw = ProgressWidget::new(0.5).label("Loading");
    // Height=1: label takes up the only line, bar doesn't have space
    let cmds = pw.render(Rect::new(0, 0, 40, 1));
    assert!(!cmds.is_empty());
}

#[test]
fn test_progress_widget_event_ignored() {
    let mut pw = ProgressWidget::new(0.5);
    let r = pw.handle_event(&Event::Tick);
    assert_eq!(r, EventResult::Ignored);
}

#[test]
fn test_progress_set_clamp() {
    let mut pw = ProgressWidget::new(0.0);
    pw.set_progress(2.0);
    assert_eq!(pw.progress, 1.0);
    pw.set_progress(-1.0);
    assert_eq!(pw.progress, 0.0);
}

#[test]
fn test_text_widget_event_ignored() {
    let mut tw = TextWidget::new("text");
    let r = tw.handle_event(&Event::Tick);
    assert_eq!(r, EventResult::Ignored);
}
