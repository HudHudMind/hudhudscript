//! Template filters — Jinja2-compatible filter functions.
//!
//! Built-in filters: raw, upper, lower, length, default, join, date.
//! Each filter returns a Value16 (string for text filters, number for length).

use hudhudscript_bytecode::Value16;

/// Apply a named filter to a value. Returns Value16 for correct type propagation.
pub fn apply_filter(name: &str, value: &Value16, args: &[String]) -> Value16 {
    match name {
        "raw" => Value16::string(filter_raw(value)),
        "upper" => Value16::string(filter_upper(value)),
        "lower" => Value16::string(filter_lower(value)),
        "length" => filter_length_value(value),
        "default" => Value16::string(filter_default(value, args)),
        "join" => Value16::string(filter_join(value, args)),
        "date" => Value16::string(filter_date(value, args)),
        _ => Value16::string(value_to_display(value)),
    }
}

/// Convert a Value16 to a display string (auto-escaped).
pub fn value_to_display(value: &Value16) -> String {
    if value.is_null() {
        return String::new();
    }
    if let Some(b) = value.as_bool() {
        return b.to_string();
    }
    if let Some(n) = value.as_number() {
        return format_number(n);
    }
    if let Some(i) = value.as_int() {
        return i.to_string();
    }
    if let Some(s) = value.as_str() {
        return html_escape(s);
    }
    if let Some(arr) = value.as_array() {
        let items: Vec<String> = arr.iter().map(|v| value_to_display(v)).collect();
        return format!("[{}]", items.join(", "));
    }
    if let Some(obj) = value.as_object() {
        let items: Vec<String> = obj
            .iter()
            .map(|(k, v)| format!("{}: {}", k, value_to_display(v)))
            .collect();
        return format!("{{{}}}", items.join(", "));
    }
    value.display_string()
}

/// Auto HTML-escape: & < > " ' → entities.
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

// ── filters ──────────────────────────────────────────────────────────

fn filter_raw(value: &Value16) -> String {
    // No escaping
    if let Some(s) = value.as_str() {
        s.to_string()
    } else {
        value_to_display(value)
    }
}

fn filter_upper(value: &Value16) -> String {
    let s = value_to_display_raw(value);
    s.to_uppercase()
}

fn filter_lower(value: &Value16) -> String {
    let s = value_to_display_raw(value);
    s.to_lowercase()
}

fn filter_length(value: &Value16) -> String {
    if let Some(arr) = value.as_array() {
        arr.len().to_string()
    } else if let Some(s) = value.as_str() {
        s.chars().count().to_string()
    } else {
        "0".to_string()
    }
}

/// Length filter returning a numeric Value16 (for comparisons like `> 0`).
fn filter_length_value(value: &Value16) -> Value16 {
    if let Some(arr) = value.as_array() {
        Value16::number(arr.len() as f64)
    } else if value.as_str().is_some() {
        Value16::number(value.as_str().unwrap().chars().count() as f64)
    } else {
        Value16::number(0.0)
    }
}

fn filter_default(value: &Value16, args: &[String]) -> String {
    if value.is_null() || (value.as_str().map(|s| s.is_empty()).unwrap_or(false)) {
        args.first().cloned().unwrap_or_default()
    } else {
        value_to_display(value)
    }
}

fn filter_join(value: &Value16, args: &[String]) -> String {
    let sep = args.first().map(|s| s.as_str()).unwrap_or(", ");
    if let Some(arr) = value.as_array() {
        let items: Vec<String> = arr.iter().map(|v| value_to_display_raw(v)).collect();
        items.join(sep)
    } else {
        value_to_display(value)
    }
}

fn filter_date(value: &Value16, _args: &[String]) -> String {
    if let Some(n) = value.as_number() {
        let secs = n as i64;
        // Simple ISO format from unix timestamp
        format_unix_timestamp(secs)
    } else if let Some(s) = value.as_str() {
        s.to_string()
    } else {
        String::new()
    }
}

// ── helpers ───────────────────────────────────────────────────────────

fn value_to_display_raw(value: &Value16) -> String {
    if let Some(s) = value.as_str() {
        s.to_string()
    } else if let Some(n) = value.as_number() {
        format_number(n)
    } else {
        value_to_display(value)
    }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

fn format_unix_timestamp(secs: i64) -> String {
    // Simple YYYY-MM-DD HH:MM:SS format
    if secs <= 0 {
        return String::new();
    }
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let secs_rem = time_of_day % 60;

    // Approximate date from days (not perfect but good enough for templates)
    let mut y = 1970;
    let mut d = days_since_epoch;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let (month, day) = month_day_from_doy(y, d as u32);

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y,
        month,
        day,
        hours,
        minutes,
        secs_rem
    )
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

fn month_day_from_doy(y: i64, doy: u32) -> (u32, u32) {
    let days_in_month = [
        31,
        if is_leap(y) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut remaining = doy;
    for (m, &dim) in days_in_month.iter().enumerate() {
        if remaining < dim {
            return (m as u32 + 1, remaining + 1);
        }
        remaining -= dim;
    }
    (12, 31)
}

