//! Application framework for TUI (#601)
//!
//! Provides the `TuiApp` trait and `WidgetTree` for composing widgets
//! into a hierarchical UI that can be rendered and updated in a loop.

use crate::event::{Event, EventResult};
use crate::widget::{Rect, RenderCommand, TuiWidget};
use serde::{Deserialize, Serialize};

/// A node in the widget tree, allowing hierarchical composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetTree {
    /// An identifier for this node
    pub id: String,
    /// Render commands produced by this node's widget
    pub commands: Vec<RenderCommand>,
    /// Child nodes
    pub children: Vec<WidgetTree>,
}

impl WidgetTree {
    /// Create a new widget tree node
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            commands: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Create a leaf node from a widget rendered in the given area
    pub fn leaf(id: impl Into<String>, widget: &dyn TuiWidget, area: Rect) -> Self {
        Self {
            id: id.into(),
            commands: widget.render(area),
            children: Vec::new(),
        }
    }

    /// Add a child node
    pub fn add_child(mut self, child: WidgetTree) -> Self {
        self.children.push(child);
        self
    }

    /// Add render commands
    pub fn with_commands(mut self, commands: Vec<RenderCommand>) -> Self {
        self.commands = commands;
        self
    }

    /// Collect all render commands in depth-first order
    pub fn flatten(&self) -> Vec<RenderCommand> {
        let mut result = self.commands.clone();
        for child in &self.children {
            result.extend(child.flatten());
        }
        result
    }

    /// Count total nodes in the tree
    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.node_count()).sum::<usize>()
    }

    /// Find a node by ID (depth-first)
    pub fn find(&self, id: &str) -> Option<&WidgetTree> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find(id) {
                return Some(found);
            }
        }
        None
    }
}

/// The core application trait for TUI programs.
///
/// Implement this trait to define your terminal application's behavior.
/// The framework calls these methods in a loop:
/// 1. `view()` to get the current widget tree
/// 2. `update()` when an event arrives
/// 3. `should_quit()` to check if the app should exit
pub trait TuiApp {
    /// Initialize the application state
    fn init(&mut self);

    /// Handle an incoming event, updating application state
    fn update(&mut self, event: Event) -> EventResult;

    /// Build the current widget tree for rendering
    fn view(&self) -> WidgetTree;

    /// Whether the application should exit the main loop
    fn should_quit(&self) -> bool;

    /// Optional: called on each tick (for animations, polling, etc.)
    fn on_tick(&mut self) {}
}
