use crate::event::{Event, EventResult, KeyCode, KeyEvent};
use crate::widget::{Alignment, Color, Margin, Rect, RenderCommand, TextStyle, TuiWidget};
use serde::{Deserialize, Serialize};

// ── InputWidget ─────────────────────────────────────────────────────

/// A text input widget with cursor and editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputWidget {
    pub value: String,
    pub cursor: usize,
    pub hint_text: String,
    pub style: TextStyle,
    pub cursor_style: TextStyle,
    /// Horizontal scroll offset for long inputs
    pub scroll_offset: usize,
}

impl InputWidget {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            hint_text: String::new(),
            style: TextStyle::default(),
            cursor_style: TextStyle::default().bg(Color::White).fg(Color::Black),
            scroll_offset: 0,
        }
    }

    pub fn hint_text(mut self, hint_text: impl Into<String>) -> Self {
        self.hint_text = hint_text.into();
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        let v: String = value.into();
        self.cursor = v.len();
        self.value = v;
        self
    }

    /// Insert a character at the cursor position
    pub fn insert_char(&mut self, ch: char) {
        self.value.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    /// Delete the character before the cursor (backspace)
    pub fn delete_backward(&mut self) {
        if self.cursor > 0 {
            // Find the previous char boundary
            let prev = self.value[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.value.remove(prev);
            self.cursor = prev;
        }
    }

    /// Delete the character at the cursor position
    pub fn delete_forward(&mut self) {
        if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.value[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        if self.cursor < self.value.len() {
            self.cursor = self.value[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.value.len());
        }
    }

    /// Move cursor to start
    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end
    pub fn move_end(&mut self) {
        self.cursor = self.value.len();
    }
}

impl Default for InputWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiWidget for InputWidget {
    fn render(&self, area: Rect) -> Vec<RenderCommand> {
        if area.is_empty() {
            return vec![];
        }

        let mut commands = Vec::new();

        // Clear the input area
        commands.push(RenderCommand::Clear { rect: area });

        let display = if self.value.is_empty() {
            &self.hint_text
        } else {
            &self.value
        };

        let visible_width = area.width as usize;
        let start = self.scroll_offset;
        let end = (start + visible_width).min(display.len());
        let visible = if start < display.len() {
            &display[start..end]
        } else {
            ""
        };

        let text_style = if self.value.is_empty() {
            // Dim style for hint_text
            TextStyle {
                dim: true,
                ..self.style.clone()
            }
        } else {
            self.style.clone()
        };

        commands.push(RenderCommand::DrawText {
            x: area.x,
            y: area.y,
            text: visible.to_string(),
            style: text_style,
        });

        // Draw cursor
        let cursor_x = area.x + (self.cursor.saturating_sub(self.scroll_offset)) as u16;
        if cursor_x < area.right() {
            commands.push(RenderCommand::MoveCursor {
                x: cursor_x,
                y: area.y,
            });
        }

        commands
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char(ch) => {
                    self.insert_char(*ch);
                    EventResult::Consumed
                }
                KeyCode::Backspace => {
                    self.delete_backward();
                    EventResult::Consumed
                }
                KeyCode::Delete => {
                    self.delete_forward();
                    EventResult::Consumed
                }
                KeyCode::Left => {
                    self.move_left();
                    EventResult::Consumed
                }
                KeyCode::Right => {
                    self.move_right();
                    EventResult::Consumed
                }
                KeyCode::Home => {
                    self.move_home();
                    EventResult::Consumed
                }
                KeyCode::End => {
                    self.move_end();
                    EventResult::Consumed
                }
                KeyCode::Enter => EventResult::Action("submit".to_string()),
                _ => EventResult::Ignored,
            }
        } else {
            EventResult::Ignored
        }
    }
}

impl InputWidget {
    /// Delegate to [`TuiWidget::render`] — available without importing the trait.
    pub fn render(&self, area: Rect) -> Vec<RenderCommand> {
        <Self as TuiWidget>::render(self, area)
    }
    /// Delegate to [`TuiWidget::handle_event`] — available without importing the trait.
    pub fn handle_event(&mut self, event: &Event) -> EventResult {
        <Self as TuiWidget>::handle_event(self, event)
    }
}
