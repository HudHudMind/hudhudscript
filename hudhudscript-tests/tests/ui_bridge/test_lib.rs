use hudhudscript_ui_bridge::fallback::StubBridge;
use hudhudscript_ui_bridge::flutter_bridge::FlutterBridge;
use hudhudscript_ui_bridge::tauri_bridge::TauriBridge;
use hudhudscript_ui_bridge::wasm_bridge::WasmBridge;
use hudhudscript_ui_bridge::web::WebBridge;
use hudhudscript_ui_bridge::{create_bridge, Framework};
use hudhudscript_ui_core::{App, PropValue, Screen, Style, UIBridge, UIUpdate, Widget, WidgetType};
use std::collections::HashMap;

// =========================================================================
// flutter_bridge
// =========================================================================

#[test]
fn test_flutter_bridge_default() {
    let bridge = FlutterBridge::default();
    assert_eq!(bridge.name(), "flutter(mobile)");
}

#[test]
fn test_flutter_bridge_lifecycle() {
    let mut bridge = FlutterBridge::new();
    assert!(bridge.init().is_ok());
    assert!(bridge.poll_event().unwrap().is_none());
    assert!(bridge.shutdown().is_ok());
}

#[test]
fn test_flutter_bridge_render() {
    let mut bridge = FlutterBridge::new();
    bridge.init().unwrap();
    let app = App {
        name: "MyApp".to_string(),
        entry_screen: "home".to_string(),
        screens: vec![],
        components: vec![],
    };
    assert!(bridge.render(&app).is_ok());
}

#[test]
fn test_flutter_widget_mapping_remaining() {
    assert_eq!(FlutterBridge::flutter_widget_name(&WidgetType::Row), "Row");
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Text),
        "Text"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Image),
        "Image"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Icon),
        "Icon"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Card),
        "Card"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Container),
        "Container"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Checkbox),
        "Checkbox"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Select),
        "DropdownButton"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Slider),
        "Slider"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Grid),
        "GridView"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Table),
        "DataTable"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::TabBar),
        "TabBar"
    );
    assert_eq!(FlutterBridge::flutter_widget_name(&WidgetType::Tab), "Tab");
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Navbar),
        "AppBar"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Link),
        "TextButton"
    );
}

#[test]
fn test_flutter_widget_mapping_fallback() {
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Screen),
        "Container"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Chart),
        "Container"
    );
}

#[test]
fn test_flutter_widget_mapping() {
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Button),
        "ElevatedButton"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Column),
        "Column"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::Input),
        "TextField"
    );
    assert_eq!(
        FlutterBridge::flutter_widget_name(&WidgetType::List),
        "ListView"
    );
}

#[test]
fn test_flutter_bridge_update() {
    let mut bridge = FlutterBridge::new();
    bridge.init().unwrap();
    let update = UIUpdate::Remove {
        id: "w1".to_string(),
    };
    assert!(bridge.update(&update).is_ok());
}

#[test]
fn test_flutter_bridge_render_with_screens() {
    let mut bridge = FlutterBridge::new();
    bridge.init().unwrap();
    let app = App {
        name: "FlutterApp".to_string(),
        entry_screen: "main".to_string(),
        screens: vec![Screen {
            name: "main".to_string(),
            params: vec![],
            root: Widget {
                id: "root".into(),
                widget_type: WidgetType::Column,
                props: HashMap::new(),
                events: HashMap::new(),
                children: vec![Widget {
                    id: "btn".into(),
                    widget_type: WidgetType::Button,
                    props: HashMap::new(),
                    events: HashMap::new(),
                    children: vec![],
                    style: Style::default(),
                }],
                style: Style::default(),
            },
        }],
        components: vec![],
    };
    assert!(bridge.render(&app).is_ok());
}

#[test]
fn test_flutter_bridge_init_sets_running() {
    let mut bridge = FlutterBridge::new();
    assert!(!bridge.running);
    bridge.init().unwrap();
    assert!(bridge.running);
}

