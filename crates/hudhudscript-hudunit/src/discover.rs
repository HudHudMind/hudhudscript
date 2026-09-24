//! Test file discovery: recursive `*.hhs` / `*.hud` / `*.hudhud` collection,
//! `test_` / `setup` / `teardown` function discovery from the AST, and
//! `@group` annotation extraction from preceding comment lines.

use std::path::{Path, PathBuf};

use hudhudscript_ast::Stmt;
use hudhudscript_parser::parse;

use crate::config::HudunitConfig;

/// A single discovered test function.
#[derive(Debug, Clone)]
pub struct DiscoveredTest {
    pub name: String,
    /// `true` for `ignore_test_*` / `_test_*` — reported as skipped.
    pub skip: bool,
    /// `@group` annotations attached via preceding comments.
    pub annotations: Vec<String>,
    /// 1-based declaration line in the source file.
    pub line: usize,
}

/// A discovered test file with its tests and lifecycle helpers.
#[derive(Debug, Clone)]
pub struct TestFile {
    pub path: PathBuf,
    /// Path-derived group chain relative to the discovery root
    /// (`tests/math/test_x.hud` → `["math"]`).
    pub group_path: Vec<String>,
    pub tests: Vec<DiscoveredTest>,
    pub has_setup: bool,
    pub has_teardown: bool,
}

/// Any error encountered while collecting/discovering tests.
#[derive(Debug, thiserror::Error)]
pub enum DiscoverError {
    #[error("IO error reading {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("parse error in {path}: {message}")]
    Parse { path: PathBuf, message: String },
}

/// Recursively collect test files under the given paths (files pass through
/// if they carry a configured extension, directories are walked). Results are
/// sorted for deterministic ordering.
pub fn collect_files(paths: &[PathBuf], cfg: &HudunitConfig) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for path in paths {
        if path.is_file() && cfg.is_test_file(path) {
            out.push(path.clone());
        } else if path.is_dir() {
            collect_dir(path, cfg, &mut out);
        }
    }
    out.sort();
    out.dedup();
    out
}

fn collect_dir(dir: &Path, cfg: &HudunitConfig, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_dir(&path, cfg, out);
        } else if cfg.is_test_file(&path) {
            out.push(path);
        }
    }
}

/// Discover the tests of a single file. `root` anchors the path-derived
/// group chain (typically the scanned directory or the file's parent).
pub fn discover_file(path: &Path, root: Option<&Path>) -> Result<TestFile, DiscoverError> {
    let source = std::fs::read_to_string(path).map_err(|source| DiscoverError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let ast = parse(&source).map_err(|e| DiscoverError::Parse {
        path: path.to_path_buf(),
        message: format!("{:?}", e),
    })?;

    let group_path = derive_group_path(path, root);
    let comment_groups = collect_group_annotations(&source);

    let mut tests = Vec::new();
    let mut has_setup = false;
    let mut has_teardown = false;

    for stmt in &ast {
        let Stmt::Function { name, span, .. } = stmt else {
            continue;
        };
        match name.as_str() {
            "setup" | "before" | "beforeEach" => {
                has_setup = true;
                continue;
            }
            "teardown" | "after" | "afterEach" => {
                has_teardown = true;
                continue;
            }
            _ => {}
        }
        let (skip, base) = if let Some(stripped) = name.strip_prefix("ignore_test_") {
            (true, stripped)
        } else if let Some(stripped) = name.strip_prefix("_test_") {
            (true, stripped)
        } else if let Some(stripped) = name.strip_prefix("test_") {
            (false, stripped)
        } else {
            continue;
        };
        let _ = base;
        let line = span.start.line;
        let annotations = comment_groups
            .iter()
            .filter(|(at_line, _)| *at_line == line)
            .map(|(_, groups)| groups.clone())
            .next()
            .unwrap_or_default();
        tests.push(DiscoveredTest {
            name: name.clone(),
            skip,
            annotations,
            line,
        });
    }

    Ok(TestFile {
        path: path.to_path_buf(),
        group_path,
        tests,
        has_setup,
        has_teardown,
    })
}

/// `tests/math/test_x.hud` with root `tests` → `["math"]`; a file directly
/// in the root keeps an empty chain (its fallback group is the file stem).
fn derive_group_path(path: &Path, root: Option<&Path>) -> Vec<String> {
    let rel = match root {
        Some(root) => path.strip_prefix(root).unwrap_or(path),
        None => path,
    };
    let mut chain: Vec<String> = rel
        .parent()
        .map(|p| {
            p.components()
                .filter_map(|c| c.as_os_str().to_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    chain.retain(|part| !part.is_empty());
    chain
}

/// Extract `// @group foo` annotations keyed by the 1-based line of the
/// *next* non-comment line. Comment lines directly above a function attach
/// their groups to it; blank lines reset the pending set. (`//` is the
/// language's comment syntax; `#` is not.)
fn collect_group_annotations(source: &str) -> Vec<(usize, Vec<String>)> {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = Vec::new();
    let mut pending: Vec<String> = Vec::new();

    for (idx, raw) in lines.iter().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            pending.clear();
            continue;
        }
        if line.starts_with("//") {
            if let Some(group) = line.trim_start_matches('/').split_whitespace().nth(1) {
                if line.contains("@group") {
                    let name = group.trim().to_string();
                    if !name.is_empty() && !pending.contains(&name) {
                        pending.push(name);
                    }
                }
            }
            continue;
        }
        // First non-comment line: attach pending groups (if any).
        if !pending.is_empty() {
            out.push((idx + 1, std::mem::take(&mut pending)));
        }
        pending.clear();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_annotations_attach_to_following_line() {
        let src = "// @group hizli\n// @group math\nfn test_a() {}\n\nfn test_b() {}\n";
        let groups = collect_group_annotations(src);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, 3);
        assert_eq!(groups[0].1, vec!["hizli".to_string(), "math".to_string()]);
    }

    #[test]
    fn blank_line_resets_pending_groups() {
        let src = "// @group x\n\nfn test_a() {}\n";
        assert!(collect_group_annotations(src).is_empty());
    }

    #[test]
    fn slash_comment_groups_supported() {
        let src = "// @group slow\nfn test_a() {}\n";
        let groups = collect_group_annotations(src);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].1, vec!["slow".to_string()]);
    }

    #[test]
    fn derive_group_path_nested() {
        let path = Path::new("/tmp/tests/math/test_a.hud");
        let chain = derive_group_path(path, Some(Path::new("/tmp/tests")));
        assert_eq!(chain, vec!["math".to_string()]);
    }

    #[test]
    fn derive_group_path_root_file() {
        let path = Path::new("/tmp/tests/test_a.hud");
        let chain = derive_group_path(path, Some(Path::new("/tmp/tests")));
        assert!(chain.is_empty());
    }

    #[test]
    fn extension_filter() {
        let cfg = HudunitConfig::default();
        assert!(cfg.is_test_file(Path::new("a.hhs")));
        assert!(cfg.is_test_file(Path::new("a.hud")));
        assert!(cfg.is_test_file(Path::new("a.hudhud")));
        assert!(!cfg.is_test_file(Path::new("a.rs")));
    }
}
