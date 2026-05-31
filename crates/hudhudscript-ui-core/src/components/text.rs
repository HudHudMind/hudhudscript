use crate::event::{Event, EventResult, KeyCode, KeyEvent};
use crate::widget::{Alignment, Color, Margin, Rect, RenderCommand, TextStyle, TuiWidget};
use serde::{Deserialize, Serialize};

// ── TextWidget ──────────────────────────────────────────────────────

/// A styled text display widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextWidget {
    pub text: String,
    pub style: TextStyle,
    pub alignment: Alignment,
}

impl TextWidget {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::default(),
            alignment: Alignment::Left,
        }
    }

    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl TuiWidget for TextWidget {
    fn render(&self, area: Rect) -> Vec<RenderCommand> {
        if area.is_empty() {
            return vec![];
        }

        let mut commands = Vec::new();
        let lines: Vec<&str> = self.text.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if i as u16 >= area.height {
                break;
            }

            let display_text = if line.len() > area.width as usize {
                &line[..area.width as usize]
            } else {
                line
            };

            let x = match self.alignment {
                Alignment::Left => area.x,
                Alignment::Center => {
                    area.x + (area.width.saturating_sub(display_text.len() as u16)) / 2
                }
                Alignment::Right => area.x + area.width.saturating_sub(display_text.len() as u16),
            };

            commands.push(RenderCommand::DrawText {
                x,
                y: area.y + i as u16,
                text: display_text.to_string(),
                style: self.style.clone(),
            });
        }

        commands
    }

    fn handle_event(&mut self, _event: &Event) -> EventResult {
        EventResult::Ignored
    }
}

impl TextWidget {
    /// Delegate to [`TuiWidget::render`] — available without importing the trait.
    pub fn render(&self, area: Rect) -> Vec<RenderCommand> {
        <Self as TuiWidget>::render(self, area)
    }
    /// Delegate to [`TuiWidget::handle_event`] — available without importing the trait.
    pub fn handle_event(&mut self, event: &Event) -> EventResult {
        <Self as TuiWidget>::handle_event(self, event)
    }
}