#[test]
fn test_flutter_bridge_shutdown_sets_running_false() {
    let mut bridge = FlutterBridge::new();
    bridge.init().unwrap();
    assert!(bridge.running);
    bridge.shutdown().unwrap();
    assert!(!bridge.running);
}

// =========================================================================
// lib (Framework / create_bridge)
// =========================================================================

#[test]
fn test_framework_parsing() {
    assert!(matches!(Framework::parse("gtk"), Some(Framework::Gtk)));
    assert!(matches!(Framework::parse("tauri"), Some(Framework::Tauri)));
    assert!(matches!(Framework::parse("web"), Some(Framework::Web)));
    assert!(matches!(
        Framework::parse("my_custom"),
        Some(Framework::Custom(_))
    ));
}

#[test]
fn test_create_web_bridge() {
    let bridge = create_bridge(&Framework::Web);
    assert!(bridge.is_ok());
    assert_eq!(bridge.unwrap().name(), "web(nextjs)");
}

#[test]
fn test_create_wasm_bridge() {
    let bridge = create_bridge(&Framework::Wasm);
    assert!(bridge.is_ok());
    assert_eq!(bridge.unwrap().name(), "wasm(browser)");
}

#[test]
fn test_create_stub_for_unknown() {
    let bridge = create_bridge(&Framework::Gtk);
    assert!(bridge.is_ok());
    assert_eq!(bridge.unwrap().name(), "unsupported(Gtk)");
}

#[test]
fn test_framework_parse_case_insensitive() {
    assert!(matches!(Framework::parse("GTK"), Some(Framework::Gtk)));
    assert!(matches!(Framework::parse("Tauri"), Some(Framework::Tauri)));
    assert!(matches!(Framework::parse("WEB"), Some(Framework::Web)));
    assert!(matches!(Framework::parse("HTML"), Some(Framework::Web)));
}

#[test]
fn test_framework_parse_all_variants() {
    assert!(matches!(Framework::parse("qt"), Some(Framework::Qt)));
    assert!(matches!(Framework::parse("iced"), Some(Framework::Iced)));
    assert!(matches!(
        Framework::parse("electron"),
        Some(Framework::Electron)
    ));
    assert!(matches!(
        Framework::parse("wxwidgets"),
        Some(Framework::WxWidgets)
    ));
    assert!(matches!(Framework::parse("wx"), Some(Framework::WxWidgets)));
    assert!(matches!(
        Framework::parse("flutter"),
        Some(Framework::Flutter)
    ));
    assert!(matches!(Framework::parse("wasm"), Some(Framework::Wasm)));
    assert!(matches!(Framework::parse("wasm32"), Some(Framework::Wasm)));
    assert!(matches!(
        Framework::parse("webassembly"),
        Some(Framework::Wasm)
    ));
}

#[test]
fn test_framework_parse_custom_preserves_name() {
    match Framework::parse("my_framework") {
        Some(Framework::Custom(name)) => assert_eq!(name, "my_framework"),
        _ => panic!("expected Custom variant"),
    }
}

#[test]
fn test_create_tauri_bridge() {
    let bridge = create_bridge(&Framework::Tauri);
    assert!(bridge.is_ok());
    assert_eq!(bridge.unwrap().name(), "tauri(desktop)");
}

#[test]
fn test_create_flutter_bridge() {
    let bridge = create_bridge(&Framework::Flutter);
    assert!(bridge.is_ok());
    assert_eq!(bridge.unwrap().name(), "flutter(mobile)");
}

#[test]
fn test_create_bridge_for_all_stub_frameworks() {
    for fw in &[
        Framework::Qt,
        Framework::Iced,
        Framework::Electron,
        Framework::WxWidgets,
        Framework::Custom("test".to_string()),
    ] {
        let bridge = create_bridge(fw);
        assert!(bridge.is_ok());
        assert!(bridge.unwrap().name().starts_with("unsupported("));
    }
}

// =========================================================================
// stub
// =========================================================================

