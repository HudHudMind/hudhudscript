use crate::event::{Event, EventResult, KeyCode, KeyEvent};
use crate::widget::{Alignment, Color, Margin, Rect, RenderCommand, TextStyle, TuiWidget};
use serde::{Deserialize, Serialize};

// ── TableWidget ─────────────────────────────────────────────────────

/// A columnar data table with headers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableWidget {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub column_widths: Vec<u16>,
    pub selected_row: Option<usize>,
    pub scroll_offset: usize,
    pub header_style: TextStyle,
    pub row_style: TextStyle,
    pub selected_style: TextStyle,
}

impl TableWidget {
    pub fn new(headers: Vec<String>, column_widths: Vec<u16>) -> Self {
        Self {
            headers,
            column_widths,
            rows: Vec::new(),
            selected_row: None,
            scroll_offset: 0,
            header_style: TextStyle::default().bold(),
            row_style: TextStyle::default(),
            selected_style: TextStyle::default().bg(Color::Blue).fg(Color::White),
        }
    }

    pub fn rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.selected_row = if rows.is_empty() { None } else { Some(0) };
        self.rows = rows;
        self
    }

    /// Select next row
    pub fn select_next(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.selected_row = Some(match self.selected_row {
            Some(i) if i + 1 < self.rows.len() => i + 1,
            Some(i) => i,
            None => 0,
        });
    }

    /// Select previous row
    pub fn select_previous(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.selected_row = Some(match self.selected_row {
            Some(0) | None => 0,
            Some(i) => i - 1,
        });
    }

    /// Render a single row of cells at the given y position
    fn render_row(
        &self,
        area: &Rect,
        y: u16,
        cells: &[String],
        style: &TextStyle,
    ) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        let mut x = area.x;

        for (col_idx, cell) in cells.iter().enumerate() {
            let col_w = self.column_widths.get(col_idx).copied().unwrap_or(10);

            if x >= area.right() {
                break;
            }

            let available = (area.right() - x).min(col_w);
            let display = if cell.len() > available as usize {
                &cell[..available as usize]
            } else {
                cell.as_str()
            };

            commands.push(RenderCommand::DrawText {
                x,
                y,
                text: display.to_string(),
                style: style.clone(),
            });

            x = x.saturating_add(col_w).saturating_add(1); // +1 for column separator
        }

        commands
    }
}

impl TuiWidget for TableWidget {
    fn render(&self, area: Rect) -> Vec<RenderCommand> {
        if area.is_empty() {
            return vec![];
        }

        let mut commands = Vec::new();

        // Render header
        commands.extend(self.render_row(&area, area.y, &self.headers, &self.header_style));

        // Separator line
        if area.height > 1 {
            let sep = "\u{2500}".repeat(area.width as usize);
            commands.push(RenderCommand::DrawText {
                x: area.x,
                y: area.y + 1,
                text: sep,
                style: self.header_style.clone(),
            });
        }

        // Data rows
        let data_start_y = area.y + 2; // header + separator
        let visible_rows = (area.height.saturating_sub(2)) as usize;

        for (vi, row_idx) in (self.scroll_offset..self.rows.len()).enumerate() {
            if vi >= visible_rows {
                break;
            }

            let is_selected = self.selected_row == Some(row_idx);
            let style = if is_selected {
                &self.selected_style
            } else {
                &self.row_style
            };

            if is_selected {
                commands.push(RenderCommand::Fill {
                    rect: Rect::new(area.x, data_start_y + vi as u16, area.width, 1),
                    ch: ' ',
                    style: style.clone(),
                });
            }

            commands.extend(self.render_row(
                &area,
                data_start_y + vi as u16,
                &self.rows[row_idx],
                style,
            ));
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
                    if let Some(idx) = self.selected_row {
                        EventResult::Action(format!("select_row:{}", idx))
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

impl TableWidget {
    /// Delegate to [`TuiWidget::render`] — available without importing the trait.
    pub fn render(&self, area: Rect) -> Vec<RenderCommand> {
        <Self as TuiWidget>::render(self, area)
    }
    /// Delegate to [`TuiWidget::handle_event`] — available without importing the trait.
    pub fn handle_event(&mut self, event: &Event) -> EventResult {
        <Self as TuiWidget>::handle_event(self, event)
    }
}
