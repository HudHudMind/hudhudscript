//! Tests for hudhudscript-ui-core: widget geometry, components, layout, state,
//! events, navigation, theme, serialization, and the app/widget-tree layer.

use std::collections::HashMap;

use hudhudscript_ui_core::components::input::InputWidget;
use hudhudscript_ui_core::components::list::ListWidget;
use hudhudscript_ui_core::components::progress::ProgressWidget;
use hudhudscript_ui_core::components::table::TableWidget;
use hudhudscript_ui_core::components::text::TextWidget;
use hudhudscript_ui_core::event::{
    Event, EventResult, KeyCode, KeyEvent, Modifiers, MouseButton, MouseEvent, MouseEventKind,
};
use hudhudscript_ui_core::layout::{Constraint, Direction, Layout};
use hudhudscript_ui_core::navigation::Router;
use hudhudscript_ui_core::state::{StateData, StateScope, StateStore};
use hudhudscript_ui_core::theme::{Theme, ThemeMode};
use hudhudscript_ui_core::widget::{Alignment, Margin, Rect, RenderCommand, TextStyle};
use hudhudscript_ui_core::{
    app::WidgetTree, BridgeError, BridgeMessage, Edges, PropValue, Style, UIEvent, UIUpdate,
    Widget, WidgetType,
};

// ── 1. Rect geometry ───────────────────────────────────────────────

#[test]
fn rect_basic_geometry() {
    let r = Rect::new(5, 10, 20, 15);
    assert_eq!(r.right(), 25);
    assert_eq!(r.bottom(), 25);
    assert_eq!(r.area(), 300);
    assert!(!r.is_empty());

    let empty = Rect::new(0, 0, 0, 10);
    assert!(empty.is_empty());
    assert_eq!(empty.area(), 0);
}

#[test]
fn rect_inner_with_margin() {
    let r = Rect::new(0, 0, 40, 20);
    let m = Margin::all(2);
    let inner = r.inner(&m);

    assert_eq!(inner.x, 2);
    assert_eq!(inner.y, 2);
    assert_eq!(inner.width, 36); // 40 - 2 - 2
    assert_eq!(inner.height, 16); // 20 - 2 - 2
}

// ── 2. Layout engine ───────────────────────────────────────────────

#[test]
fn layout_vertical_fixed_and_fill() {
    let layout = Layout::new()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Fixed(3),
            Constraint::Fill,
            Constraint::Fixed(1),
        ]);

    let area = Rect::new(0, 0, 80, 24);
    let rects = layout.split(area);

    assert_eq!(rects.len(), 3);
    // First: fixed 3 rows at top
    assert_eq!(rects[0], Rect::new(0, 0, 80, 3));
    // Last: fixed 1 row
    assert_eq!(rects[2], Rect::new(0, 23, 80, 1));
    // Middle: remaining 20 rows
    assert_eq!(rects[1], Rect::new(0, 3, 80, 20));
}

#[test]
fn layout_horizontal_percentage() {
    let layout = Layout::new()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)]);

    let area = Rect::new(0, 0, 100, 10);
    let rects = layout.split(area);

    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].width, 25);
    assert_eq!(rects[1].width, 75);
    assert_eq!(rects[1].x, 25);
}

// ── 3. InputWidget editing ─────────────────────────────────────────

#[test]
fn input_widget_typing_and_backspace() {
    let mut input = InputWidget::new();
    input.insert_char('H');
    input.insert_char('i');
    assert_eq!(input.value, "Hi");
    assert_eq!(input.cursor, 2);

    input.delete_backward();
    assert_eq!(input.value, "H");
    assert_eq!(input.cursor, 1);

    // Cursor movement
    input.insert_char('e');
    input.insert_char('y');
    // value = "Hey", cursor at 3
    input.move_home();
    assert_eq!(input.cursor, 0);
    input.move_end();
    assert_eq!(input.cursor, 3);
}

#[test]
fn input_widget_event_handling() {
    let mut input = InputWidget::new();
    let ev = Event::Key(KeyEvent::new(KeyCode::Char('a')));
    let result = input.handle_event(&ev);
    assert_eq!(result, EventResult::Consumed);
    assert_eq!(input.value, "a");

    let enter = Event::Key(KeyEvent::new(KeyCode::Enter));
    let result = input.handle_event(&enter);
    assert_eq!(result, EventResult::Action("submit".to_string()));

    // Mouse events are ignored by InputWidget
    let mouse = Event::Mouse(MouseEvent {
        kind: MouseEventKind::Click,
        x: 0,
        y: 0,
        button: MouseButton::Left,
    });
    assert_eq!(input.handle_event(&mouse), EventResult::Ignored);
}

// ── 4. ListWidget selection ────────────────────────────────────────