// v0.4.47.9: StubBridge now returns honest BridgeError::Unsupported errors
// instead of fake successes. Tests updated to verify the error behavior.

#[test]
fn test_stub_bridge_init_returns_unsupported() {
    let mut bridge = StubBridge::new("TestFramework".to_string());
    assert_eq!(bridge.name(), "unsupported(TestFramework)");
    assert!(bridge.init().is_err(), "init must return Unsupported");
    // Shutdown is the only no-op success
    assert!(bridge.shutdown().is_ok());
}

#[test]
fn test_stub_bridge_render_returns_unsupported() {
    let mut bridge = StubBridge::new("Test".to_string());
    let app = App {
        name: "TestApp".to_string(),
        entry_screen: "main".to_string(),
        screens: vec![],
        components: vec![],
    };
    assert!(
        bridge.render(&app).is_err(),
        "render must return Unsupported"
    );
}

#[test]
fn test_stub_bridge_poll_event_returns_unsupported() {
    let mut bridge = StubBridge::new("Test".to_string());
    let result = bridge.poll_event();
    assert!(
        result.is_err(),
        "poll_event must return Unsupported (was previously Ok(None))"
    );
}

#[test]
fn test_stub_bridge_update_returns_unsupported() {
    let mut bridge = StubBridge::new("Test".to_string());
    let update = UIUpdate::Remove {
        id: "widget1".to_string(),
    };
    assert!(
        bridge.update(&update).is_err(),
        "update must return Unsupported"
    );
}

#[test]
fn test_stub_bridge_shutdown_succeeds() {
    let mut bridge = StubBridge::new("StateTest".to_string());
    // Shutdown is no-op success — closing a non-running bridge.
    assert!(bridge.shutdown().is_ok());
    assert!(!bridge.running);
}

#[test]
fn test_stub_bridge_name_format() {
    let bridge = StubBridge::new("GTK".to_string());
    assert_eq!(bridge.name(), "unsupported(GTK)");

    let bridge = StubBridge::new("MyCustomFramework".to_string());
    assert_eq!(bridge.name(), "unsupported(MyCustomFramework)");
}

// =========================================================================
// tauri_bridge
// =========================================================================
//
// v0.4.47.9: TauriBridge now returns BridgeError::Unsupported for init/render
// /update/poll_event because the real `tauri` crate is not linked at the
// workspace level. Tests verify the honest error behavior.

#[test]
fn test_tauri_bridge_init_returns_unsupported() {
    let mut bridge = TauriBridge::new();
    assert_eq!(bridge.name(), "tauri(desktop)");
    assert!(
        bridge.init().is_err(),
        "init must return Unsupported (no tauri crate linked)"
    );
}

#[test]
fn test_tauri_bridge_default() {
    let bridge = TauriBridge::default();
    assert_eq!(bridge.name(), "tauri(desktop)");
}

#[test]
fn test_tauri_bridge_render_returns_unsupported() {
    let mut bridge = TauriBridge::new();
    let app = App {
        name: "TestApp".to_string(),
        entry_screen: "main".to_string(),
        screens: vec![],
        components: vec![],
    };
    assert!(bridge.render(&app).is_err());
}

#[test]
fn test_tauri_bridge_update_returns_unsupported() {
    let mut bridge = TauriBridge::new();
    let update = UIUpdate::Remove {
        id: "w1".to_string(),
    };
    assert!(bridge.update(&update).is_err());
}

#[test]
fn test_tauri_bridge_poll_event_returns_unsupported() {
    let mut bridge = TauriBridge::new();
    assert!(bridge.poll_event().is_err());
}

#[test]
fn test_tauri_bridge_default_fields() {
    let bridge = TauriBridge::new();
    assert_eq!(bridge.title, "HudHudScript App");
    assert_eq!(bridge.width, 1200);
    assert_eq!(bridge.height, 800);
    assert!(!bridge.running);
}

#[test]
fn test_tauri_bridge_shutdown_succeeds() {
    let mut bridge = TauriBridge::new();
    // Shutdown is no-op success (closing a non-running bridge)
    assert!(bridge.shutdown().is_ok());
    assert!(!bridge.running);
}

