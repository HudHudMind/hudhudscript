use hudhudscript_ui_core::state::{StateData, StateScope, StateStore};

#[test]
fn test_state_define_and_get() {
    let mut store = StateStore::new();
    store.define("count".into(), StateData::Number(0.0), StateScope::Screen);
    assert_eq!(store.get("count"), Some(&StateData::Number(0.0)));
}

#[test]
fn test_state_set_returns_affected_widgets() {
    let mut store = StateStore::new();
    store.define("count".into(), StateData::Number(0.0), StateScope::Screen);
    store.bind("count".into(), "label1".into(), "text".into());
    store.bind("count".into(), "label2".into(), "text".into());

    let affected = store.set("count", StateData::Number(1.0));
    assert_eq!(affected.len(), 2);
    assert!(affected.contains(&"label1".to_string()));
    assert!(affected.contains(&"label2".to_string()));
}

#[test]
fn test_state_no_change_no_dirty() {
    let mut store = StateStore::new();
    store.define("flag".into(), StateData::Boolean(true), StateScope::App);
    store.bind("flag".into(), "w1".into(), "visible".into());

    // Same value — no affected widgets
    let affected = store.set("flag", StateData::Boolean(true));
    assert!(affected.is_empty());
}

#[test]
fn test_drain_dirty() {
    let mut store = StateStore::new();
    store.define("a".into(), StateData::Number(1.0), StateScope::App);
    store.define("b".into(), StateData::Number(2.0), StateScope::App);
    store.set("a", StateData::Number(10.0));

    let dirty = store.drain_dirty();
    assert_eq!(dirty, vec!["a".to_string()]);

    // After drain, no more dirty
    let dirty2 = store.drain_dirty();
    assert!(dirty2.is_empty());
}

#[test]
fn test_bindings_for() {
    let mut store = StateStore::new();
    store.define(
        "name".into(),
        StateData::String("test".into()),
        StateScope::Screen,
    );
    store.bind("name".into(), "title".into(), "text".into());
    store.bind("name".into(), "header".into(), "label".into());
    store.bind("other".into(), "footer".into(), "text".into());

    assert_eq!(store.bindings_for("name").len(), 2);
    assert_eq!(store.bindings_for("other").len(), 1);
}

#[test]
fn test_clear_screen_state() {
    let mut store = StateStore::new();
    store.define(
        "screen_val".into(),
        StateData::Number(1.0),
        StateScope::Screen,
    );
    store.define("app_val".into(), StateData::Number(2.0), StateScope::App);
    store.bind("screen_val".into(), "w1".into(), "text".into());
    store.bind("app_val".into(), "w2".into(), "text".into());

    store.clear_screen_state();

    // Screen-scoped state should be removed
    assert_eq!(store.get("screen_val"), None);
    // App-scoped state should remain
    assert_eq!(store.get("app_val"), Some(&StateData::Number(2.0)));
}

#[test]
fn test_set_undefined_state_returns_empty() {
    let mut store = StateStore::new();
    let affected = store.set("undefined", StateData::Number(1.0));
    assert!(affected.is_empty());
}

#[test]
fn test_get_missing_state() {
    let store = StateStore::new();
    assert_eq!(store.get("nope"), None);
}

#[test]
fn test_state_data_equality() {
    assert_eq!(StateData::Null, StateData::Null);
    assert_ne!(StateData::Number(1.0), StateData::Number(2.0));
    assert_eq!(StateData::String("a".into()), StateData::String("a".into()));
    assert_ne!(StateData::Boolean(true), StateData::Boolean(false));
}

#[test]
fn test_drain_dirty_multiple() {
    let mut store = StateStore::new();
    store.define("x".into(), StateData::Number(0.0), StateScope::App);
    store.define("y".into(), StateData::Number(0.0), StateScope::App);
    store.set("x", StateData::Number(1.0));
    store.set("y", StateData::Number(2.0));

    let mut dirty = store.drain_dirty();
    dirty.sort();
    assert_eq!(dirty.len(), 2);
    assert!(dirty.contains(&"x".to_string()));
    assert!(dirty.contains(&"y".to_string()));
}
