//! Shared string method implementations — used by both VM and interpreter.

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

/// Execute a string method on `s` with the given arguments.
///
/// Handles 25+ string methods: length, split, trim, toUpperCase, toLowerCase,
/// indexOf, contains, replace, substring, slice, concat, startsWith, endsWith,
/// trimStart, trimEnd, repeat, padStart, padEnd, match, matchAll, replaceRegex.
pub fn call_string_method(
    s: &str,
    method: &str,
    args: &[Value16],
    is_ascii: bool,
) -> SharedResult<Value16> {
    match method {
        "length" => {
            let len = if is_ascii { s.len() } else { s.chars().count() };
            Ok(Value16::int(len as i64))
        }

        "split" | "böl" | "ayır" => {
            let delimiter = args.first().and_then(|v| v.as_str()).unwrap_or(" ");
            let parts: Vec<Value16> = s
                .split(delimiter)
                .map(|p| Value16::string_from_str(p))
                .collect();
            Ok(Value16::array(parts))
        }

        "trim" | "kırp" => Ok(Value16::string_from_str(s.trim())),
        "toUpperCase" | "toUpper" | "to_upper" | "büyükHarfeÇevir" | "büyült" => {
            Ok(Value16::string(s.to_uppercase()))
        }
        "toLowerCase" | "toLower" | "to_lower" | "küçükHarfeÇevir" | "küçült" => {
            Ok(Value16::string(s.to_lowercase()))
        }

        "indexOf" => {
            let needle = args.first().and_then(|v| v.as_str()).unwrap_or("");
            let idx = s.find(needle).map(|i| i as i64).unwrap_or(-1);
            Ok(Value16::int(idx))
        }

        "contains" => {
            let needle = args.first().and_then(|v| v.as_str()).unwrap_or("");
            Ok(Value16::boolean(s.contains(needle)))
        }

        "replace" | "değiştir" => {
            let pattern = args.first().and_then(|v| v.as_str()).unwrap_or("");
            let replacement = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
            Ok(Value16::string(s.replace(pattern, replacement)))
        }

        "substring" | "slice" => {
            let start = args.first().and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
            let end = args
                .get(1)
                .and_then(|v| v.as_number())
                .map(|n| n as usize)
                .unwrap_or_else(|| if is_ascii { s.len() } else { s.chars().count() });
            // P4: O(1) byte slice for ASCII receivers (no per-call scan).
            if is_ascii {
                let start = start.min(s.len());
                let end = end.min(s.len()).max(start);
                return Ok(Value16::string_from_str(&s[start..end]));
            }
            let result: String = s
                .chars()
                .skip(start)
                .take(end.saturating_sub(start))
                .collect();
            Ok(Value16::string_from_str(&result))
        }

        "concat" => {
            let mut result = s.to_string();
            for arg in args {
                result.push_str(&arg.display_string());
            }
            Ok(Value16::string_from_str(&result))
        }

        "startsWith" | "starts_with" => {
            let prefix = args.first().and_then(|v| v.as_str()).unwrap_or("");
            Ok(Value16::boolean(s.starts_with(prefix)))
        }

        "endsWith" | "ends_with" => {
            let suffix = args.first().and_then(|v| v.as_str()).unwrap_or("");
            Ok(Value16::boolean(s.ends_with(suffix)))
        }

        "trimStart" | "trim_start" => Ok(Value16::string_from_str(s.trim_start())),
        "trimEnd" | "trim_end" => Ok(Value16::string_from_str(s.trim_end())),

        // Bottleneck #2 fix: O(n) string reverse without per-char allocation.
        "reverse" => {
            let rev: String = s.chars().rev().collect();
            Ok(Value16::string_from_str(&rev))
        }

        // ISSUE-5: two-pointer palindrome check — zero allocation.
        "is_palindrome" => {
            let mut iter = s.chars();
            while let (Some(l), Some(r)) = (iter.next(), iter.next_back()) {
                if l != r {
                    return Ok(Value16::bool_(false));
                }
            }
            Ok(Value16::bool_(true))
        }

        "repeat" => {
            let n = args
                .first()
                .and_then(|v| v.as_number())
                .ok_or_else(|| runtime_error("repeat() requires a number argument"))?;
            let count = n as usize;
            if count > 10_000_000 {
                return Err(runtime_error(
                    "string.repeat: count exceeds maximum (10000000)".to_string(),
                ));
            }
            Ok(Value16::string(s.repeat(count)))
        }

        "padStart" | "pad_start" => {
            let target_len = args
                .first()
                .and_then(|v| v.as_number())
                .ok_or_else(|| runtime_error("padStart() requires a number argument"))?
                as usize;
            if target_len > 10_000_000 {
                return Err(runtime_error(
                    "string.padStart: result length exceeds maximum (10000000)".to_string(),
                ));
            }
            let pad_char = args
                .get(1)
                .and_then(|v| v.as_str())
                .and_then(|s| s.chars().next())
                .unwrap_or(' ');
            let char_count = if is_ascii { s.len() } else { s.chars().count() };
            if char_count >= target_len {
                Ok(Value16::string(s.to_string()))
            } else {
                let padding: String =
                    std::iter::repeat_n(pad_char, target_len - char_count).collect();
                Ok(Value16::string(format!("{}{}", padding, s)))
            }
        }

        "padEnd" | "pad_end" => {
            let target_len = args
                .first()
                .and_then(|v| v.as_number())
                .ok_or_else(|| runtime_error("padEnd() requires a number argument"))?
                as usize;
            if target_len > 10_000_000 {
                return Err(runtime_error(
                    "string.padEnd: result length exceeds maximum (10000000)".to_string(),
                ));
            }
            let pad_char = args
                .get(1)
                .and_then(|v| v.as_str())
                .and_then(|s| s.chars().next())
                .unwrap_or(' ');
            let char_count = if is_ascii { s.len() } else { s.chars().count() };
            if char_count >= target_len {
                Ok(Value16::string(s.to_string()))
            } else {
                let padding: String =
                    std::iter::repeat_n(pad_char, target_len - char_count).collect();
                Ok(Value16::string(format!("{}{}", s, padding)))
            }
        }

        // Regex-powered string methods
        "match" | "match_all" | "matchAll" | "replace_regex" | "replaceRegex" => {
            let pattern = args
                .first()
                .and_then(|v| v.as_str())
                .ok_or_else(|| runtime_error(format!("{}: expected pattern string", method)))?;

            let flags_idx = if method.contains("replace") { 2 } else { 1 };
            let flags = args.get(flags_idx).and_then(|v| v.as_str());

            let pat = if let Some(flags) = flags {
                let mut prefix = String::from("(?");
                for ch in flags.chars() {
                    match ch {
                        'i' | 'm' | 's' | 'x' => prefix.push(ch),
                        _ => {}
                    }
                }
                prefix.push(')');
                format!("{}{}", prefix, pattern)
            } else {
                pattern.to_string()
            };

            let re = regex::Regex::new(&pat)
                .map_err(|e| runtime_error(format!("regex error: {}", e)))?;

            match method {
                "match" => match re.captures(s) {
                    Some(caps) => {
                        let m = caps.get(0).unwrap();
                        let mut result = hudhudscript_bytecode::ObjMap::default();
                        result.insert("matched".to_string(), Value16::boolean(true));
                        result.insert("index".to_string(), Value16::number(m.start() as f64));
                        result.insert("value".to_string(), Value16::string(m.as_str().to_string()));
                        let groups: Vec<Value16> = caps
                            .iter()
                            .skip(1)
                            .map(|m: Option<regex::Match<'_>>| match m {
                                Some(m) => Value16::string(m.as_str().to_string()),
                                None => Value16::null(),
                            })
                            .collect();
                        result.insert("groups".to_string(), Value16::array(groups));
                        Ok(Value16::object(result))
                    }
                    None => Ok(Value16::null()),
                },
                "match_all" | "matchAll" => {
                    let matches: Vec<Value16> = re
                        .find_iter(s)
                        .map(|m: regex::Match<'_>| Value16::string(m.as_str().to_string()))
                        .collect();
                    Ok(Value16::array(matches))
                }
                "replace_regex" | "replaceRegex" => {
                    let replacement = args
                        .get(1)
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| runtime_error("replace_regex: expected replacement"))?;
                    Ok(Value16::string(re.replace_all(s, replacement).to_string()))
                }
                _ => unreachable!(),
            }
        }

        _ => Err(runtime_error(format!("Unknown string method: {}", method))),
    }
}
