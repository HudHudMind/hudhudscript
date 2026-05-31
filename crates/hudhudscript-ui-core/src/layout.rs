//! Layout engine for TUI framework (#601)
//!
//! Splits a rectangular area into sub-areas based on constraints.
//! Supports horizontal and vertical layouts with fixed, percentage,
//! min, max, and fill constraints.

use crate::widget::Rect;
use serde::{Deserialize, Serialize};

/// Layout direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Direction {
    /// Split area horizontally (left to right)
    Horizontal,
    /// Split area vertically (top to bottom)
    #[default]
    Vertical,
}

/// Layout constraint for a single segment
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Constraint {
    /// Fixed size in cells
    Fixed(u16),
    /// Percentage of available space (0-100)
    Percentage(u16),
    /// Minimum size (at least this many cells)
    Min(u16),
    /// Maximum size (at most this many cells)
    Max(u16),
    /// Fill remaining space (distributed equally among all Fill constraints)
    Fill,
}

/// Layout engine that splits a rect into sub-rects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    direction: Direction,
    constraints: Vec<Constraint>,
}

impl Layout {
    /// Create a new layout with default vertical direction
    pub fn new() -> Self {
        Self {
            direction: Direction::Vertical,
            constraints: Vec::new(),
        }
    }

    /// Set the direction
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Set the constraints
    pub fn constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.constraints = constraints;
        self
    }

    /// Get the direction
    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    /// Get the constraints
    pub fn get_constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    /// Split the given area according to the constraints
    pub fn split(&self, area: Rect) -> Vec<Rect> {
        if self.constraints.is_empty() {
            return vec![area];
        }

        let total = match self.direction {
            Direction::Horizontal => area.width,
            Direction::Vertical => area.height,
        };

        let sizes = self.resolve_constraints(total);

        let mut rects = Vec::with_capacity(sizes.len());
        let mut offset: u16 = 0;

        for size in &sizes {
            let rect = match self.direction {
                Direction::Horizontal => {
                    Rect::new(area.x.saturating_add(offset), area.y, *size, area.height)
                }
                Direction::Vertical => {
                    Rect::new(area.x, area.y.saturating_add(offset), area.width, *size)
                }
            };
            rects.push(rect);
            offset = offset.saturating_add(*size);
        }

        rects
    }

    /// Resolve constraints into actual sizes for the given total space
    fn resolve_constraints(&self, total: u16) -> Vec<u16> {
        let n = self.constraints.len();
        let mut sizes = vec![0u16; n];
        let mut remaining = total;
        let mut fill_count: u16 = 0;

        // First pass: allocate Fixed and Percentage
        for (i, c) in self.constraints.iter().enumerate() {
            match c {
                Constraint::Fixed(v) => {
                    let v = (*v).min(remaining);
                    sizes[i] = v;
                    remaining = remaining.saturating_sub(v);
                }
                Constraint::Percentage(p) => {
                    let v = ((total as u32 * (*p).min(100) as u32) / 100) as u16;
                    let v = v.min(remaining);
                    sizes[i] = v;
                    remaining = remaining.saturating_sub(v);
                }
                Constraint::Min(_) | Constraint::Max(_) | Constraint::Fill => {
                    // handled in second pass
                }
            }
        }

        // Second pass: allocate Min/Max, count Fill
        for (i, c) in self.constraints.iter().enumerate() {
            match c {
                Constraint::Min(min_val) => {
                    let v = (*min_val).min(remaining);
                    sizes[i] = v;
                    remaining = remaining.saturating_sub(v);
                    // Min also acts like fill for leftover
                    fill_count += 1;
                }
                Constraint::Max(max_val) => {
                    let v = remaining.min(*max_val);
                    sizes[i] = v;
                    remaining = remaining.saturating_sub(v);
                }
                Constraint::Fill => {
                    fill_count += 1;
                }
                _ => {}
            }
        }

        // Third pass: distribute remaining space among Fill and Min
        if fill_count > 0 && remaining > 0 {
            let share = remaining / fill_count;
            let mut extra = remaining % fill_count;

            for (i, c) in self.constraints.iter().enumerate() {
                match c {
                    Constraint::Fill | Constraint::Min(_) => {
                        let bonus = if extra > 0 {
                            extra -= 1;
                            1
                        } else {
                            0
                        };
                        sizes[i] = sizes[i].saturating_add(share + bonus);
                    }
                    _ => {}
                }
            }
        }

        sizes
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}