// =========================================================================
// wasm_bridge
// =========================================================================

#[test]
fn test_widget_to_tag() {
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Button), "button");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Text), "p");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Column), "div");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Input), "input");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Table), "table");
}

#[test]
fn test_wasm_bridge_default() {
    let bridge = WasmBridge::default();
    assert_eq!(bridge.name(), "wasm(browser)");
}

#[test]
fn test_wasm_bridge_render_populates_element_map() {
    let mut bridge = WasmBridge::new();
    bridge.init().unwrap();
    let app = App {
        name: "TestApp".to_string(),
        entry_screen: "home".to_string(),
        screens: vec![Screen {
            name: "home".to_string(),
            params: vec![],
            root: Widget {
                id: "root".into(),
                widget_type: WidgetType::Column,
                props: HashMap::new(),
                events: HashMap::new(),
                children: vec![Widget {
                    id: "btn1".into(),
                    widget_type: WidgetType::Button,
                    props: HashMap::new(),
                    events: HashMap::new(),
                    children: vec![],
                    style: Style::default(),
                }],
                style: Style::default(),
            },
        }],
        components: vec![],
    };
    bridge.render(&app).unwrap();
    assert_eq!(bridge.element_map.len(), 2); // root + btn1
}

#[test]
fn test_wasm_bridge_update_remove() {
    let mut bridge = WasmBridge::new();
    bridge.init().unwrap();
    bridge
        .element_map
        .insert("w1".to_string(), "div".to_string());
    let update = UIUpdate::Remove {
        id: "w1".to_string(),
    };
    bridge.update(&update).unwrap();
    assert!(!bridge.element_map.contains_key("w1"));
}

#[test]
fn test_wasm_bridge_update_insert() {
    let mut bridge = WasmBridge::new();
    bridge.init().unwrap();
    let new_widget = Widget {
        id: "new1".into(),
        widget_type: WidgetType::Text,
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![],
        style: Style::default(),
    };
    let update = UIUpdate::Insert {
        parent_id: "root".to_string(),
        index: 0,
        widget: new_widget,
    };
    bridge.update(&update).unwrap();
    assert!(bridge.element_map.contains_key("new1"));
}

#[test]
fn test_wasm_bridge_shutdown_clears_map() {
    let mut bridge = WasmBridge::new();
    bridge.init().unwrap();
    bridge
        .element_map
        .insert("w1".to_string(), "div".to_string());
    bridge.shutdown().unwrap();
    assert!(bridge.element_map.is_empty());
}

#[test]
fn test_widget_to_tag_remaining() {
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Image), "img");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Link), "a");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::List), "ul");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Checkbox), "input");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Select), "select");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Slider), "input");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Navbar), "nav");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Menubar), "nav");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Menu), "ul");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::MenuItem), "li");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::TabBar), "div");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Tab), "button");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Icon), "span");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Container), "div");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Card), "div");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Grid), "div");
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Screen), "div");
}

#[test]
fn test_widget_to_tag_fallback() {
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Chart), "div");
}

#[test]
fn test_wasm_bridge_lifecycle() {
    let mut bridge = WasmBridge::new();
    assert!(bridge.init().is_ok());
    assert_eq!(bridge.name(), "wasm(browser)");
    assert!(bridge.shutdown().is_ok());
}

#[test]
fn test_wasm_bridge_update_existing_widget() {
    let mut bridge = WasmBridge::new();
    bridge.init().unwrap();
    bridge
        .element_map
        .insert("btn1".to_string(), "button".to_string());
    let update = UIUpdate::UpdateWidget {
        id: "btn1".to_string(),
        props: {
            let mut p = HashMap::new();
            p.insert("label".into(), PropValue::String("New Label".into()));
            p
        },
    };
    assert!(bridge.update(&update).is_ok());
    assert!(bridge.element_map.contains_key("btn1"));
}

