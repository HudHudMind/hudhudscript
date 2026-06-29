//! patch.apply — SEARCH/REPLACE diff applier for AIDER loop engineering.
//!
//! Applies a SEARCH/REPLACE diff to a file. The SEARCH block must match
//! exactly once. Multi-match and no-match return errors (fail-closed,
//! Kural 7c: no silent wrong patch).

use std::fs;
use std::path::Path;

/// Result of a patch.apply operation.
#[derive(Debug)]
pub struct PatchResult {
    /// Number of replacements made (always 1 on success).
    pub replacements: usize,
    /// Whether the file was modified.
    pub modified: bool,
}

/// Apply a SEARCH/REPLACE diff to a file.
///
/// # Algorithm
/// 1. Read file content.
/// 2. Find SEARCH block — must match exactly once.
/// 3. If no match, return error.
/// 4. If multiple matches, return error (ambiguous).
/// 5. Replace matched block with REPLACE block.
/// 6. Atomic write: temp file → rename.
pub fn patch_apply(
    file_path: &str,
    search: &str,
    replace: &str,
) -> Result<PatchResult, String> {
    let path = Path::new(file_path);
    let content = fs::read_to_string(path)
        .map_err(|e| format!("patch.apply: cannot read '{}': {}", file_path, e))?;

    // Count exact matches of the search block.
    let matches: Vec<usize> = content.match_indices(search).map(|(i, _)| i).collect();

    if matches.is_empty() {
        return Err(format!(
            "patch.apply: SEARCH block not found in '{}'. \
             The file may have changed since the diff was generated.",
            file_path
        ));
    }

    if matches.len() > 1 {
        return Err(format!(
            "patch.apply: SEARCH block matches {} times in '{}'. \
             Add more context lines to make the match unique.",
            matches.len(),
            file_path
        ));
    }

    // Apply the replacement.
    let new_content = content.replacen(search, replace, 1);

    // Atomic write via temp file.
    let tmp_path = format!("{}.hudhud_patch_tmp", file_path);
    fs::write(&tmp_path, &new_content)
        .map_err(|e| format!("patch.apply: cannot write temp file: {}", e))?;
    fs::rename(&tmp_path, path)
        .map_err(|e| format!("patch.apply: cannot commit patch: {}", e))?;

    Ok(PatchResult {
        replacements: 1,
        modified: search != replace,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup(path: &str, content: &str) {
        fs::write(path, content).unwrap();
    }

    fn cleanup(path: &str) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(format!("{}.hudhud_patch_tmp", path));
    }

    #[test]
    fn exact_match_single_replacement() {
        let p = "/tmp/test_patch_exact.txt";
        setup(p, "hello world\nfoo bar\n");
        let r = patch_apply(p, "hello world", "HELLO WORLD").unwrap();
        assert!(r.modified);
        assert_eq!(r.replacements, 1);
        assert_eq!(fs::read_to_string(p).unwrap(), "HELLO WORLD\nfoo bar\n");
        cleanup(p);
    }

    #[test]
    fn no_match_returns_error() {
        let p = "/tmp/test_patch_nomatch.txt";
        setup(p, "hello world\n");
        let r = patch_apply(p, "nonexistent", "x");
        assert!(r.is_err());
        cleanup(p);
    }

    #[test]
    fn multi_match_returns_error() {
        let p = "/tmp/test_patch_multi.txt";
        setup(p, "foo\nfoo\n");
        let r = patch_apply(p, "foo", "bar");
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("matches 2 times"));
        cleanup(p);
    }

    #[test]
    fn no_change_when_search_equals_replace() {
        let p = "/tmp/test_patch_nochange.txt";
        setup(p, "hello world\n");
        let r = patch_apply(p, "hello world", "hello world").unwrap();
        assert!(!r.modified);
        assert_eq!(r.replacements, 1);
        cleanup(p);
    }

    #[test]
    fn file_not_found() {
        let r = patch_apply("/tmp/no_such_file_xyz.txt", "a", "b");
        assert!(r.is_err());
    }
}