#[test]
fn list_widget_navigation() {
    let items = vec!["Alpha".into(), "Beta".into(), "Gamma".into()];
    let mut list = ListWidget::new(items);

    assert_eq!(list.selected, Some(0));
    assert_eq!(list.selected_item(), Some("Alpha"));

    list.select_next();
    assert_eq!(list.selected_item(), Some("Beta"));

    list.select_next();
    assert_eq!(list.selected_item(), Some("Gamma"));

    // Already at end -- stays at Gamma
    list.select_next();
    assert_eq!(list.selected_item(), Some("Gamma"));

    list.select_previous();
    assert_eq!(list.selected_item(), Some("Beta"));

    // Event: Enter produces action with selected item
    let enter = Event::Key(KeyEvent::new(KeyCode::Enter));
    let result = list.handle_event(&enter);
    assert_eq!(result, EventResult::Action("select:Beta".to_string()));
}

// ── 5. TableWidget ─────────────────────────────────────────────────

#[test]
fn table_widget_select_rows() {
    let headers = vec!["Name".into(), "Age".into()];
    let widths = vec![20, 5];
    let rows = vec![
        vec!["Alice".into(), "30".into()],
        vec!["Bob".into(), "25".into()],
    ];
    let mut table = TableWidget::new(headers, widths).rows(rows);

    assert_eq!(table.selected_row, Some(0));
    table.select_next();
    assert_eq!(table.selected_row, Some(1));
    // Past end stays put
    table.select_next();
    assert_eq!(table.selected_row, Some(1));

    let enter = Event::Key(KeyEvent::new(KeyCode::Enter));
    let result = table.handle_event(&enter);
    assert_eq!(result, EventResult::Action("select_row:1".to_string()));
}

// ── 6. ProgressWidget ──────────────────────────────────────────────

#[test]
fn progress_widget_clamping_and_percentage() {
    let mut p = ProgressWidget::new(0.5);
    assert_eq!(p.percentage(), 50);

    p.set_progress(1.5); // over 1.0 clamped
    assert_eq!(p.progress, 1.0);
    assert_eq!(p.percentage(), 100);

    p.set_progress(-0.3); // under 0.0 clamped
    assert_eq!(p.progress, 0.0);
    assert_eq!(p.percentage(), 0);
}

// ── 7. TextWidget rendering ────────────────────────────────────────

#[test]
fn text_widget_renders_draw_commands() {
    let tw = TextWidget::new("Hello\nWorld")
        .alignment(Alignment::Left)
        .style(TextStyle::default().bold());

    let area = Rect::new(0, 0, 20, 10);
    let cmds = tw.render(area);

    assert_eq!(cmds.len(), 2); // two lines
    match &cmds[0] {
        RenderCommand::DrawText { x, y, text, style } => {
            assert_eq!(*x, 0);
            assert_eq!(*y, 0);
            assert_eq!(text, "Hello");
            assert!(style.bold);
        }
        other => panic!("Expected DrawText, got {:?}", other),
    }
}

// ── 8. StateStore reactive bindings ────────────────────────────────

#[test]
fn state_store_define_set_bind() {
    let mut store = StateStore::new();
    store.define("count".into(), StateData::Number(0.0), StateScope::Screen);
    store.bind("count".into(), "label_1".into(), "text".into());
    store.bind("count".into(), "label_2".into(), "text".into());

    assert_eq!(store.get("count"), Some(&StateData::Number(0.0)));

    // Setting to same value returns empty (no change)
    let affected = store.set("count", StateData::Number(0.0));
    assert!(affected.is_empty());

    // Setting to new value returns bound widget IDs
    let affected = store.set("count", StateData::Number(1.0));
    assert_eq!(affected.len(), 2);
    assert!(affected.contains(&"label_1".to_string()));
    assert!(affected.contains(&"label_2".to_string()));

    // drain_dirty clears the flag
    let dirty = store.drain_dirty();
    assert_eq!(dirty, vec!["count".to_string()]);
    let dirty2 = store.drain_dirty();
    assert!(dirty2.is_empty());
}

// ── 9. Navigation / Router ─────────────────────────────────────────

#[test]
fn router_navigate_and_history() {
    let screens = vec!["home".into(), "settings".into(), "profile".into()];
    let mut router = Router::new(screens, "home");

    assert_eq!(router.current().screen, "home");
    assert!(!router.can_go_back());
    assert_eq!(router.history_len(), 1);

    // Navigate to settings
    router.navigate("settings".into(), HashMap::new()).unwrap();
    assert_eq!(router.current().screen, "settings");
    assert!(router.can_go_back());

    // Navigate to unknown screen fails
    let err = router.navigate("nonexistent".into(), HashMap::new());
    assert!(err.is_err());

    // Go back
    router.back().unwrap();
    assert_eq!(router.current().screen, "home");
    assert!(router.can_go_forward());

    // Go forward
    router.forward().unwrap();
    assert_eq!(router.current().screen, "settings");

    // Deep links
    router.add_deep_link("/settings".into(), "settings".into());
    assert_eq!(
        router.resolve_deep_link("/settings"),
        Some(&"settings".to_string())
    );
    assert_eq!(router.resolve_deep_link("/unknown"), None);
}

// ── 10. Theme system ───────────────────────────────────────────────

