use hudhudscript_ui_core::{
    App, BridgeMessage, ComponentDef, PropValue, Screen, Style, UIEvent, UIUpdate, Widget,
    WidgetType,
};
use std::collections::HashMap;

#[test]
fn test_widget_tree_serialization() {
    let button = Widget {
        id: "btn1".to_string(),
        widget_type: WidgetType::Button,
        props: {
            let mut p = HashMap::new();
            p.insert(
                "label".to_string(),
                PropValue::String("Click me".to_string()),
            );
            p
        },
        events: {
            let mut e = HashMap::new();
            e.insert("on_click".to_string(), "handle_click".to_string());
            e
        },
        children: vec![],
        style: Style::default(),
    };

    let json = serde_json::to_string(&button).unwrap();
    assert!(json.contains("btn1"));
    assert!(json.contains("Click me"));
}

#[test]
fn test_bridge_message_serialization() {
    let event = BridgeMessage::Event(UIEvent::Click {
        widget_id: "btn1".to_string(),
    });
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("btn1"));
}

#[test]
fn test_ui_event_serialization_variants() {
    let input_ev = UIEvent::Input {
        widget_id: "input1".to_string(),
        value: "hello".to_string(),
    };
    let json = serde_json::to_string(&input_ev).unwrap();
    assert!(json.contains("input1"));
    assert!(json.contains("hello"));

    let select_ev = UIEvent::Select {
        widget_id: "sel1".to_string(),
        value: "opt2".to_string(),
    };
    let json2 = serde_json::to_string(&select_ev).unwrap();
    assert!(json2.contains("sel1"));

    let nav_ev = UIEvent::Navigate {
        screen: "Settings".to_string(),
        params: HashMap::new(),
    };
    let json3 = serde_json::to_string(&nav_ev).unwrap();
    assert!(json3.contains("Settings"));
}

#[test]
fn test_ui_update_serialization() {
    let update = UIUpdate::Remove {
        id: "widget1".to_string(),
    };
    let json = serde_json::to_string(&update).unwrap();
    assert!(json.contains("widget1"));

    let update2 = UIUpdate::UpdateStyle {
        id: "w2".to_string(),
        style: Style::default(),
    };
    let json2 = serde_json::to_string(&update2).unwrap();
    assert!(json2.contains("w2"));
}

#[test]
fn test_component_def_serialization() {
    let comp = ComponentDef {
        name: "MyButton".to_string(),
        props: HashMap::new(),
        template: Widget {
            id: "root".to_string(),
            widget_type: WidgetType::Button,
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![],
            style: Style::default(),
        },
        on_mount: Some("onMountHandler".to_string()),
        on_update: None,
        on_unmount: None,
    };
    let json = serde_json::to_string(&comp).unwrap();
    assert!(json.contains("MyButton"));
    assert!(json.contains("onMountHandler"));
}

#[test]
fn test_screen_serialization() {
    let screen = Screen {
        name: "Home".to_string(),
        params: vec!["id".to_string()],
        root: Widget {
            id: "root".to_string(),
            widget_type: WidgetType::Screen,
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![],
            style: Style::default(),
        },
    };
    let json = serde_json::to_string(&screen).unwrap();
    assert!(json.contains("Home"));
}

#[test]
fn test_prop_value_variants() {
    let arr = PropValue::Array(vec![
        PropValue::Number(1.0),
        PropValue::Boolean(true),
        PropValue::Null,
    ]);
    let json = serde_json::to_string(&arr).unwrap();
    assert!(json.contains("1.0"));

    let mut map = HashMap::new();
    map.insert("key".to_string(), PropValue::String("val".to_string()));
    let obj = PropValue::Object(map);
    let json2 = serde_json::to_string(&obj).unwrap();
    assert!(json2.contains("key"));
}

