//! Widget system for TUI framework (#601)
//!
//! Abstract widget trait, rect geometry, and render commands.
//! This is a platform-agnostic rendering abstraction that does not
//! depend on any specific TUI library (ratatui, crossterm, etc.).

use crate::event::{Event, EventResult};
use serde::{Deserialize, Serialize};

/// A rectangular area on the terminal screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Right edge (exclusive)
    pub const fn right(&self) -> u16 {
        self.x.saturating_add(self.width)
    }

    /// Bottom edge (exclusive)
    pub const fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }

    /// Total area in cells
    pub const fn area(&self) -> u32 {
        self.width as u32 * self.height as u32
    }

    /// Whether this rect has zero area
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Shrink the rect by the given margin on each side
    pub fn inner(&self, margin: &Margin) -> Rect {
        let x = self.x.saturating_add(margin.left);
        let y = self.y.saturating_add(margin.top);
        let w = self
            .width
            .saturating_sub(margin.left)
            .saturating_sub(margin.right);
        let h = self
            .height
            .saturating_sub(margin.top)
            .saturating_sub(margin.bottom);
        Rect::new(x, y, w, h)
    }
}

/// Margin around a rect (in terminal cells)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Margin {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl Margin {
    pub const fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Uniform margin on all sides
    pub const fn all(value: u16) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Vertical and horizontal margins
    pub const fn symmetric(vertical: u16, horizontal: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

/// Text color (simple ANSI-compatible)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    /// 256-color index
    Indexed(u8),
    /// RGB color
    Rgb(u8, u8, u8),
}

/// Text styling attributes
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TextStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
}

impl TextStyle {
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

/// Low-level render commands produced by widgets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderCommand {
    /// Draw text at a position with styling
    DrawText {
        x: u16,
        y: u16,
        text: String,
        style: TextStyle,
    },
    /// Draw a box (border) around an area
    DrawBox { rect: Rect, style: TextStyle },
    /// Set foreground/background color for subsequent operations
    SetColor {
        fg: Option<Color>,
        bg: Option<Color>,
    },
    /// Clear a rectangular area
    Clear { rect: Rect },
    /// Move the cursor to a position
    MoveCursor { x: u16, y: u16 },
    /// Fill a rect with a character
    Fill {
        rect: Rect,
        ch: char,
        style: TextStyle,
    },
}

/// The core widget trait for TUI components.
///
/// Widgets produce render commands for a given area and can handle events.
/// This is intentionally abstract and does not depend on any TUI framework.
pub trait TuiWidget {
    /// Render this widget into the given area, producing draw commands
    fn render(&self, area: Rect) -> Vec<RenderCommand>;

    /// Handle an event, returning how it was processed
    fn handle_event(&mut self, event: &Event) -> EventResult;
}
