/// Test that ALL example files in examples/ directory parse successfully
use hudhudscript_parser::parse;
use std::fs;
use std::path::Path;

fn collect_example_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip _wip directory — contains aspirational examples with
                // syntax not yet supported by the parser
                if path.file_name().map(|n| n == "_wip").unwrap_or(false) {
                    continue;
                }
                files.extend(collect_example_files(&path));
            } else if let Some(ext) = path.extension() {
                if ext == "hud" || ext == "hudhud" {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    files
}

#[test]
fn all_examples_parse_successfully() {
    // Find examples dir — could be at workspace root or relative
    // CWD is hudhud-script-tests/ (separate repo, workspace member)
    // examples/ is in the parent hudhud-script repo
    let possible_paths = [
        Path::new("../hudhud-script/examples"),
        Path::new("../examples"),
        Path::new("examples"),
    ];

    let examples_dir = possible_paths
        .iter()
        .find(|p| p.exists())
        .expect("examples/ directory not found");

    let files = collect_example_files(examples_dir);
    assert!(
        !files.is_empty(),
        "No .hud/.hudhud files found in examples/"
    );

    let mut pass = 0;
    let mut fail = 0;
    let mut failures = Vec::new();

    for file in &files {
        let content = fs::read_to_string(file).unwrap_or_default();
        if content.trim().is_empty() {
            continue;
        }
        match parse(&content) {
            Ok(stmts) => {
                assert!(
                    !stmts.is_empty(),
                    "Parsed 0 statements from {}",
                    file.display()
                );
                pass += 1;
            }
            Err(e) => {
                fail += 1;
                failures.push(format!(
                    "{}: {}",
                    file.display(),
                    format!("{}", e).chars().take(100).collect::<String>()
                ));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "Parse failures ({}/{}):\n{}",
            fail,
            pass + fail,
            failures.join("\n")
        );
    }

    eprintln!("All {} examples parsed successfully", pass);
}
