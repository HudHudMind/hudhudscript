//! JUnit-style XML report (`--junit path`) for CI systems that ingest the
//! de-facto standard format (GitHub Actions test annotations, GitLab,
//! Jenkins, …). One `<testsuite>` per file, one `<testcase>` per test.

use std::io::Write;
use std::path::Path;

use crate::runner::SuiteOutcome;

/// Render the report as a JUnit XML document.
pub fn render(outcome: &SuiteOutcome) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let total = outcome.tests().count();
    let failures = outcome.failed_count();
    let errors = outcome.files.iter().filter(|f| f.error.is_some()).count();
    let skipped = outcome.skipped();
    let duration_s = outcome.duration.as_secs_f64();
    xml.push_str(&format!(
        "<testsuites name=\"hudunit\" tests=\"{}\" failures=\"{}\" errors=\"{}\" skipped=\"{}\" time=\"{:.3}\">\n",
        total, failures, errors, skipped, duration_s
    ));

    for file in &outcome.files {
        let file_failures = file
            .tests
            .iter()
            .filter(|t| matches!(t.outcome, crate::runner::Outcome::Failed(_)))
            .count();
        let file_skipped = file
            .tests
            .iter()
            .filter(|t| t.outcome == crate::runner::Outcome::Skipped)
            .count();
        let file_time: f64 = file.tests.iter().map(|t| t.duration.as_secs_f64()).sum();
        xml.push_str(&format!(
            "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" skipped=\"{}\" time=\"{:.3}\">\n",
            escape(&file.path.display().to_string()),
            file.tests.len(),
            file_failures,
            file_skipped,
            file_time
        ));
        if let Some(error) = &file.error {
            xml.push_str(&format!(
                "    <testcase name=\"{}\" classname=\"{}\">",
                escape(&file.path.display().to_string()),
                escape(&file.group)
            ));
            xml.push_str(&format!(
                "<error message=\"{}\"/></testcase>\n",
                escape(error)
            ));
        }
        for test in &file.tests {
            xml.push_str(&format!(
                "    <testcase name=\"{}\" classname=\"{}\" time=\"{:.3}\">",
                escape(&test.name),
                escape(&file.group),
                test.duration.as_secs_f64()
            ));
            match &test.outcome {
                crate::runner::Outcome::Passed => {}
                crate::runner::Outcome::Skipped => {
                    xml.push_str("<skipped/>");
                }
                crate::runner::Outcome::Failed(message) => {
                    xml.push_str(&format!(
                        "<failure message=\"{}\">{}</failure>",
                        escape(message),
                        escape(message)
                    ));
                }
            }
            xml.push_str("</testcase>\n");
        }
        xml.push_str("  </testsuite>\n");
    }
    xml.push_str("</testsuites>\n");
    xml
}

/// Write the JUnit XML to `path`, creating parent directories as needed.
pub fn write_to(path: &Path, outcome: &SuiteOutcome) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::File::create(path)?;
    file.write_all(render(outcome).as_bytes())?;
    Ok(())
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{FileOutcome, Outcome, TestOutcome};
    use std::time::Duration;

    #[test]
    fn junit_structure_and_escaping() {
        let suite = SuiteOutcome {
            files: vec![FileOutcome {
                path: std::path::PathBuf::from("/t/tests/test_a.hud"),
                group: "math".into(),
                tests: vec![
                    TestOutcome {
                        name: "test_ok".into(),
                        outcome: Outcome::Passed,
                        duration: Duration::from_millis(2),
                        groups: vec![],
                        line: 1,
                        output: None,
                    },
                    TestOutcome {
                        name: "test_bozuk".into(),
                        outcome: Outcome::Failed("expected 2 <&> got 1".into()),
                        duration: Duration::from_millis(1),
                        groups: vec![],
                        line: 5,
                        output: None,
                    },
                    TestOutcome {
                        name: "test_atla".into(),
                        outcome: Outcome::Skipped,
                        duration: Duration::ZERO,
                        groups: vec![],
                        line: 9,
                        output: None,
                    },
                ],
                error: None,
                fail_fast_stopped: false,
            }],
            duration: Duration::from_millis(3),
            ..Default::default()
        };
        let xml = render(&suite);
        assert!(xml.contains("<testsuites name=\"hudunit\" tests=\"3\" failures=\"1\""));
        assert!(xml.contains("<testsuite name=\"/t/tests/test_a.hud\""));
        assert!(xml.contains("<testcase name=\"test_ok\" classname=\"math\""));
        assert!(xml.contains("<skipped/>"));
        assert!(xml.contains("<failure message=\"expected 2 &lt;&amp;&gt; got 1\">"));
        // Valid XML: no raw angle brackets inside attribute values.
        assert!(!xml.contains("message=\"expected 2 <"));
    }
}