#[test]
fn test_bridge_message_shutdown() {
    let msg = BridgeMessage::Shutdown;
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("Shutdown"));

    let msg2 = BridgeMessage::Ready;
    let json2 = serde_json::to_string(&msg2).unwrap();
    assert!(json2.contains("Ready"));

    let msg3 = BridgeMessage::Error("oops".to_string());
    let json3 = serde_json::to_string(&msg3).unwrap();
    assert!(json3.contains("oops"));
}

#[test]
fn test_ui_update_insert_serialization() {
    let update = UIUpdate::Insert {
        parent_id: "container".to_string(),
        index: 2,
        widget: Widget {
            id: "new_widget".to_string(),
            widget_type: WidgetType::Text,
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![],
            style: Style::default(),
        },
    };
    let json = serde_json::to_string(&update).unwrap();
    assert!(json.contains("new_widget"));
    assert!(json.contains("container"));
}

#[test]
fn test_ui_update_navigate_serialization() {
    let update = UIUpdate::Navigate {
        screen: "Settings".to_string(),
        params: HashMap::new(),
    };
    let json = serde_json::to_string(&update).unwrap();
    assert!(json.contains("Settings"));
}

#[test]
fn test_ui_update_update_widget_serialization() {
    let update = UIUpdate::UpdateWidget {
        id: "w1".to_string(),
        props: {
            let mut p = HashMap::new();
            p.insert("text".to_string(), PropValue::String("hello".to_string()));
            p
        },
    };
    let json = serde_json::to_string(&update).unwrap();
    assert!(json.contains("w1"));
    assert!(json.contains("hello"));
}

#[test]
fn test_ui_event_submit_serialization() {
    let ev = UIEvent::Submit {
        widget_id: "form1".to_string(),
    };
    let json = serde_json::to_string(&ev).unwrap();
    assert!(json.contains("form1"));
}

#[test]
fn test_ui_event_custom_serialization() {
    let ev = UIEvent::Custom {
        name: "my_event".to_string(),
        data: HashMap::new(),
    };
    let json = serde_json::to_string(&ev).unwrap();
    assert!(json.contains("my_event"));
}

#[test]
fn test_widget_type_platform_serialization() {
    let w = Widget {
        id: "native".to_string(),
        widget_type: WidgetType::Platform("gtk_button".to_string()),
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![],
        style: Style::default(),
    };
    let json = serde_json::to_string(&w).unwrap();
    assert!(json.contains("gtk_button"));
}

#[test]
fn test_widget_type_component_serialization() {
    let w = Widget {
        id: "comp".to_string(),
        widget_type: WidgetType::Component("MyWidget".to_string()),
        props: HashMap::new(),
        events: HashMap::new(),
        children: vec![],
        style: Style::default(),
    };
    let json = serde_json::to_string(&w).unwrap();
    assert!(json.contains("MyWidget"));
}

#[test]
fn test_bridge_message_render_serialization() {
    let app = App {
        name: "TestApp".to_string(),
        entry_screen: "Main".to_string(),
        screens: vec![],
        components: vec![],
    };
    let msg = BridgeMessage::Render(Box::new(app));
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("TestApp"));
}

#[test]
fn test_bridge_message_update_serialization() {
    let msg = BridgeMessage::Update(UIUpdate::Remove {
        id: "w1".to_string(),
    });
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("w1"));
}

#[test]
fn test_style_default_all_none() {
    let s = Style::default();
    assert!(s.size.is_none());
    assert!(s.color.is_none());
    assert!(s.background.is_none());
    assert!(s.bold.is_none());
    assert!(s.display.is_none());
    assert!(s.flex_direction.is_none());
    assert!(s.justify_content.is_none());
    assert!(s.align_items.is_none());
    assert!(s.flex_wrap.is_none());
    assert!(s.overflow.is_none());
    assert!(s.padding.is_none());
    assert!(s.margin.is_none());
    assert!(s.border_radius.is_none());
    assert!(s.shadow.is_none());
    assert!(s.opacity.is_none());
    assert!(s.theme_variant.is_none());
}
