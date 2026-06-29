//! Shared Regex builtin — used by both VM and interpreter.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}
use std::collections::HashMap;
use std::sync::Mutex;

const REGEX_CACHE_CAPACITY: usize = 128;

static REGEX_CACHE: Mutex<Option<RegexCache>> = Mutex::new(None);

struct RegexCache {
    entries: Vec<(String, regex::Regex)>,
}

impl RegexCache {
    fn new() -> Self {
        Self {
            entries: Vec::with_capacity(REGEX_CACHE_CAPACITY),
        }
    }

    fn get(&mut self, key: &str) -> Option<regex::Regex> {
        if let Some(pos) = self.entries.iter().position(|(k, _)| k == key) {
            let entry = self.entries.remove(pos);
            let re = entry.1.clone();
            self.entries.push(entry);
            Some(re)
        } else {
            None
        }
    }

    fn insert(&mut self, key: String, re: regex::Regex) {
        if self.entries.len() >= REGEX_CACHE_CAPACITY {
            self.entries.remove(0);
        }
        self.entries.push((key, re));
    }
}

/// Build a `regex::Regex` from a pattern plus a flag string.
///
/// Thin public wrapper over the cached internal [`build_re`] helper so
/// external callers (the retiring interpreter-era `builtins::regex` shim,
/// string method dispatchers, direct tests) can reuse the exact same
/// flag-prefix handling and regex cache as `call_regex_method`. Recognised
/// flags: `i`, `m`, `s`, `x` (other characters are ignored, matching the
/// interpreter-era `build_regex`).
///
/// The generic `V: SharedValue` parameter is unused at runtime; it keeps
/// call-sites consistent with the rest of the shared builtin API
/// (`build_regex::<V>(pat, flags)`).
pub fn build_regex(pattern: &str, flags: &str) -> HudHudResult<regex::Regex> {
    let flags_opt = if flags.is_empty() { None } else { Some(flags) };
    build_re(pattern, flags_opt)
}

fn build_re(pattern: &str, flags: Option<&str>) -> HudHudResult<regex::Regex> {
    let mut pat = pattern.to_string();
    if let Some(flags) = flags {
        let mut prefix = String::from("(?");
        for ch in flags.chars() {
            match ch {
                'i' | 'm' | 's' | 'x' => prefix.push(ch),
                _ => {}
            }
        }
        prefix.push(')');
        pat = format!("{}{}", prefix, pat);
    }

    let mut cache = REGEX_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let cache = cache.get_or_insert_with(RegexCache::new);
    if let Some(re) = cache.get(&pat) {
        return Ok(re);
    }

    let re = regex::Regex::new(&pat)
        .map_err(|e| runtime_error(format!("regex: invalid pattern: {}", e)))?;
    cache.insert(pat, re.clone());
    Ok(re)
}

/// Execute a regex method on the given arguments.
///
/// Handles: test, match, find_all, replace, replace_all, split, escape.
/// Enum identifying each regex operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegexMethodId {
    Test,
    Match,
    FindAll,
    Replace,
    ReplaceAll,
    Split,
    Escape,
}

impl std::str::FromStr for RegexMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "test" => Ok(Self::Test),
            "match" => Ok(Self::Match),
            "find_all" => Ok(Self::FindAll),
            "replace" => Ok(Self::Replace),
            "replace_all" => Ok(Self::ReplaceAll),
            "split" => Ok(Self::Split),
            "escape" => Ok(Self::Escape),
            _ => Err(runtime_error(format!("Unknown regex method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch for regex operations.
pub fn dispatch(method: RegexMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    let pattern = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            runtime_error(format!("regex.{:?}: expected pattern string", method).to_lowercase())
        })?
        .to_string();

    let input = match args.get(1).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None if method == RegexMethodId::Escape => {
            return Ok(Value16::string(regex::escape(&pattern)));
        }
        None => {
            return Err(runtime_error(
                format!("regex.{:?}: expected input string", method).to_lowercase(),
            ))
        }
    };

    let flags = args.get(2).and_then(|v| v.as_str());

    match method {
        RegexMethodId::Test => {
            let re = build_re(&pattern, flags)?;
            Ok(Value16::bool_(re.is_match(&input)))
        }
        RegexMethodId::Match => {
            let re = build_re(&pattern, flags)?;
            match re.captures(&input) {
                Some(caps) => {
                    let m = caps.get(0).unwrap();
                    let mut result = hudhudscript_bytecode::ObjMap::default();
                    result.insert("matched".to_string(), Value16::bool_(true));
                    result.insert("index".to_string(), Value16::number(m.start() as f64));
                    result.insert("value".to_string(), Value16::string(m.as_str().to_string()));
                    let groups: Vec<Value16> = caps
                        .iter()
                        .skip(1)
                        .map(|m| match m {
                            Some(m) => Value16::string(m.as_str().to_string()),
                            None => Value16::null(),
                        })
                        .collect();
                    result.insert("groups".to_string(), Value16::array(groups));
                    Ok(Value16::object(result))
                }
                None => Ok(Value16::null()),
            }
        }
        RegexMethodId::FindAll => {
            let re = build_re(&pattern, flags)?;
            let matches: Vec<Value16> = re
                .find_iter(&input)
                .map(|m| {
                    let mut obj = hudhudscript_bytecode::ObjMap::default();
                    obj.insert("index".to_string(), Value16::number(m.start() as f64));
                    obj.insert("value".to_string(), Value16::string(m.as_str().to_string()));
                    Value16::object(obj)
                })
                .collect();
            Ok(Value16::array(matches))
        }
        RegexMethodId::Replace => {
            let replacement = args
                .get(2)
                .and_then(|v| v.as_str())
                .ok_or_else(|| runtime_error("regex.replace: expected replacement"))?
                .to_string();
            let re_flags = args.get(3).and_then(|v| v.as_str());
            let re = build_re(&pattern, re_flags)?;
            Ok(Value16::string(
                re.replace(&input, replacement.as_str()).to_string(),
            ))
        }
        RegexMethodId::ReplaceAll => {
            let replacement = args
                .get(2)
                .and_then(|v| v.as_str())
                .ok_or_else(|| runtime_error("regex.replace_all: expected replacement"))?
                .to_string();
            let re_flags = args.get(3).and_then(|v| v.as_str());
            let re = build_re(&pattern, re_flags)?;
            Ok(Value16::string(
                re.replace_all(&input, replacement.as_str()).to_string(),
            ))
        }
        RegexMethodId::Split => {
            let re = build_re(&pattern, flags)?;
            let parts: Vec<Value16> = re
                .split(&input)
                .map(|s| Value16::string(s.to_string()))
                .collect();
            Ok(Value16::array(parts))
        }
        RegexMethodId::Escape => Ok(Value16::string(regex::escape(&pattern))),
    }
}