#[test]
fn theme_active_colors_by_mode() {
    let mut theme = Theme::default();
    assert_eq!(theme.mode, ThemeMode::Light);
    assert_eq!(theme.active_colors().background, "#FFFFFF");

    theme.mode = ThemeMode::Dark;
    assert_eq!(theme.active_colors().background, "#121212");

    // Typography defaults
    assert_eq!(theme.typography.font_size_md, 14.0);
    assert_eq!(theme.spacing.md, 16.0);
    assert_eq!(theme.border_radius.full, 9999.0);
}

// ── 11. Serde round-trip ───────────────────────────────────────────

#[test]
fn widget_tree_serde_roundtrip() {
    let mut props = HashMap::new();
    props.insert("label".to_string(), PropValue::String("Click me".into()));
    props.insert("disabled".to_string(), PropValue::Boolean(false));
    props.insert("count".to_string(), PropValue::Number(42.0));

    let widget = Widget {
        id: "btn1".into(),
        widget_type: WidgetType::Button,
        props,
        events: {
            let mut m = HashMap::new();
            m.insert("onClick".into(), "handleClick".into());
            m
        },
        children: vec![],
        style: Style {
            color: Some("#FF0000".into()),
            bold: Some(true),
            ..Style::default()
        },
    };

    let json = serde_json::to_string(&widget).expect("serialize");
    let deser: Widget = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deser.id, "btn1");
    match &deser.widget_type {
        WidgetType::Button => {}
        other => panic!("Expected Button, got {:?}", other),
    }
    assert_eq!(
        deser.events.get("onClick").map(|s| s.as_str()),
        Some("handleClick")
    );
}

#[test]
fn bridge_message_serde_roundtrip() {
    let event = BridgeMessage::Event(UIEvent::Click {
        widget_id: "btn_ok".into(),
    });
    let json = serde_json::to_string(&event).unwrap();
    let deser: BridgeMessage = serde_json::from_str(&json).unwrap();
    match deser {
        BridgeMessage::Event(UIEvent::Click { widget_id }) => {
            assert_eq!(widget_id, "btn_ok");
        }
        other => panic!("Expected Event(Click), got {:?}", other),
    }

    // Also test UIUpdate variant
    let update = BridgeMessage::Update(UIUpdate::Remove {
        id: "old_widget".into(),
    });
    let json2 = serde_json::to_string(&update).unwrap();
    let deser2: BridgeMessage = serde_json::from_str(&json2).unwrap();
    match deser2 {
        BridgeMessage::Update(UIUpdate::Remove { id }) => assert_eq!(id, "old_widget"),
        other => panic!("Expected Update(Remove), got {:?}", other),
    }
}

// ── 12. WidgetTree (app module) ────────────────────────────────────

#[test]
fn widget_tree_composition_and_find() {
    let root = WidgetTree::new("root")
        .with_commands(vec![RenderCommand::Clear {
            rect: Rect::new(0, 0, 80, 24),
        }])
        .add_child(
            WidgetTree::new("header").with_commands(vec![RenderCommand::DrawText {
                x: 0,
                y: 0,
                text: "Title".into(),
                style: TextStyle::default(),
            }]),
        )
        .add_child(WidgetTree::new("body").add_child(WidgetTree::new("list")));

    assert_eq!(root.node_count(), 4); // root + header + body + list

    // find by id
    assert!(root.find("header").is_some());
    assert!(root.find("list").is_some());
    assert!(root.find("nonexistent").is_none());

    // flatten collects all commands depth-first
    let all = root.flatten();
    assert_eq!(all.len(), 2); // Clear + DrawText
}

// ── 13. Modifiers bitflags ─────────────────────────────────────────

#[test]
fn modifier_flags_work_correctly() {
    let ctrl_shift = Modifiers::from_bits(Modifiers::CTRL | Modifiers::SHIFT);
    assert!(ctrl_shift.contains(Modifiers::CTRL));
    assert!(ctrl_shift.contains(Modifiers::SHIFT));
    assert!(!ctrl_shift.contains(Modifiers::ALT));
    assert!(!ctrl_shift.is_empty());

    let empty = Modifiers::empty();
    assert!(empty.is_empty());
}

// ── 14. Edges helper ───────────────────────────────────────────────

#[test]
fn edges_all_sets_uniform_value() {
    let e = Edges::all(16.0);
    assert_eq!(e.top, Some(16.0));
    assert_eq!(e.right, Some(16.0));
    assert_eq!(e.bottom, Some(16.0));
    assert_eq!(e.left, Some(16.0));

    let d = Edges::default();
    assert!(d.top.is_none());
}

// ── 15. BridgeError display ────────────────────────────────────────

#[test]
fn bridge_error_display_messages() {
    let e1 = BridgeError::InitFailed("no display".into());
    assert_eq!(format!("{}", e1), "Bridge init failed: no display");

    let e2 = BridgeError::ConnectionLost("timeout".into());
    assert!(format!("{}", e2).contains("timeout"));

    // Verify it implements std::error::Error
    let _: &dyn std::error::Error = &e1;
}
