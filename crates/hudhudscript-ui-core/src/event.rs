//! Event system for TUI framework (#601)
//!
//! Keyboard, mouse, resize, and tick events for terminal UI applications.

use serde::{Deserialize, Serialize};

/// Top-level event types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    /// Keyboard event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize: (width, height)
    Resize(u16, u16),
    /// Periodic tick for animations/polling
    Tick,
}

/// A keyboard event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: Modifiers,
}

impl KeyEvent {
    /// Create a simple key event with no modifiers
    pub fn new(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: Modifiers::empty(),
        }
    }

    /// Create a key event with modifiers
    pub fn with_modifiers(code: KeyCode, modifiers: Modifiers) -> Self {
        Self { code, modifiers }
    }
}

/// Key codes for keyboard events
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    Char(char),
    Enter,
    Esc,
    Tab,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    F(u8),
    Null,
}

/// Modifier keys (bitflags-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Modifiers {
    bits: u8,
}

impl Modifiers {
    pub const SHIFT: u8 = 0b001;
    pub const CTRL: u8 = 0b010;
    pub const ALT: u8 = 0b100;

    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn from_bits(bits: u8) -> Self {
        Self { bits }
    }

    pub const fn shift() -> Self {
        Self { bits: Self::SHIFT }
    }

    pub const fn ctrl() -> Self {
        Self { bits: Self::CTRL }
    }

    pub const fn alt() -> Self {
        Self { bits: Self::ALT }
    }

    pub const fn contains(&self, flag: u8) -> bool {
        self.bits & flag != 0
    }

    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }
}

/// A mouse event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
}

/// Mouse event kinds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseEventKind {
    Click,
    DoubleClick,
    ScrollUp,
    ScrollDown,
    Move,
    Drag,
    Release,
}

/// Mouse buttons
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    None,
}

/// Result of handling an event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventResult {
    /// Event was consumed by the handler
    Consumed,
    /// Event was not handled — pass to next handler
    Ignored,
    /// Event triggered a named action
    Action(String),
}
