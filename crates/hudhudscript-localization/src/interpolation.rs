//! String interpolation for translated messages
//!
//! Supports two forms:
//!   - Simple variable substitution: `{variable}`
//!   - Inline plural expressions: `{count, plural, one{# item} other{# items}}`

use std::collections::HashMap;

/// Interpolate variables into a template string.
///
/// `{name}` is replaced with the value of `name` from `vars`.
/// Unknown variables are left as-is.
///
/// Also supports ICU-style inline plural:
///   `{count, plural, zero{...} one{...} two{...} few{...} many{...} other{...}}`
/// where `#` inside the braces is replaced with the actual count value.
pub fn interpolate(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(template.len());
    let chars: Vec<char> = template.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '{' {
            if let Some(end) = find_matching_brace(&chars, i) {
                let inner: String = chars[i + 1..end].iter().collect();

                if let Some(replacement) = try_plural_expr(&inner, vars) {
                    result.push_str(&replacement);
                } else if let Some(val) = vars.get(inner.trim()) {
                    result.push_str(val);
                } else {
                    // Unknown variable: keep original
                    result.push('{');
                    result.push_str(&inner);
                    result.push('}');
                }
                i = end + 1;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Find the matching closing brace, accounting for nested braces.
fn find_matching_brace(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 0;
    for (j, &ch) in chars.iter().enumerate().skip(start) {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
    }
    None
}

/// Try to parse an ICU-style plural expression.
///
/// Format: `count, plural, one{# item} other{# items}`
fn try_plural_expr(inner: &str, vars: &HashMap<String, String>) -> Option<String> {
    // Split on first two commas: variable, "plural", forms...
    let mut parts = inner.splitn(3, ',');
    let var_name = parts.next()?.trim();
    let kind = parts.next()?.trim();
    let forms_str = parts.next()?.trim();

    if kind != "plural" {
        return None;
    }

    let count_str = vars.get(var_name)?;
    let count: f64 = count_str.parse().ok()?;
    let n = count.abs().floor() as u64;

    // Parse forms: `one{...} other{...}`
    let forms = parse_plural_forms(forms_str);

    // Determine category (simple English-style for inline)
    let category = if n == 0 {
        "zero"
    } else if n == 1 {
        "one"
    } else if n == 2 {
        "two"
    } else {
        "other"
    };

    let template = forms
        .get(category)
        .or_else(|| forms.get("other"))
        .cloned()
        .unwrap_or_default();

    // Replace `#` with the count
    let display = if count.fract() == 0.0 {
        format!("{}", count as i64)
    } else {
        format!("{}", count)
    };

    Some(template.replace('#', &display))
}

/// Parse `one{# item} other{# items}` into a map.
fn parse_plural_forms(s: &str) -> HashMap<String, String> {
    let mut forms = HashMap::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Skip whitespace
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= len {
            break;
        }

        // Read category name (until '{')
        let cat_start = i;
        while i < len && chars[i] != '{' {
            i += 1;
        }
        if i >= len {
            break;
        }
        let category: String = chars[cat_start..i].iter().collect();
        let category = category.trim().to_string();

        // Read the brace-delimited value
        if let Some(end) = find_matching_brace(&chars, i) {
            let value: String = chars[i + 1..end].iter().collect();
            forms.insert(category, value);
            i = end + 1;
        } else {
            break;
        }
    }

    forms
}
