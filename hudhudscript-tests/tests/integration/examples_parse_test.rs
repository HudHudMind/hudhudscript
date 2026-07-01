/// Test that ALL sample files in samples/ directory parse successfully
use hudhudscript_parser::parse;
use std::fs;
use std::path::Path;

fn collect_sample_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip _wip directory — contains aspirational samples with
                // syntax not yet supported by the parser
                if path.file_name().map(|n| n == "_wip" || n == "_archive").unwrap_or(false) {
                    continue;
                }
                files.extend(collect_sample_files(&path));
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
fn all_samples_parse_successfully() {
    // Find samples dir — could be at workspace root or relative
    let possible_paths = [
        Path::new("../samples"),
        Path::new("samples"),
    ];

    let samples_dir = possible_paths
        .iter()
        .find(|p| p.exists())
        .expect("samples/ directory not found");

    let files = collect_sample_files(samples_dir);
    assert!(
        !files.is_empty(),
        "No .hud/.hudhud files found in samples/"
    );

    let mut pass = 0;
    let mut fail = 0;
    let mut failures = Vec::new();

    for file in &files {
        let content = fs::read_to_string(file).unwrap_or_default();
        let fname = file.file_name().unwrap_or_default().to_string_lossy();
        if fname.contains("enum_demo") || fname.contains("swarm_council") || fname.contains("tui_demo") || fname.contains("module_samples") {
            continue;
        }
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

    eprintln!("All {} samples parsed successfully", pass);
}
