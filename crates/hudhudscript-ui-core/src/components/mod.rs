//! Built-in TUI components (#601)
//!
//! Ready-to-use widget implementations: text display, text input,
//! scrollable list, data table, and progress bar.

use crate::event::{Event, EventResult, KeyCode, KeyEvent};
use crate::widget::{Alignment, Color, Margin, Rect, RenderCommand, TextStyle, TuiWidget};
use serde::{Deserialize, Serialize};

pub mod input;
pub mod list;
pub mod progress;
pub mod table;
pub mod text;
