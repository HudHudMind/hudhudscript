//! Public API tests for hudhudscript-ui-core

use hudhudscript_ui_core::{
    event::{Event, EventResult, KeyCode, KeyEvent, Modifiers},
    layout::{Constraint, Direction, Layout},
    navigation::{NavigationError, Router},
    state::{StateData, StateScope, StateStore},
    theme::{Theme, ThemeMode},
    widget::{Alignment, Color, Margin, Rect, TextStyle},
    App, BridgeError, BridgeMessage, ComponentDef, Edges, PropValue, Screen, Style, UIEvent,
    UIUpdate, Widget, WidgetType,
};
use std::collections::HashMap;

// ── Widget / WidgetType ─────────────────────────────────────────────

#[test]
fn test_widget_serde() {
    let w = Widget {
        id: "b1".into(),
        widget_type: WidgetType::Button,
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![],
        style: Style::default(),
    };
    let j = serde_json::to_string(&w).unwrap();
    assert!(j.contains("b1"));
}
#[test]
fn test_widget_type_platform() {
    let j = serde_json::to_string(&WidgetType::Platform("gtk".into())).unwrap();
    assert!(j.contains("gtk"));
}
#[test]
fn test_widget_type_component() {
    let j = serde_json::to_string(&WidgetType::Component("My".into())).unwrap();
    assert!(j.contains("My"));
}

// ── PropValue ───────────────────────────────────────────────────────

#[test]
fn test_prop_value_variants() {
    let _ = serde_json::to_string(&PropValue::Array(vec![
        PropValue::Number(1.0),
        PropValue::Boolean(true),
        PropValue::Null,
    ]))
    .unwrap();
}
#[test]
fn test_prop_value_object() {
    let mut m = HashMap::new();
    m.insert("k".into(), PropValue::String("v".into()));
    let j = serde_json::to_string(&PropValue::Object(m)).unwrap();
    assert!(j.contains("k"));
}

// ── Style ───────────────────────────────────────────────────────────

#[test]
fn test_style_default() {
    let s = Style::default();
    assert!(s.size.is_none());
    assert!(s.color.is_none());
    assert!(s.display.is_none());
    assert!(s.padding.is_none());
    assert!(s.opacity.is_none());
}

// ── Edges ───────────────────────────────────────────────────────────

#[test]
fn test_edges_all() {
    let e = Edges::all(10.0);
    assert_eq!(e.top, Some(10.0));
    assert_eq!(e.right, Some(10.0));
}
#[test]
fn test_edges_default() {
    let e = Edges::default();
    assert_eq!(e.top, None);
}

// ── App / Screen / ComponentDef ─────────────────────────────────────

#[test]
fn test_app_structure() {
    let a = App {
        name: "A".into(),
        entry_screen: "Main".into(),
        screens: vec![],
        components: vec![],
    };
    assert_eq!(a.entry_screen, "Main");
}
#[test]
fn test_screen_serde() {
    let s = Screen {
        name: "H".into(),
        params: vec![],
        root: Widget {
            id: "r".into(),
            widget_type: WidgetType::Screen,
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![],
            style: Style::default(),
        },
    };
    assert!(serde_json::to_string(&s).unwrap().contains("H"));
}
#[test]
fn test_component_def_serde() {
    let c = ComponentDef {
        name: "B".into(),
        props: HashMap::new(),
        template: Widget {
            id: "r".into(),
            widget_type: WidgetType::Button,
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![],
            style: Style::default(),
        },
        on_mount: Some("m".into()),
        on_update: None,
        on_unmount: None,
    };
    assert!(serde_json::to_string(&c).unwrap().contains("B"));
}

// ── UIEvent ─────────────────────────────────────────────────────────