#[test]
fn test_wasm_bridge_update_nonexistent_widget() {
    let mut bridge = WasmBridge::new();
    bridge.init().unwrap();
    let update = UIUpdate::UpdateWidget {
        id: "nonexistent".to_string(),
        props: HashMap::new(),
    };
    assert!(bridge.update(&update).is_ok());
}

#[test]
fn test_wasm_bridge_render_clears_old_map() {
    let mut bridge = WasmBridge::new();
    bridge.init().unwrap();
    bridge
        .element_map
        .insert("old".to_string(), "div".to_string());
    let app = App {
        name: "App".to_string(),
        entry_screen: "home".to_string(),
        screens: vec![Screen {
            name: "home".to_string(),
            params: vec![],
            root: Widget {
                id: "new_root".into(),
                widget_type: WidgetType::Column,
                props: HashMap::new(),
                events: HashMap::new(),
                children: vec![],
                style: Style::default(),
            },
        }],
        components: vec![],
    };
    bridge.render(&app).unwrap();
    assert!(!bridge.element_map.contains_key("old"));
    assert!(bridge.element_map.contains_key("new_root"));
}

#[test]
fn test_wasm_bridge_insert_nested() {
    let mut bridge = WasmBridge::new();
    bridge.init().unwrap();
    let nested = Widget {
        id: "parent".into(),
        widget_type: WidgetType::Column,
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![
            Widget {
                id: "child1".into(),
                widget_type: WidgetType::Text,
                props: HashMap::new(),
                events: HashMap::new(),
                children: vec![],
                style: Style::default(),
            },
            Widget {
                id: "child2".into(),
                widget_type: WidgetType::Button,
                props: HashMap::new(),
                events: HashMap::new(),
                children: vec![],
                style: Style::default(),
            },
        ],
        style: Style::default(),
    };
    let update = UIUpdate::Insert {
        parent_id: "root".to_string(),
        index: 0,
        widget: nested,
    };
    bridge.update(&update).unwrap();
    assert!(bridge.element_map.contains_key("parent"));
    assert!(bridge.element_map.contains_key("child1"));
    assert!(bridge.element_map.contains_key("child2"));
    assert_eq!(bridge.element_map["parent"], "div");
    assert_eq!(bridge.element_map["child1"], "p");
    assert_eq!(bridge.element_map["child2"], "button");
}

#[test]
fn test_widget_to_tag_row() {
    assert_eq!(WasmBridge::widget_to_tag(&WidgetType::Row), "div");
}

// =========================================================================
// web
// =========================================================================

#[test]
fn test_widget_to_html_text() {
    let widget = Widget {
        id: "t1".into(),
        widget_type: WidgetType::Text,
        props: {
            let mut p = HashMap::new();
            p.insert("label".into(), PropValue::String("Hello".into()));
            p
        },
        events: HashMap::new(),
        children: vec![],
        style: Style::default(),
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("<p"));
    assert!(html.contains("Hello"));
}

#[test]
fn test_web_bridge_lifecycle() {
    let mut bridge = WebBridge::new(8080);
    assert_eq!(bridge.name(), "web(nextjs)");
    assert!(bridge.init().is_ok());
    assert!(bridge.poll_event().unwrap().is_none());
    assert!(bridge.shutdown().is_ok());
}

#[test]
fn test_widget_to_html_button() {
    let widget = Widget {
        id: "b1".into(),
        widget_type: WidgetType::Button,
        props: {
            let mut p = HashMap::new();
            p.insert("label".into(), PropValue::String("Click me".into()));
            p
        },
        events: HashMap::new(),
        children: vec![],
        style: Style::default(),
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("<button"));
    assert!(html.contains("Click me"));
    assert!(html.contains("</button>"));
}

#[test]
fn test_widget_to_html_button_default_label() {
    let widget = Widget {
        id: "b2".into(),
        widget_type: WidgetType::Button,
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![],
        style: Style::default(),
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("Button"));
}

