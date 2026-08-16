//! HudHudScript UI Core — platform-agnostic widget tree, events, and protocol
//!
//! This crate defines the intermediate representation for UI:
//! - Widget tree (text, button, input, column, row, list, card, grid, chart, etc.)
//! - Event system (click, input, submit, navigate, etc.)
//! - Bridge protocol (RENDER, EVENT, UPDATE messages)
//! - Style system (size, color, padding, margin, etc.)
//!
//! GUI frameworks (GTK, Qt, Tauri, Iced, web, etc.) consume the widget tree
//! and render it using native components. HudHudScript runtime produces the tree
//! and handles events.
//!
//! # Architecture
//! ```text
//! HudHudScript (ui keyword)
//!         ↓
//!    [ui-core] → WidgetTree (this crate)
//!         ↓
//!    [ui-bridge] → Framework adapters
//!         ↓
//!    GTK / Qt / Tauri / Iced / Web / Electron
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod app;
pub mod bridge_error;
pub mod components;
pub mod event;
pub mod layout;
pub mod navigation;
pub mod state;
pub mod theme;
pub mod widget;
pub use bridge_error::BridgeError;

// ── Widget Types ────────────────────────────────────────────────────

/// All supported widget types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    // Layout
    Screen,
    Column,
    Row,
    Grid,
    Container,
    Card,

    // Content
    Text,
    Image,
    Icon,

    // Input
    Button,
    Input,
    Checkbox,
    Select,
    Slider,

    // Data
    List,
    Table,
    Chart,

    // Navigation
    Navbar,
    Menubar,
    Menu,
    MenuItem,
    Link,
    TabBar,
    Tab,

    // Platform-specific wrapper
    Platform(String),

    // Custom component reference
    Component(String),
}

/// A single widget node in the tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: String,
    pub widget_type: WidgetType,
    pub props: HashMap<String, PropValue>,
    pub events: HashMap<String, String>,
    pub children: Vec<Widget>,
    pub style: Style,
}

/// Property value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<PropValue>),
    Object(HashMap<String, PropValue>),
}

/// Display mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Display {
    Flex,
    Grid,
    Block,
    None,
}

/// Flex direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

/// Alignment on the main axis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Alignment on the cross axis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}

/// Flex wrap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

/// Overflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

/// 4-directional spacing (padding, margin, border-radius)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Edges {
    pub top: Option<f64>,
    pub right: Option<f64>,
    pub bottom: Option<f64>,
    pub left: Option<f64>,
}

impl Edges {
    /// Uniform value on all sides
    pub fn all(value: f64) -> Self {
        Self {
            top: Some(value),
            right: Some(value),
            bottom: Some(value),
            left: Some(value),
        }
    }
}

/// Style properties (#547: flexbox layout model)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Style {
    // Legacy simple fields
    pub size: Option<f64>,
    pub color: Option<String>,
    pub background: Option<String>,
    pub bold: Option<bool>,
    pub align: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,

    // Flexbox layout (#547)
    pub display: Option<Display>,
    pub flex_direction: Option<FlexDirection>,
    pub justify_content: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignItems>,
    pub flex_wrap: Option<FlexWrap>,
    pub flex_grow: Option<f64>,
    pub flex_shrink: Option<f64>,
    pub flex_basis: Option<String>,
    pub gap: Option<f64>,
    pub row_gap: Option<f64>,
    pub column_gap: Option<f64>,
    pub overflow: Option<Overflow>,

    // 4-directional spacing
    pub padding: Option<Edges>,
    pub margin: Option<Edges>,
    pub border_radius: Option<Edges>,

    // Border
    pub border_width: Option<f64>,
    pub border_color: Option<String>,
    pub border_style: Option<String>,

    // Shadow
    pub shadow: Option<String>,

    // Opacity
    pub opacity: Option<f64>,

    // Theme token reference (#548)
    pub theme_variant: Option<String>,
}

// ── Screen & App ────────────────────────────────────────────────────

/// A screen is a top-level view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screen {
    pub name: String,
    pub params: Vec<String>,
    pub root: Widget,
}

/// An app is a collection of screens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub name: String,
    pub entry_screen: String,
    pub screens: Vec<Screen>,
    pub components: Vec<ComponentDef>,
}

/// A reusable component definition (#549: lifecycle hooks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDef {
    pub name: String,
    pub props: HashMap<String, PropValue>,
    pub template: Widget,
    /// Lifecycle hook: called when component first renders
    pub on_mount: Option<String>,
    /// Lifecycle hook: called when props or state change
    pub on_update: Option<String>,
    /// Lifecycle hook: called when component is removed (cleanup)
    pub on_unmount: Option<String>,
}

// ── Events ──────────────────────────────────────────────────────────

/// Event types from GUI framework to HudHudScript runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIEvent {
    Click {
        widget_id: String,
    },
    Input {
        widget_id: String,
        value: String,
    },
    Submit {
        widget_id: String,
    },
    Navigate {
        screen: String,
        params: HashMap<String, PropValue>,
    },
    Select {
        widget_id: String,
        value: String,
    },
    Custom {
        name: String,
        data: HashMap<String, PropValue>,
    },
}

/// Update command from HudHudScript runtime to GUI framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIUpdate {
    /// Re-render entire widget tree
    Render(Widget),
    /// Update a single widget's properties
    UpdateWidget {
        id: String,
        props: HashMap<String, PropValue>,
    },
    /// Update a single widget's style
    UpdateStyle { id: String, style: Style },
    /// Remove a widget
    Remove { id: String },
    /// Insert a new widget
    Insert {
        parent_id: String,
        index: usize,
        widget: Widget,
    },
    /// Navigate to a screen
    Navigate {
        screen: String,
        params: HashMap<String, PropValue>,
    },
}

// ── Bridge Protocol ─────────────────────────────────────────────────

/// Messages between HudHudScript runtime and GUI framework
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum BridgeMessage {
    /// Runtime → Framework: render this tree
    Render(Box<App>),
    /// Framework → Runtime: user interaction
    Event(UIEvent),
    /// Runtime → Framework: partial update
    Update(UIUpdate),
    /// Runtime → Framework: show error
    Error(String),
    /// Framework → Runtime: ready signal
    Ready,
    /// Either direction: shutdown
    Shutdown,
}

// ── Bridge Trait ─────────────────────────────────────────────────────

/// Trait that GUI framework adapters must implement
pub trait UIBridge: Send {
    /// Initialize the bridge
    fn init(&mut self) -> Result<(), BridgeError>;

    /// Render the full app
    fn render(&mut self, app: &App) -> Result<(), BridgeError>;

    /// Send a partial update
    fn update(&mut self, update: &UIUpdate) -> Result<(), BridgeError>;

    /// Poll for next event (non-blocking)
    fn poll_event(&mut self) -> Result<Option<UIEvent>, BridgeError>;

    /// Shutdown the bridge
    fn shutdown(&mut self) -> Result<(), BridgeError>;

    /// Get bridge name
    fn name(&self) -> &str;
}