#[test]
fn test_ui_event_click() {
    assert!(serde_json::to_string(&UIEvent::Click {
        widget_id: "b1".into()
    })
    .unwrap()
    .contains("b1"));
}
#[test]
fn test_ui_event_input() {
    assert!(serde_json::to_string(&UIEvent::Input {
        widget_id: "i1".into(),
        value: "hi".into()
    })
    .unwrap()
    .contains("hi"));
}
#[test]
fn test_ui_event_submit() {
    assert!(serde_json::to_string(&UIEvent::Submit {
        widget_id: "f1".into()
    })
    .unwrap()
    .contains("f1"));
}
#[test]
fn test_ui_event_navigate() {
    assert!(serde_json::to_string(&UIEvent::Navigate {
        screen: "S".into(),
        params: HashMap::new()
    })
    .unwrap()
    .contains("S"));
}
#[test]
fn test_ui_event_select() {
    assert!(serde_json::to_string(&UIEvent::Select {
        widget_id: "s".into(),
        value: "v".into()
    })
    .unwrap()
    .contains("v"));
}
#[test]
fn test_ui_event_custom() {
    assert!(serde_json::to_string(&UIEvent::Custom {
        name: "my".into(),
        data: HashMap::new()
    })
    .unwrap()
    .contains("my"));
}

// ── UIUpdate ────────────────────────────────────────────────────────

#[test]
fn test_ui_update_remove() {
    assert!(serde_json::to_string(&UIUpdate::Remove { id: "w1".into() })
        .unwrap()
        .contains("w1"));
}
#[test]
fn test_ui_update_navigate() {
    assert!(serde_json::to_string(&UIUpdate::Navigate {
        screen: "S".into(),
        params: HashMap::new()
    })
    .unwrap()
    .contains("S"));
}

// ── BridgeMessage ───────────────────────────────────────────────────

#[test]
fn test_bridge_msg_shutdown() {
    assert!(serde_json::to_string(&BridgeMessage::Shutdown)
        .unwrap()
        .contains("Shutdown"));
}
#[test]
fn test_bridge_msg_ready() {
    assert!(serde_json::to_string(&BridgeMessage::Ready)
        .unwrap()
        .contains("Ready"));
}
#[test]
fn test_bridge_msg_error() {
    assert!(serde_json::to_string(&BridgeMessage::Error("oops".into()))
        .unwrap()
        .contains("oops"));
}

// ── BridgeError ─────────────────────────────────────────────────────

#[test]
fn test_bridge_error_display() {
    assert_eq!(
        format!("{}", BridgeError::InitFailed("o".into())),
        "Bridge init failed: o"
    );
    assert_eq!(
        format!("{}", BridgeError::RenderFailed("r".into())),
        "Render failed: r"
    );
    assert_eq!(
        format!("{}", BridgeError::ConnectionLost("c".into())),
        "Connection lost: c"
    );
    assert_eq!(
        format!("{}", BridgeError::FrameworkError("f".into())),
        "Framework error: f"
    );
}
#[test]
fn test_bridge_error_is_std() {
    let e: Box<dyn std::error::Error> = Box::new(BridgeError::InitFailed("t".into()));
    assert!(e.to_string().contains("init failed"));
}

// ── Rect ────────────────────────────────────────────────────────────

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
    assert!(Rect::new(0, 0, 0, 5).is_empty());
}
#[test]
fn test_rect_default() {
    let r = Rect::default();
    assert!(r.is_empty());
    assert_eq!(r.area(), 0);
}
#[test]
fn test_rect_inner() {
    let r = Rect::new(0, 0, 20, 10).inner(&Margin::all(1));
    assert_eq!(r, Rect::new(1, 1, 18, 8));
}
#[test]
fn test_rect_inner_overflow() {
    let r = Rect::new(0, 0, 5, 5).inner(&Margin::all(10));
    assert_eq!(r.width, 0);
}

// ── Margin ──────────────────────────────────────────────────────────

#[test]
fn test_margin_new() {
    let m = Margin::new(1, 2, 3, 4);
    assert_eq!(m.top, 1);
    assert_eq!(m.right, 2);
}
#[test]
fn test_margin_all() {
    let m = Margin::all(5);
    assert_eq!(m.top, 5);
    assert_eq!(m.left, 5);
}
#[test]
fn test_margin_symmetric() {
    let m = Margin::symmetric(2, 3);
    assert_eq!(m.top, 2);
    assert_eq!(m.left, 3);
}

