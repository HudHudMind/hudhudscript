//! `hudunit init` — scaffold a testable HudHudScript project in the current
//! directory: `src/` with a starter module, `tests/` with a passing example
//! test (demonstrating imports, assertions and a group), and a commented
//! `hudunit.toml`. Existing files are never overwritten.

use std::path::Path;

/// What `init` did, for the CLI to report.
#[derive(Debug, Default, PartialEq)]
pub struct InitReport {
    pub created: Vec<String>,
    pub skipped: Vec<String>,
}

const SRC_MATH: &str = r#"// src/math.hud — gerçek kodunuz buraya.
// Test edilecek fonksiyonlar normal fonksiyonlardır.

fn topla(a, b) {
    return a + b;
}
"#;

const TEST_MATH: &str = r#"// tests/math/test_math.hud — örnek test dosyası.
// Çalıştırmak için: hudunit

import { topla } from "../../src/math.hud";

// @group hizli
fn test_topla_pozitif() {
    assert_eq(topla(2, 3), 5);
}

fn test_topla_negatif() {
    assert_eq(topla(-1, 1), 0);
}
"#;

const CONFIG: &str = r#"# hudunit.toml — tüm alanlar opsiyoneldir; bu dosya olmadan da çalışır.

test_dir = "tests"
# extensions = ["hspec"]   # hhs/hud/hudhud her zaman dahildir
timeout_ms = 10000
fuel = 500000000
# coverage_threshold = 80.0
"#;

/// Create the scaffold under `dir`. Files that already exist are reported
/// in `skipped` and left untouched.
pub fn init(dir: &Path) -> std::io::Result<InitReport> {
    let mut report = InitReport::default();
    let entries: &[(&str, &str)] = &[
        ("src/math.hud", SRC_MATH),
        ("tests/math/test_math.hud", TEST_MATH),
        ("hudunit.toml", CONFIG),
    ];
    for (rel, content) in entries {
        let path = dir.join(rel);
        if path.exists() {
            report.skipped.push((*rel).to_string());
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        report.created.push((*rel).to_string());
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::HudunitConfig;

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
        let files = crate::discover::collect_files(&[dir.join("tests")], &cfg);
        assert_eq!(files.len(), 1);
        let discovered: Vec<_> = files
            .iter()
            .map(|p| crate::discover::discover_file(p, None).unwrap())
            .collect();
        let opts = crate::runner::RunOptions::default();
        let outcome = crate::runner::run(&discovered, &cfg, &opts);
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
}
