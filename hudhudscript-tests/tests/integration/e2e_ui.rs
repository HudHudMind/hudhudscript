//! E2E UI pipeline tests (#559)
//!
//! Tests the full flow: build App → Bridge init → render → update → event → shutdown

use hudhudscript_ui_bridge::*;
use hudhudscript_ui_core::*;
use std::collections::HashMap;

fn sample_app() -> App {
    App {
        name: "TestApp".to_string(),
        entry_screen: "Main".to_string(),
        screens: vec![Screen {
            name: "Main".to_string(),
            params: vec![],
            root: Widget {
                id: "root".to_string(),
                widget_type: WidgetType::Column,
                props: HashMap::new(),
                events: HashMap::new(),
                children: vec![
                    Widget {
                        id: "title".to_string(),
                        widget_type: WidgetType::Text,
                        props: {
                            let mut p = HashMap::new();
                            p.insert("label".into(), PropValue::String("Hello".into()));
                            p
                        },
                        events: HashMap::new(),
                        children: vec![],
                        style: Style {
                            size: Some(24.0),
                            bold: Some(true),
                            ..Style::default()
                        },
                    },
                    Widget {
                        id: "btn1".to_string(),
                        widget_type: WidgetType::Button,
                        props: {
                            let mut p = HashMap::new();
                            p.insert("label".into(), PropValue::String("Click".into()));
                            p
                        },
                        events: {
                            let mut e = HashMap::new();
                            e.insert("on_click".into(), "handle_click".into());
                            e
                        },
                        children: vec![],
                        style: Style::default(),
                    },
                ],
                style: Style::default(),
            },
        }],
        components: vec![],
    }
}

#[test]
fn test_web_bridge_full_lifecycle() {
    let mut bridge = create_bridge(&Framework::Web).unwrap();
    assert!(bridge.init().is_ok());
    assert!(bridge.render(&sample_app()).is_ok());

    // Partial update
    let update = UIUpdate::UpdateWidget {
        id: "title".into(),
        props: {
            let mut p = HashMap::new();
            p.insert("label".into(), PropValue::String("Updated".into()));
            p
        },
    };
    assert!(bridge.update(&update).is_ok());

    // Poll — no events from stub
    assert!(bridge.poll_event().unwrap().is_none());
    assert!(bridge.shutdown().is_ok());
}

#[test]
fn test_tauri_bridge_returns_unsupported() {
    // v0.4.47.9: Tauri bridge returns Unsupported until real tauri crate is linked.
    let mut bridge = create_bridge(&Framework::Tauri).unwrap();
    assert!(
        bridge.init().is_err(),
        "tauri::init must return Unsupported"
    );
    assert!(bridge.render(&sample_app()).is_err());
    assert!(bridge.poll_event().is_err());
    // Shutdown is the only no-op success
    assert!(bridge.shutdown().is_ok());
}

#[test]
fn test_flutter_bridge_full_lifecycle() {
    // FlutterBridge serializes app to JSON for FFI transport — real behavior.
    let mut bridge = create_bridge(&Framework::Flutter).unwrap();
    assert!(bridge.init().is_ok());
    assert!(bridge.render(&sample_app()).is_ok());
    assert!(bridge.shutdown().is_ok());
}

#[test]
fn test_wasm_bridge_full_lifecycle() {
    let mut bridge = create_bridge(&Framework::Wasm).unwrap();
    assert!(bridge.init().is_ok());
    assert!(bridge.render(&sample_app()).is_ok());

    // Insert a new widget
    let insert = UIUpdate::Insert {
        parent_id: "root".into(),
        index: 2,
        widget: Widget {
            id: "new_text".into(),
            widget_type: WidgetType::Text,
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![],
            style: Style::default(),
        },
    };
    assert!(bridge.update(&insert).is_ok());

    // Remove a widget
    let remove = UIUpdate::Remove {
        id: "new_text".into(),
    };
    assert!(bridge.update(&remove).is_ok());

    assert!(bridge.shutdown().is_ok());
}

#[test]
fn test_app_json_roundtrip() {
    let app = sample_app();
    let json = serde_json::to_string(&app).unwrap();
    let deserialized: App = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "TestApp");
    assert_eq!(deserialized.screens.len(), 1);
    assert_eq!(deserialized.screens[0].root.children.len(), 2);
}

#[test]
fn test_web_html_generation() {
    let widget = Widget {
        id: "col".into(),
        widget_type: WidgetType::Column,
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![
            Widget {
                id: "t".into(),
                widget_type: WidgetType::Text,
                props: {
                    let mut p = HashMap::new();
                    p.insert("label".into(), PropValue::String("Test".into()));
                    p
                },
                events: HashMap::new(),
                children: vec![],
                style: Style {
                    color: Some("#f00".into()),
                    ..Style::default()
                },
            },
            Widget {
                id: "b".into(),
                widget_type: WidgetType::Button,
                props: {
                    let mut p = HashMap::new();
                    p.insert("label".into(), PropValue::String("OK".into()));
                    p
                },
                events: HashMap::new(),
                children: vec![],
                style: Style::default(),
            },
        ],
        style: Style::default(),
    };
    let html = hudhudscript_ui_bridge::web::WebBridge::widget_to_html(&widget);
    assert!(html.contains("<div"));
    assert!(html.contains("flex-direction:column"));
    assert!(html.contains("Test"));
    assert!(html.contains("<button"));
    assert!(html.contains("OK"));
    assert!(html.contains("color:#f00"));
}
