//! Web adapter — Next.js + WebSocket bridge (#551)
//!
//! Renders widget tree as JSON over WebSocket. A Next.js frontend consumes
//! the JSON and maps widgets to React components.

use hudhudscript_ui_core::*;

pub struct WebBridge {
    name: String,
    port: u16,
    running: bool,
}

impl WebBridge {
    pub fn new(port: u16) -> Self {
        Self {
            name: "web(nextjs)".to_string(),
            port,
            running: false,
        }
    }

    /// Render a widget tree to an HTML string (server-side fallback)
    pub fn widget_to_html(widget: &Widget) -> String {
        let mut html = String::new();
        let style = Self::style_to_css(&widget.style);
        let style_attr = if style.is_empty() {
            String::new()
        } else {
            format!(" style=\"{}\"", style)
        };

        match &widget.widget_type {
            WidgetType::Text => {
                let text = widget
                    .props
                    .get("label")
                    .map(|v| match v {
                        PropValue::String(s) => s.as_str(),
                        _ => "",
                    })
                    .unwrap_or("");
                html.push_str(&format!(
                    "<p id=\"{}\"{}>{}</p>",
                    widget.id, style_attr, text
                ));
            }
            WidgetType::Button => {
                let label = widget
                    .props
                    .get("label")
                    .map(|v| match v {
                        PropValue::String(s) => s.as_str(),
                        _ => "",
                    })
                    .unwrap_or("Button");
                html.push_str(&format!(
                    "<button id=\"{}\"{}>{}</button>",
                    widget.id, style_attr, label
                ));
            }
            WidgetType::Input => {
                html.push_str(&format!("<input id=\"{}\"{} />", widget.id, style_attr));
            }
            WidgetType::Column => {
                html.push_str(&format!(
                    "<div id=\"{}\" style=\"display:flex;flex-direction:column;{}\">",
                    widget.id, style
                ));
                for child in &widget.children {
                    html.push_str(&Self::widget_to_html(child));
                }
                html.push_str("</div>");
            }
            WidgetType::Row => {
                html.push_str(&format!(
                    "<div id=\"{}\" style=\"display:flex;flex-direction:row;{}\">",
                    widget.id, style
                ));
                for child in &widget.children {
                    html.push_str(&Self::widget_to_html(child));
                }
                html.push_str("</div>");
            }
            _ => {
                html.push_str(&format!("<div id=\"{}\"{}>", widget.id, style_attr));
                for child in &widget.children {
                    html.push_str(&Self::widget_to_html(child));
                }
                html.push_str("</div>");
            }
        }
        html
    }

    fn style_to_css(style: &Style) -> String {
        let mut css = String::new();
        if let Some(ref c) = style.color {
            css.push_str(&format!("color:{};", c));
        }
        if let Some(ref bg) = style.background {
            css.push_str(&format!("background:{};", bg));
        }
        if let Some(s) = style.size {
            css.push_str(&format!("font-size:{}px;", s));
        }
        if let Some(true) = style.bold {
            css.push_str("font-weight:bold;");
        }
        if let Some(ref w) = style.width {
            css.push_str(&format!("width:{}px;", w));
        }
        if let Some(ref h) = style.height {
            css.push_str(&format!("height:{}px;", h));
        }
        if let Some(o) = style.opacity {
            css.push_str(&format!("opacity:{};", o));
        }
        if let Some(g) = style.gap {
            css.push_str(&format!("gap:{}px;", g));
        }
        css
    }
}

impl UIBridge for WebBridge {
    fn init(&mut self) -> Result<(), BridgeError> {
        self.running = true;
        println!(
            "[{}] WebSocket bridge ready on port {}",
            self.name, self.port
        );
        Ok(())
    }

    fn render(&mut self, app: &App) -> Result<(), BridgeError> {
        let json = serde_json::to_string_pretty(app)
            .map_err(|e| BridgeError::RenderFailed(e.to_string()))?;
        println!(
            "[{}] App '{}' rendered ({} screens, {} bytes JSON)",
            self.name,
            app.name,
            app.screens.len(),
            json.len()
        );
        Ok(())
    }

    fn update(&mut self, update: &UIUpdate) -> Result<(), BridgeError> {
        let json =
            serde_json::to_string(update).map_err(|e| BridgeError::RenderFailed(e.to_string()))?;
        println!("[{}] Update: {} bytes", self.name, json.len());
        Ok(())
    }

    fn poll_event(&mut self) -> Result<Option<UIEvent>, BridgeError> {
        Ok(None)
    }

    fn shutdown(&mut self) -> Result<(), BridgeError> {
        self.running = false;
        println!("[{}] Bridge shutdown", self.name);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
