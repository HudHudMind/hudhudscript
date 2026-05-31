use hudhudscript_ui_core::event::{
    Event, EventResult, KeyCode, KeyEvent, Modifiers, MouseButton, MouseEvent, MouseEventKind,
};

#[test]
fn test_key_event_creation() {
    let ev = KeyEvent::new(KeyCode::Char('a'));
    assert_eq!(ev.code, KeyCode::Char('a'));
    assert!(ev.modifiers.is_empty());
}

#[test]
fn test_key_event_with_modifiers() {
    let ev = KeyEvent::with_modifiers(KeyCode::Char('c'), Modifiers::ctrl());
    assert!(ev.modifiers.contains(Modifiers::CTRL));
    assert!(!ev.modifiers.contains(Modifiers::SHIFT));
}

#[test]
fn test_modifier_combinations() {
    let mods = Modifiers::from_bits(Modifiers::CTRL | Modifiers::SHIFT);
    assert!(mods.contains(Modifiers::CTRL));
    assert!(mods.contains(Modifiers::SHIFT));
    assert!(!mods.contains(Modifiers::ALT));
}

#[test]
fn test_mouse_event() {
    let ev = MouseEvent {
        kind: MouseEventKind::Click,
        x: 10,
        y: 20,
        button: MouseButton::Left,
    };
    assert_eq!(ev.x, 10);
    assert_eq!(ev.y, 20);
}

#[test]
fn test_event_result() {
    let result = EventResult::Action("submit".to_string());
    assert_eq!(result, EventResult::Action("submit".to_string()));
}

#[test]
fn test_event_serialization() {
    let ev = Event::Key(KeyEvent::new(KeyCode::Enter));
    let json = serde_json::to_string(&ev).unwrap();
    assert!(json.contains("Enter"));

    let ev2 = Event::Resize(80, 24);
    let json2 = serde_json::to_string(&ev2).unwrap();
    assert!(json2.contains("80"));
}

#[test]
fn test_modifiers_shift() {
    let m = Modifiers::shift();
    assert!(m.contains(Modifiers::SHIFT));
    assert!(!m.contains(Modifiers::CTRL));
    assert!(!m.is_empty());
}

#[test]
fn test_modifiers_alt() {
    let m = Modifiers::alt();
    assert!(m.contains(Modifiers::ALT));
    assert!(!m.contains(Modifiers::CTRL));
}

#[test]
fn test_event_tick_serialization() {
    let ev = Event::Tick;
    let json = serde_json::to_string(&ev).unwrap();
    assert!(json.contains("Tick"));
}

#[test]
fn test_mouse_event_serialization() {
    let ev = Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        x: 5,
        y: 10,
        button: MouseButton::None,
    });
    let json = serde_json::to_string(&ev).unwrap();
    assert!(json.contains("ScrollUp"));
}

#[test]
fn test_event_result_variants() {
    assert_eq!(EventResult::Consumed, EventResult::Consumed);
    assert_eq!(EventResult::Ignored, EventResult::Ignored);
    assert_ne!(EventResult::Consumed, EventResult::Ignored);
}

#[test]
fn test_key_code_f_keys() {
    let ev = KeyEvent::new(KeyCode::F(12));
    assert_eq!(ev.code, KeyCode::F(12));
}
