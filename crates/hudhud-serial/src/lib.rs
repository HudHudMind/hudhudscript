//! HudHud serialization primitives (no builtins dependency).
pub mod csv_ops;
pub mod ini_ops;
pub mod toml_ops;
pub mod yaml_ops;

pub fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}
