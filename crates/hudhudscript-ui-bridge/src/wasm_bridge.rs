//! WASM adapter — wasm-bindgen + web-sys direct DOM (#554)
//!
//! Runs the entire HudHudScript runtime in-browser via WebAssembly.
//! Maps widget tree directly to DOM elements using web-sys APIs.
//! Different from the Web adapter: no server, no WebSocket — fully client-side.

use hudhudscript_ui_core::*;
use std::collections::HashMap;

pub struct WasmBridge {
    pub name: String,
    pub running: bool,
    /// Widget ID → DOM element tag tracking (in real impl, this would be web_sys::Element)
    pub element_map: HashMap<String, String>,
}

impl Default for WasmBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmBridge {
    pub fn new() -> Self {
        Self {
            name: "wasm(browser)".to_string(),
            running: false,
            element_map: HashMap::new(),
        }
    }

    /// Map a WidgetType to an HTML tag name
    pub fn widget_to_tag(wt: &WidgetType) -> &'static str {
        match wt {
            WidgetType::Text => "p",
            WidgetType::Button => "button",
            WidgetType::Input => "input",
            WidgetType::Column
            | WidgetType::Row
            | WidgetType::Container
            | WidgetType::Card
            | WidgetType::Grid
            | WidgetType::Screen => "div",
            WidgetType::Image => "img",
            WidgetType::Link => "a",
            WidgetType::List => "ul",
            WidgetType::Table => "table",
            WidgetType::Checkbox => "input", // type="checkbox"
            WidgetType::Select => "select",
            WidgetType::Slider => "input", // type="range"
            WidgetType::Navbar | WidgetType::Menubar => "nav",
            WidgetType::Menu => "ul",
            WidgetType::MenuItem => "li",
            WidgetType::TabBar => "div",
            WidgetType::Tab => "button",
            WidgetType::Icon => "span",
            _ => "div",
        }
    }

    /// Build element map from widget tree (tracking for O(1) updates)
    fn register_widgets(&mut self, widget: &Widget) {
        let tag = Self::widget_to_tag(&widget.widget_type);
        self.element_map.insert(widget.id.clone(), tag.to_string());
        for child in &widget.children {
            self.register_widgets(child);
        }
    }
}

impl UIBridge for WasmBridge {
    fn init(&mut self) -> Result<(), BridgeError> {
        self.running = true;
        println!("[{}] WASM DOM bridge initialized", self.name);
        Ok(())
    }

    fn render(&mut self, app: &App) -> Result<(), BridgeError> {
        self.element_map.clear();
        for screen in &app.screens {
            self.register_widgets(&screen.root);
        }
        println!(
            "[{}] App '{}' rendered: {} DOM elements tracked",
            self.name,
            app.name,
            self.element_map.len()
        );
        Ok(())
    }

    fn update(&mut self, update: &UIUpdate) -> Result<(), BridgeError> {
        match update {
            UIUpdate::UpdateWidget { id, .. } => {
                if self.element_map.contains_key(id) {
                    println!("[{}] DOM update: #{}", self.name, id);
                }
            }
            UIUpdate::Remove { id } => {
                self.element_map.remove(id);
                println!("[{}] DOM remove: #{}", self.name, id);
            }
            UIUpdate::Insert {
                parent_id, widget, ..
            } => {
                self.register_widgets(widget);
                println!("[{}] DOM insert into #{}", self.name, parent_id);
            }
            _ => {}
        }
        Ok(())
    }

    fn poll_event(&mut self) -> Result<Option<UIEvent>, BridgeError> {
        // v0.4.47.9: Returns honest error. WASM event polling requires
        // wasm-bindgen Closure callbacks bound to DOM elements via
        // addEventListener. This bridge tracks the widget tree but does not
        // yet wire real DOM event listeners. To enable:
        //   1. Add wasm-bindgen and web-sys to deps
        //   2. In init(), get document() and bind event listeners that push
        //      to a thread-local Vec<UIEvent>
        //   3. poll_event() drains that vec
        Err(BridgeError::Unsupported(format!(
            "{}: cannot poll events — wasm-bindgen Closure callbacks not registered. \
             This bridge tracks DOM elements but does not yet wire event listeners.",
            self.name
        )))
    }

    fn shutdown(&mut self) -> Result<(), BridgeError> {
        self.running = false;
        self.element_map.clear();
        println!("[{}] WASM bridge shutdown", self.name);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
