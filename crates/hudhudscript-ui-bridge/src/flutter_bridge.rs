//! Flutter mobile adapter (#553)
//!
//! Serializes widget tree as JSON for consumption by a Flutter app via FFI.
//! The Flutter side deserializes and maps to native Flutter widgets.

use hudhudscript_ui_core::*;

pub struct FlutterBridge {
    pub name: String,
    pub running: bool,
}

impl Default for FlutterBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl FlutterBridge {
    pub fn new() -> Self {
        Self {
            name: "flutter(mobile)".to_string(),
            running: false,
        }
    }

    /// Generate a Flutter widget mapping hint for a WidgetType
    pub fn flutter_widget_name(wt: &WidgetType) -> &'static str {
        match wt {
            WidgetType::Column => "Column",
            WidgetType::Row => "Row",
            WidgetType::Text => "Text",
            WidgetType::Button => "ElevatedButton",
            WidgetType::Input => "TextField",
            WidgetType::Image => "Image",
            WidgetType::Icon => "Icon",
            WidgetType::Card => "Card",
            WidgetType::Container => "Container",
            WidgetType::Checkbox => "Checkbox",
            WidgetType::Select => "DropdownButton",
            WidgetType::Slider => "Slider",
            WidgetType::List => "ListView",
            WidgetType::Grid => "GridView",
            WidgetType::Table => "DataTable",
            WidgetType::TabBar => "TabBar",
            WidgetType::Tab => "Tab",
            WidgetType::Navbar => "AppBar",
            WidgetType::Link => "TextButton",
            _ => "Container",
        }
    }
}

impl UIBridge for FlutterBridge {
    fn init(&mut self) -> Result<(), BridgeError> {
        self.running = true;
        println!("[{}] FFI bridge initialized", self.name);
        Ok(())
    }

    fn render(&mut self, app: &App) -> Result<(), BridgeError> {
        let json =
            serde_json::to_string(app).map_err(|e| BridgeError::RenderFailed(e.to_string()))?;
        println!(
            "[{}] App '{}' serialized for Flutter ({} bytes)",
            self.name,
            app.name,
            json.len()
        );
        Ok(())
    }

    fn update(&mut self, update: &UIUpdate) -> Result<(), BridgeError> {
        let json =
            serde_json::to_string(update).map_err(|e| BridgeError::RenderFailed(e.to_string()))?;
        println!("[{}] FFI update: {} bytes", self.name, json.len());
        Ok(())
    }

    fn poll_event(&mut self) -> Result<Option<UIEvent>, BridgeError> {
        Ok(None)
    }

    fn shutdown(&mut self) -> Result<(), BridgeError> {
        self.running = false;
        println!("[{}] FFI bridge shutdown", self.name);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