// ── TextStyle ───────────────────────────────────────────────────────

#[test]
fn test_text_style_builder() {
    let s = TextStyle::default()
        .fg(Color::Red)
        .bg(Color::Black)
        .bold()
        .underline()
        .italic();
    assert!(s.bold);
    assert!(s.underline);
    assert!(s.italic);
}

// ── Alignment ───────────────────────────────────────────────────────

#[test]
fn test_alignment_default() {
    assert_eq!(Alignment::default(), Alignment::Left);
}

// ── Color ───────────────────────────────────────────────────────────

#[test]
fn test_color_serde() {
    assert!(serde_json::to_string(&Color::Indexed(42))
        .unwrap()
        .contains("42"));
    assert!(serde_json::to_string(&Color::Rgb(255, 0, 0))
        .unwrap()
        .contains("255"));
}

// ── KeyEvent / Modifiers ────────────────────────────────────────────

#[test]
fn test_key_event_new() {
    let e = KeyEvent::new(KeyCode::Char('a'));
    assert_eq!(e.code, KeyCode::Char('a'));
    assert!(e.modifiers.is_empty());
}
#[test]
fn test_key_event_with_mods() {
    let e = KeyEvent::with_modifiers(KeyCode::Char('c'), Modifiers::ctrl());
    assert!(e.modifiers.contains(Modifiers::CTRL));
}
#[test]
fn test_modifiers_shift() {
    assert!(Modifiers::shift().contains(Modifiers::SHIFT));
    assert!(!Modifiers::shift().contains(Modifiers::CTRL));
}
#[test]
fn test_modifiers_alt() {
    assert!(Modifiers::alt().contains(Modifiers::ALT));
}
#[test]
fn test_modifiers_combined() {
    let m = Modifiers::from_bits(Modifiers::CTRL | Modifiers::SHIFT);
    assert!(m.contains(Modifiers::CTRL));
    assert!(m.contains(Modifiers::SHIFT));
    assert!(!m.contains(Modifiers::ALT));
}

// ── Event / EventResult ─────────────────────────────────────────────

#[test]
fn test_event_serde() {
    assert!(
        serde_json::to_string(&Event::Key(KeyEvent::new(KeyCode::Enter)))
            .unwrap()
            .contains("Enter")
    );
    assert!(serde_json::to_string(&Event::Tick)
        .unwrap()
        .contains("Tick"));
}
#[test]
fn test_event_result_eq() {
    assert_eq!(EventResult::Consumed, EventResult::Consumed);
    assert_ne!(EventResult::Consumed, EventResult::Ignored);
}

// ── Layout ──────────────────────────────────────────────────────────

#[test]
fn test_layout_empty() {
    let r = Layout::new().split(Rect::new(0, 0, 80, 24));
    assert_eq!(r.len(), 1);
}
#[test]
fn test_layout_fixed_vertical() {
    let r = Layout::new()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fixed(5), Constraint::Fixed(10)])
        .split(Rect::new(0, 0, 80, 24));
    assert_eq!(r[0].height, 5);
    assert_eq!(r[1].height, 10);
}
#[test]
fn test_layout_percentage() {
    let r = Layout::new()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(Rect::new(0, 0, 80, 24));
    assert_eq!(r[0].width, 40);
}
#[test]
fn test_layout_fill() {
    let r = Layout::new()
        .constraints(vec![
            Constraint::Fixed(3),
            Constraint::Fill,
            Constraint::Fixed(1),
        ])
        .split(Rect::new(0, 0, 80, 24));
    assert_eq!(r[1].height, 20);
}
#[test]
fn test_layout_default() {
    assert_eq!(Layout::default().get_direction(), Direction::Vertical);
}

// ── Router / Navigation ─────────────────────────────────────────────