#[test]
fn test_widget_to_html_input() {
    let widget = Widget {
        id: "inp1".into(),
        widget_type: WidgetType::Input,
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![],
        style: Style::default(),
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("<input"));
    assert!(html.contains("id=\"inp1\""));
}

#[test]
fn test_widget_to_html_row() {
    let widget = Widget {
        id: "row1".into(),
        widget_type: WidgetType::Row,
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![],
        style: Style::default(),
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("flex-direction:row"));
}

#[test]
fn test_widget_to_html_fallback() {
    let widget = Widget {
        id: "chart1".into(),
        widget_type: WidgetType::Chart,
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![],
        style: Style::default(),
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("<div"));
    assert!(html.contains("id=\"chart1\""));
}

#[test]
fn test_style_to_css() {
    let widget = Widget {
        id: "styled".into(),
        widget_type: WidgetType::Text,
        props: {
            let mut p = HashMap::new();
            p.insert("label".into(), PropValue::String("Hi".into()));
            p
        },
        events: HashMap::new(),
        children: vec![],
        style: Style {
            color: Some("red".to_string()),
            background: Some("blue".to_string()),
            size: Some(16.0),
            bold: Some(true),
            width: Some("100".to_string()),
            height: Some("50".to_string()),
            opacity: Some(0.5),
            gap: Some(8.0),
            ..Style::default()
        },
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("color:red;"));
    assert!(html.contains("background:blue;"));
    assert!(html.contains("font-size:16px;"));
    assert!(html.contains("font-weight:bold;"));
    assert!(html.contains("width:100px;"));
    assert!(html.contains("height:50px;"));
    assert!(html.contains("opacity:0.5;"));
    assert!(html.contains("gap:8px;"));
}

#[test]
fn test_widget_to_html_text_no_label() {
    let widget = Widget {
        id: "t2".into(),
        widget_type: WidgetType::Text,
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![],
        style: Style::default(),
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("<p id=\"t2\">"));
    assert!(html.contains("</p>"));
}

#[test]
fn test_widget_to_html_text_non_string_label() {
    let widget = Widget {
        id: "t3".into(),
        widget_type: WidgetType::Text,
        props: {
            let mut p = HashMap::new();
            p.insert("label".into(), PropValue::Number(42.0));
            p
        },
        events: HashMap::new(),
        children: vec![],
        style: Style::default(),
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("<p id=\"t3\"></p>"));
}

#[test]
fn test_widget_to_html_column() {
    let widget = Widget {
        id: "col1".into(),
        widget_type: WidgetType::Column,
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![Widget {
            id: "t1".into(),
            widget_type: WidgetType::Text,
            props: {
                let mut p = HashMap::new();
                p.insert("label".into(), PropValue::String("Child".into()));
                p
            },
            events: HashMap::new(),
            children: vec![],
            style: Style::default(),
        }],
        style: Style::default(),
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("flex-direction:column"));
    assert!(html.contains("Child"));
}

#[test]
fn test_web_bridge_render_with_screens() {
    let mut bridge = WebBridge::new(9090);
    bridge.init().unwrap();
    let app = App {
        name: "MultiScreen".to_string(),
        entry_screen: "home".to_string(),
        screens: vec![
            Screen {
                name: "home".to_string(),
                params: vec![],
                root: Widget {
                    id: "root1".into(),
                    widget_type: WidgetType::Column,
                    props: HashMap::new(),
                    events: HashMap::new(),
                    children: vec![],
                    style: Style::default(),
                },
            },
            Screen {
                name: "settings".to_string(),
                params: vec![],
                root: Widget {
                    id: "root2".into(),
                    widget_type: WidgetType::Column,
                    props: HashMap::new(),
                    events: HashMap::new(),
                    children: vec![],
                    style: Style::default(),
                },
            },
        ],
        components: vec![],
    };
    assert!(bridge.render(&app).is_ok());
}

#[test]
fn test_web_bridge_update() {
    let mut bridge = WebBridge::new(8080);
    bridge.init().unwrap();
    let update = UIUpdate::Remove {
        id: "w1".to_string(),
    };
    assert!(bridge.update(&update).is_ok());
}

