//! Value16 formatting utilities.

/// Shared number formatting: displays integers without decimal point,
/// floats normally. Replaces duplicated logic across json, csv, ini,
/// value_to_string, etc.
pub fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}
