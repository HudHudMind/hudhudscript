//! AST Visitor control flow enum.

/// Control flow for visitors — continue walking, stop early, or skip children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisitControl {
    /// Continue walking into child nodes.
    Continue,
    /// Stop the entire traversal immediately.
    Stop,
    /// Skip children of the current node but continue with siblings.
    SkipChildren,
}
