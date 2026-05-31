use crate::event::{Event, EventResult, KeyCode, KeyEvent};
use crate::widget::{Alignment, Color, Margin, Rect, RenderCommand, TextStyle, TuiWidget};
use serde::{Deserialize, Serialize};

// ── ListWidget ──────────────────────────────────────────────────────

/// A scrollable list with selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWidget {
    pub items: Vec<String>,
    pub selected: Option<usize>,
    pub scroll_offset: usize,
    pub style: TextStyle,
    pub selected_style: TextStyle,
    pub margin: Margin,
}

impl ListWidget {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            selected: if items.is_empty() { None } else { Some(0) },
            items,
            scroll_offset: 0,
            style: TextStyle::default(),
            selected_style: TextStyle::default().bg(Color::Blue).fg(Color::White),
            margin: Margin::default(),
        }
    }

    /// Select the next item
    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = Some(match self.selected {
            Some(i) if i + 1 < self.items.len() => i + 1,
            Some(i) => i,
            None => 0,
        });
    }

    /// Select the previous item
    pub fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = Some(match self.selected {
            Some(0) | None => 0,
            Some(i) => i - 1,
        });
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> Option<&str> {
        self.selected
            .and_then(|i| self.items.get(i))
            .map(|s| s.as_str())
    }

    /// Ensure the selected item is visible by adjusting scroll offset
    pub fn ensure_visible(&mut self, visible_height: usize) {
        if let Some(sel) = self.selected {
            if sel < self.scroll_offset {
                self.scroll_offset = sel;
            } else if sel >= self.scroll_offset + visible_height {
                self.scroll_offset = sel.saturating_sub(visible_height.saturating_sub(1));
            }
        }
    }
}

impl TuiWidget for ListWidget {
    fn render(&self, area: Rect) -> Vec<RenderCommand> {
        if area.is_empty() {
            return vec![];
        }

        let inner = area.inner(&self.margin);
        let mut commands = Vec::new();
        let visible_height = inner.height as usize;

        // Clone to compute scroll (render is &self)
        let scroll = if let Some(sel) = self.selected {
            if sel < self.scroll_offset {
                sel
            } else if sel >= self.scroll_offset + visible_height {
                sel.saturating_sub(visible_height.saturating_sub(1))
            } else {
                self.scroll_offset
            }
        } else {
            self.scroll_offset
        };

        for (vi, idx) in (scroll..self.items.len()).enumerate() {
            if vi >= visible_height {
                break;
            }

            let item = &self.items[idx];
            let is_selected = self.selected == Some(idx);
            let style = if is_selected {
                self.selected_style.clone()
            } else {
                self.style.clone()
            };

            let display = if item.len() > inner.width as usize {
                &item[..inner.width as usize]
            } else {
                item.as_str()
            };

            // If selected, fill the entire line background
            if is_selected {
                commands.push(RenderCommand::Fill {
                    rect: Rect::new(inner.x, inner.y + vi as u16, inner.width, 1),
                    ch: ' ',
                    style: style.clone(),
                });
            }

            commands.push(RenderCommand::DrawText {
                x: inner.x,
                y: inner.y + vi as u16,
                text: display.to_string(),
                style,
            });
        }

        commands
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Up => {
                    self.select_previous();
                    EventResult::Consumed
                }
                KeyCode::Down => {
                    self.select_next();
                    EventResult::Consumed
                }
                KeyCode::Enter => {
                    if let Some(item) = self.selected_item() {
                        EventResult::Action(format!("select:{}", item))
                    } else {
                        EventResult::Ignored
                    }
                }
                _ => EventResult::Ignored,
            }
        } else {
            EventResult::Ignored
        }
    }
}

impl ListWidget {
    /// Delegate to [`TuiWidget::render`] — available without importing the trait.
    pub fn render(&self, area: Rect) -> Vec<RenderCommand> {
        <Self as TuiWidget>::render(self, area)
    }
    /// Delegate to [`TuiWidget::handle_event`] — available without importing the trait.
    pub fn handle_event(&mut self, event: &Event) -> EventResult {
        <Self as TuiWidget>::handle_event(self, event)
    }
}
