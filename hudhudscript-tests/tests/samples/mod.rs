//! CI validation for sample scripts.
//!
//! Walks `samples/` and ensures every `.hudhud` file parses without errors.
//! This catches broken syntax in sample scripts before they reach users.

use std::path::{Path, PathBuf};

use hudhudscript_parser::parse;

/// Recursively collect all `.hudhud` files under `dir`.
fn collect_hudhud_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.is_dir() {
        return files;
    }
    for entry in std::fs::read_dir(dir).expect("failed to read samples dir") {
        let entry = entry.expect("failed to read dir entry");
        let path = entry.path();
        if path.is_dir() {
            // Skip _wip directories — these contain work-in-progress examples
            // that may have intentionally broken syntax
            if matches!(
                path.file_name().and_then(|n| n.to_str()),
                Some("_wip") | Some("_archive")
            ) {
                continue;
            }
            files.extend(collect_hudhud_files(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("hudhud") {
            files.push(path);
        }
    }
    files.sort();
    files
}

/// Resolve the samples/ directory relative to the workspace root.
fn samples_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("manifest dir has no parent");
    let samples = workspace_root.join("samples");
    if samples.is_dir() {
        return samples;
    }
    panic!("Cannot find samples/ directory at {}", samples.display());
}

#[test]
fn all_samples_parse_successfully() {
    let dir = samples_dir();
    let files = collect_hudhud_files(&dir);

    assert!(
        !files.is_empty(),
        "No .hudhud files found under {}",
        dir.display()
    );

    let mut failures: Vec<(PathBuf, String)> = Vec::new();

    for file in &files {
        let source = std::fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", file.display(), e));

        if let Err(err) = parse(&source) {
            failures.push((file.clone(), format!("{err}")));
        }
    }

    if !failures.is_empty() {
        let mut msg = format!(
            "\n{} of {} sample files failed to parse:\n\n",
            failures.len(),
            files.len()
        );
        for (path, err) in &failures {
            let relative = path.strip_prefix(&dir).unwrap_or(path);
            msg.push_str(&format!(
                "  FAIL  {}\n        {}\n\n",
                relative.display(),
                err
            ));
        }
        panic!("{msg}");
    }

    eprintln!(
        "OK: all {} sample files in {} parsed successfully",
        files.len(),
        dir.display()
    );
}
