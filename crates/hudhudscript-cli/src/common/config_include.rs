//! Recursive TOML include and deep-merge engine (JIT_AOT_ARCHITECTURE §20.2).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_INCLUDE_DEPTH: usize = 8;

/// Loads a TOML file with recursive `include = [...]` resolution and deep-merge.
pub fn load_toml_with_includes(path: &Path) -> Result<toml::Value, String> {
    let mut visited = HashSet::new();
    load_recursive(path, &mut visited, 0)
}

fn load_recursive(
    path: &Path,
    visited: &mut HashSet<PathBuf>,
    depth: usize,
) -> Result<toml::Value, String> {
    if depth > MAX_INCLUDE_DEPTH {
        return Err(format!(
            "maximum include depth of {} exceeded while including {}",
            MAX_INCLUDE_DEPTH,
            path.display()
        ));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read config file {}: {}", path.display(), e))?;

    if !content.contains("include") {
        return toml::from_str(&content)
            .map_err(|e| format!("parse error in {}: {}", path.display(), e));
    }

    let norm_path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(norm_path.clone()) {
        return Err(format!(
            "cyclical include detected for {}",
            path.display()
        ));
    }

    let parsed: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("parse error in {}: {}", path.display(), e))?;

    let has_includes = if let toml::Value::Table(ref table) = parsed {
        table.contains_key("include")
    } else {
        false
    };

    if !has_includes {
        visited.remove(&norm_path);
        return Ok(parsed);
    }

    let mut merged = toml::Value::Table(toml::map::Map::new());

    // Resolve `include` array if present
    if let toml::Value::Table(ref table) = parsed {
        if let Some(inc_val) = table.get("include") {
            let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
            match inc_val {
                toml::Value::Array(items) => {
                    for item in items {
                        if let toml::Value::String(rel_str) = item {
                            let inc_path = base_dir.join(rel_str);
                            let included_val = load_recursive(&inc_path, visited, depth + 1)?;
                            deep_merge_toml(&mut merged, included_val);
                        } else {
                            visited.remove(&norm_path);
                            return Err(format!(
                                "invalid non-string item in `include` array of {}",
                                path.display()
                            ));
                        }
                    }
                }
                toml::Value::String(rel_str) => {
                    let inc_path = base_dir.join(rel_str);
                    let included_val = load_recursive(&inc_path, visited, depth + 1)?;
                    deep_merge_toml(&mut merged, included_val);
                }
                _ => {
                    visited.remove(&norm_path);
                    return Err(format!(
                        "`include` key must be an array of strings or string in {}",
                        path.display()
                    ));
                }
            }
        }
    }

    // Main file overrides includes (JIT_AOT_ARCHITECTURE §20.2)
    deep_merge_toml(&mut merged, parsed);

    visited.remove(&norm_path);
    Ok(merged)
}

/// Recursively merges `overlay` into `base` (deep-merge).
/// Overlay table keys overwrite base table keys. Non-table values are replaced.
pub fn deep_merge_toml(base: &mut toml::Value, overlay: toml::Value) {
    match (base, overlay) {
        (toml::Value::Table(base_map), toml::Value::Table(overlay_map)) => {
            for (k, v) in overlay_map {
                match base_map.get_mut(&k) {
                    Some(base_v) => deep_merge_toml(base_v, v),
                    None => {
                        base_map.insert(k, v);
                    }
                }
            }
        }
        (base_slot, overlay_val) => {
            *base_slot = overlay_val;
        }
    }
}