#[test]
fn test_style_to_css_empty() {
    let widget = Widget {
        id: "nostyle".into(),
        widget_type: WidgetType::Text,
        props: {
            let mut p = HashMap::new();
            p.insert("label".into(), PropValue::String("plain".into()));
            p
        },
        events: HashMap::new(),
        children: vec![],
        style: Style::default(),
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("plain"));
}

#[test]
fn test_style_to_css_bold_false() {
    let widget = Widget {
        id: "notbold".into(),
        widget_type: WidgetType::Text,
        props: {
            let mut p = HashMap::new();
            p.insert("label".into(), PropValue::String("text".into()));
            p
        },
        events: HashMap::new(),
        children: vec![],
        style: Style {
            bold: Some(false),
            ..Style::default()
        },
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(!html.contains("font-weight:bold"));
}

#[test]
fn test_widget_to_html_nested_column() {
    let widget = Widget {
        id: "outer".into(),
        widget_type: WidgetType::Column,
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![
            Widget {
                id: "child1".into(),
                widget_type: WidgetType::Text,
                props: {
                    let mut p = HashMap::new();
                    p.insert("label".into(), PropValue::String("First".into()));
                    p
                },
                events: HashMap::new(),
                children: vec![],
                style: Style::default(),
            },
            Widget {
                id: "child2".into(),
                widget_type: WidgetType::Button,
                props: {
                    let mut p = HashMap::new();
                    p.insert("label".into(), PropValue::String("Click".into()));
                    p
                },
                events: HashMap::new(),
                children: vec![],
                style: Style::default(),
            },
        ],
        style: Style::default(),
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("First"));
    assert!(html.contains("Click"));
    assert!(html.contains("flex-direction:column"));
}

#[test]
fn test_widget_to_html_row_with_children() {
    let widget = Widget {
        id: "row".into(),
        widget_type: WidgetType::Row,
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![
            Widget {
                id: "left".into(),
                widget_type: WidgetType::Text,
                props: {
                    let mut p = HashMap::new();
                    p.insert("label".into(), PropValue::String("Left".into()));
                    p
                },
                events: HashMap::new(),
                children: vec![],
                style: Style::default(),
            },
            Widget {
                id: "right".into(),
                widget_type: WidgetType::Text,
                props: {
                    let mut p = HashMap::new();
                    p.insert("label".into(), PropValue::String("Right".into()));
                    p
                },
                events: HashMap::new(),
                children: vec![],
                style: Style::default(),
            },
        ],
        style: Style::default(),
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("flex-direction:row"));
    assert!(html.contains("Left"));
    assert!(html.contains("Right"));
}

#[test]
fn test_widget_to_html_fallback_with_children() {
    let widget = Widget {
        id: "container".into(),
        widget_type: WidgetType::Card,
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![Widget {
            id: "inner".into(),
            widget_type: WidgetType::Text,
            props: {
                let mut p = HashMap::new();
                p.insert("label".into(), PropValue::String("Inside".into()));
                p
            },
            events: HashMap::new(),
            children: vec![],
            style: Style::default(),
        }],
        style: Style::default(),
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("<div id=\"container\""));
    assert!(html.contains("Inside"));
    assert!(html.contains("</div>"));
}

#[test]
fn test_style_partial() {
    let widget = Widget {
        id: "partial".into(),
        widget_type: WidgetType::Text,
        props: {
            let mut p = HashMap::new();
            p.insert("label".into(), PropValue::String("styled".into()));
            p
        },
        events: HashMap::new(),
        children: vec![],
        style: Style {
            color: Some("green".to_string()),
            opacity: Some(0.8),
            ..Style::default()
        },
    };
    let html = WebBridge::widget_to_html(&widget);
    assert!(html.contains("color:green;"));
    assert!(html.contains("opacity:0.8;"));
    assert!(!html.contains("background:"));
    assert!(!html.contains("font-size:"));
}