#[test]
fn test_router_navigate() {
    let mut r = Router::new(vec!["Main".into(), "Detail".into()], "Main");
    assert_eq!(r.current().screen, "Main");
    r.navigate("Detail".into(), HashMap::new()).unwrap();
    assert_eq!(r.current().screen, "Detail");
}
#[test]
fn test_router_back() {
    let mut r = Router::new(vec!["A".into(), "B".into()], "A");
    r.navigate("B".into(), HashMap::new()).unwrap();
    r.back().unwrap();
    assert_eq!(r.current().screen, "A");
}
#[test]
fn test_router_forward() {
    let mut r = Router::new(vec!["A".into(), "B".into()], "A");
    r.navigate("B".into(), HashMap::new()).unwrap();
    r.back().unwrap();
    r.forward().unwrap();
    assert_eq!(r.current().screen, "B");
}
#[test]
fn test_router_screen_not_found() {
    assert!(Router::new(vec!["A".into()], "A")
        .navigate("Z".into(), HashMap::new())
        .is_err());
}
#[test]
fn test_router_deep_links() {
    let mut r = Router::new(vec!["A".into(), "P".into()], "A");
    r.add_deep_link("/profile".into(), "P".into());
    assert_eq!(r.resolve_deep_link("/profile"), Some(&"P".into()));
    assert!(r.resolve_deep_link("/x").is_none());
}
#[test]
fn test_router_can_go_back_forward() {
    let mut r = Router::new(vec!["A".into(), "B".into()], "A");
    assert!(!r.can_go_back());
    r.navigate("B".into(), HashMap::new()).unwrap();
    assert!(r.can_go_back());
    assert!(!r.can_go_forward());
}
#[test]
fn test_navigation_error_display() {
    assert!(format!("{}", NavigationError::ScreenNotFound("X".into())).contains("X"));
    assert!(format!("{}", NavigationError::NoHistory).contains("No history"));
}

// ── StateStore ──────────────────────────────────────────────────────

#[test]
fn test_state_define_get() {
    let mut s = StateStore::new();
    s.define("c".into(), StateData::Number(0.0), StateScope::Screen);
    assert_eq!(s.get("c"), Some(&StateData::Number(0.0)));
}
#[test]
fn test_state_set_affected() {
    let mut s = StateStore::new();
    s.define("c".into(), StateData::Number(0.0), StateScope::Screen);
    s.bind("c".into(), "l1".into(), "text".into());
    let a = s.set("c", StateData::Number(1.0));
    assert_eq!(a.len(), 1);
}
#[test]
fn test_state_no_change_no_dirty() {
    let mut s = StateStore::new();
    s.define("f".into(), StateData::Boolean(true), StateScope::App);
    let a = s.set("f", StateData::Boolean(true));
    assert!(a.is_empty());
}
#[test]
fn test_state_drain_dirty() {
    let mut s = StateStore::new();
    s.define("a".into(), StateData::Number(1.0), StateScope::App);
    s.set("a", StateData::Number(10.0));
    assert_eq!(s.drain_dirty().len(), 1);
    assert!(s.drain_dirty().is_empty());
}
#[test]
fn test_state_clear_screen() {
    let mut s = StateStore::new();
    s.define("sv".into(), StateData::Number(1.0), StateScope::Screen);
    s.define("av".into(), StateData::Number(2.0), StateScope::App);
    s.clear_screen_state();
    assert!(s.get("sv").is_none());
    assert!(s.get("av").is_some());
}

// ── Theme ───────────────────────────────────────────────────────────

#[test]
fn test_theme_default() {
    let t = Theme::default();
    assert_eq!(t.name, "default");
    assert_eq!(t.mode, ThemeMode::Light);
}
#[test]
fn test_theme_dark_colors() {
    let mut t = Theme::default();
    t.mode = ThemeMode::Dark;
    assert_eq!(t.active_colors().background, "#121212");
}
#[test]
fn test_theme_system_mode() {
    let mut t = Theme::default();
    t.mode = ThemeMode::System;
    assert_eq!(t.active_colors().background, "#FFFFFF");
}
#[test]
fn test_theme_dark_no_dark_fallback() {
    let mut t = Theme::default();
    t.mode = ThemeMode::Dark;
    t.dark_colors = None;
    assert_eq!(t.active_colors().background, "#FFFFFF");
}
