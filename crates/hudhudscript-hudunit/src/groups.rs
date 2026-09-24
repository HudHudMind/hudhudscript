//! Group model: every test belongs to a path-derived group (directory chain,
//! falling back to the file stem) plus any `@group` annotations. Selection
//! logic honors `--group` (any-of) and `--exclude-group`.

use crate::discover::{DiscoveredTest, TestFile};

/// Effective groups of a single test inside a file.
pub fn groups_of(file: &TestFile, test: &DiscoveredTest) -> Vec<String> {
    let mut groups = file.group_path.clone();
    if groups.is_empty() {
        // Root-level file: the file stem is the group.
        let stem = file
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("default")
            .to_string();
        groups.push(stem);
    }
    for annotation in &test.annotations {
        if !groups.contains(annotation) {
            groups.push(annotation.clone());
        }
    }
    groups
}

/// The display group used for console/JSON/HTML grouping headers: the deepest
/// path-derived group (or file stem), so annotations don't fragment output.
pub fn display_group(file: &TestFile) -> String {
    if let Some(last) = file.group_path.last() {
        return last.clone();
    }
    file.path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("default")
        .to_string()
}

/// `--group math --group hizli` semantics: a test is selected when at least
/// one of its groups matches a requested group (prefix match on path-derived
/// chains, exact match on annotations), `exclude` wins over `include`.
/// Empty `include` selects everything not excluded.
pub fn selected(
    file: &TestFile,
    test: &DiscoveredTest,
    include: &[String],
    exclude: &[String],
) -> bool {
    let groups = groups_of(file, test);
    let excluded = exclude
        .iter()
        .any(|pattern| groups.iter().any(|g| group_matches(g, pattern)));
    if excluded {
        return false;
    }
    if include.is_empty() {
        return true;
    }
    groups.iter().any(|g| include.iter().any(|p| group_matches(g, p)))
}

/// `math` matches `math` and nested chains like `math/arithmetic`.
fn group_matches(group: &str, pattern: &str) -> bool {
    group == pattern || group.starts_with(&format!("{}/", pattern))
}

/// Aggregate of all groups across discovered files: group name → test count.
pub fn group_summary(files: &[TestFile]) -> Vec<(String, usize)> {
    let mut counts = std::collections::BTreeMap::new();
    for file in files {
        for test in &file.tests {
            for group in groups_of(file, test) {
                *counts.entry(group).or_insert(0usize) += 1;
            }
        }
    }
    counts.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn file(chain: &[&str], stem: &str) -> TestFile {
        TestFile {
            path: PathBuf::from(format!("/t/{}/{}.hud", chain.join("/"), stem)),
            group_path: chain.iter().map(|s| s.to_string()).collect(),
            tests: vec![],
            has_setup: false,
            has_teardown: false,
        }
    }

    fn test(annotations: &[&str]) -> DiscoveredTest {
        DiscoveredTest {
            name: "test_x".into(),
            skip: false,
            annotations: annotations.iter().map(|s| s.to_string()).collect(),
            line: 1,
        }
    }

    #[test]
    fn root_file_groups_fall_back_to_stem() {
        let f = file(&[], "test_math");
        let g = groups_of(&f, &test(&[]));
        assert_eq!(g, vec!["test_math".to_string()]);
    }

    #[test]
    fn annotations_merge_with_path_groups() {
        let f = file(&["math"], "test_arithmetic");
        let g = groups_of(&f, &test(&["hizli"]));
        assert_eq!(g, vec!["math".to_string(), "hizli".to_string()]);
    }

    #[test]
    fn include_selects_matching_group() {
        let f = file(&["math"], "t");
        assert!(selected(&f, &test(&[]), &["math".into()], &[]));
        assert!(!selected(&f, &test(&[]), &["string".into()], &[]));
    }

    #[test]
    fn nested_chain_prefix_matches() {
        let f = file(&["math", "arithmetic"], "t");
        assert!(selected(&f, &test(&[]), &["math".into()], &[]));
    }

    #[test]
    fn exclude_overrides_include() {
        let f = file(&["math"], "t");
        assert!(!selected(
            &f,
            &test(&["yavas"]),
            &["math".into()],
            &["yavas".into()]
        ));
    }

    #[test]
    fn empty_include_selects_all() {
        let f = file(&["string"], "t");
        assert!(selected(&f, &test(&[]), &[], &[]));
    }
}
