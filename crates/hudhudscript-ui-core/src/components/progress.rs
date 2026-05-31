use crate::event::{Event, EventResult, KeyCode, KeyEvent};
use crate::widget::{Alignment, Color, Margin, Rect, RenderCommand, TextStyle, TuiWidget};
use serde::{Deserialize, Serialize};

// ── ProgressWidget ──────────────────────────────────────────────────

/// A progress bar widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressWidget {
    /// Progress value from 0.0 to 1.0
    pub progress: f64,
    pub label: Option<String>,
    pub filled_char: char,
    pub empty_char: char,
    pub style: TextStyle,
    pub filled_style: TextStyle,
    pub show_percentage: bool,
}

impl ProgressWidget {
    pub fn new(progress: f64) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            label: None,
            filled_char: '\u{2588}', // █
            empty_char: '\u{2591}',  // ░
            style: TextStyle::default(),
            filled_style: TextStyle::default().fg(Color::Green),
            show_percentage: true,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Set progress value (clamped to 0.0..=1.0)
    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// Get progress as percentage (0..100)
    pub fn percentage(&self) -> u16 {
        (self.progress * 100.0) as u16
    }
}

impl TuiWidget for ProgressWidget {
    fn render(&self, area: Rect) -> Vec<RenderCommand> {
        if area.is_empty() {
            return vec![];
        }

        let mut commands = Vec::new();

        // Optional label on first line
        let bar_y = if let Some(label) = &self.label {
            if area.height > 1 {
                commands.push(RenderCommand::DrawText {
                    x: area.x,
                    y: area.y,
                    text: label.clone(),
                    style: self.style.clone(),
                });
                area.y + 1
            } else {
                area.y
            }
        } else {
            area.y
        };

        // Percentage suffix
        let pct_text = if self.show_percentage {
            format!(" {}%", self.percentage())
        } else {
            String::new()
        };

        let bar_width = area.width.saturating_sub(pct_text.len() as u16) as usize;
        let filled_count = ((bar_width as f64) * self.progress) as usize;
        let empty_count = bar_width.saturating_sub(filled_count);

        let bar: String = std::iter::repeat_n(self.filled_char, filled_count)
            .chain(std::iter::repeat_n(self.empty_char, empty_count))
            .collect();

        commands.push(RenderCommand::DrawText {
            x: area.x,
            y: bar_y,
            text: bar,
            style: self.filled_style.clone(),
        });

        if !pct_text.is_empty() {
            commands.push(RenderCommand::DrawText {
                x: area.x + bar_width as u16,
                y: bar_y,
                text: pct_text,
                style: self.style.clone(),
            });
        }

        commands
    }

    fn handle_event(&mut self, _event: &Event) -> EventResult {
        EventResult::Ignored
    }
}

impl ProgressWidget {
    /// Delegate to [`TuiWidget::render`] — available without importing the trait.
    pub fn render(&self, area: Rect) -> Vec<RenderCommand> {
        <Self as TuiWidget>::render(self, area)
    }
    /// Delegate to [`TuiWidget::handle_event`] — available without importing the trait.
    pub fn handle_event(&mut self, event: &Event) -> EventResult {
        <Self as TuiWidget>::handle_event(self, event)
    }
}
