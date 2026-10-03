//! Tests for hudhudscript-hudunit project scaffolding — `init` creates a
//! runnable scaffold and never touches existing files.

use hudhudscript_hudunit::config::HudunitConfig;
use hudhudscript_hudunit::discover::{collect_files, discover_file};
use hudhudscript_hudunit::init::init;
use hudhudscript_hudunit::runner::{run, RunOptions};

#[test]
fn init_creates_scaffold_and_respects_existing_files() {
    let dir = std::env::temp_dir().join(format!(
        "hudunit_init_{}_{}",
        std::process::id(),
        line!()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let first = init(&dir).unwrap();
    assert!(first.created.contains(&"src/math.hud".to_string()));
    assert!(first.created.contains(&"tests/math/test_math.hud".to_string()));
    assert!(first.created.contains(&"hudunit.toml".to_string()));
    assert!(first.skipped.is_empty());

    // The scaffold must be immediately runnable through the framework.
    let cfg = HudunitConfig::default();
    let files = collect_files(&[dir.join("tests")], &cfg);
    assert_eq!(files.len(), 1);
    let discovered: Vec<_> = files
        .iter()
        .map(|p| discover_file(p, None).unwrap())
        .collect();
    let opts = RunOptions::default();
    let outcome = run(&discovered, &cfg, &opts);
    assert_eq!(outcome.passed(), 2, "scaffold tests must pass as-is");

    // Second init must not touch anything.
    std::fs::write(dir.join("src/math.hud"), "// changed").unwrap();
    let second = init(&dir).unwrap();
    assert!(second.created.is_empty());
    assert_eq!(second.skipped.len(), 3);
    assert_eq!(
        std::fs::read_to_string(dir.join("src/math.hud")).unwrap(),
        "// changed"
    );

    std::fs::remove_dir_all(&dir).ok();
}
