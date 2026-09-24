//! Colored, group-grouped console output (pytest/phpunit style).

use std::io::IsTerminal;
use std::io::Write;

use crate::runner::SuiteOutcome;

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// Console renderer. `color` overrides auto-detection (TTY check); CI
/// environments without a TTY get plain output by default.
pub struct ConsoleReporter {
    color: bool,
    verbose: bool,
    quiet: bool,
}

impl ConsoleReporter {
    pub fn new(color: bool, verbose: bool, quiet: bool) -> Self {
        Self { color, verbose, quiet }
    }

    pub fn auto(verbose: bool, quiet: bool) -> Self {
        // NO_COLOR (https://no-color.org) forces plain output even on a TTY.
        let color =
            std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
        Self::new(color, verbose, quiet)
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("{}{}{}", code, text, RESET)
        } else {
            text.to_string()
        }
    }

    /// Emit the run to stdout as it is being reported (post-run rendering).
    pub fn report(&self, outcome: &SuiteOutcome) {
        let mut out = std::io::stdout().lock();
        let _ = self.render(&mut out, outcome);
    }

    pub fn render(&self, out: &mut impl Write, outcome: &SuiteOutcome) -> std::io::Result<()> {
        if !self.quiet {
            for file in &outcome.files {
                if file.tests.is_empty() && file.error.is_none() {
                    continue;
                }
                writeln!(
                    out,
                    "{} {} {}",
                    self.paint(DIM, "──"),
                    self.paint(CYAN, &file.group),
                    self.paint(DIM, &format!("({})", file.path.display()))
                )?;
                if let Some(error) = &file.error {
                    writeln!(
                        out,
                        "  {} {}",
                        self.paint(RED, "✗"),
                        self.paint(RED, error)
                    )?;
                }
                for test in &file.tests {
                    self.render_test(out, &file.path, test)?;
                }
            }
            writeln!(out)?;
            self.render_summary(out, outcome)?;
        } else {
            self.render_summary(out, outcome)?;
        }
        out.flush()
    }

    fn render_test(
        &self,
        out: &mut impl Write,
        file_path: &std::path::Path,
        test: &crate::runner::TestOutcome,
    ) -> std::io::Result<()> {
        use crate::runner::Outcome;
        let duration = format_duration(test.duration);
        match &test.outcome {
            Outcome::Passed => {
                let tags = self.tag_suffix(test);
                writeln!(
                    out,
                    "  {} {:<40} {} {}",
                    self.paint(GREEN, "✓"),
                    test.name,
                    self.paint(DIM, &duration),
                    tags
                )?;
                if self.verbose {
                    if let Some(output) = &test.output {
                        writeln!(out, "      {} çıktı:", self.paint(DIM, "·"))?;
                        for line in output.lines() {
                            writeln!(out, "        {}", self.paint(DIM, line))?;
                        }
                    }
                }
                Ok(())
            }
            Outcome::Skipped => writeln!(
                out,
                "  {} {:<40}",
                self.paint(YELLOW, "s"),
                test.name
            ),
            Outcome::Failed(message) => {
                let location = format!("{}:{}", short_path(file_path), test.line);
                writeln!(
                    out,
                    "  {} {:<40} {}",
                    self.paint(RED, "✗"),
                    test.name,
                    self.paint(RED, &location)
                )?;
                if self.verbose || !self.quiet {
                    writeln!(
                        out,
                        "      {} {}",
                        self.paint(RED, "→"),
                        indent_continuations(message)
                    )?;
                    if let Some(output) = &test.output {
                        writeln!(out, "      {} çıktı:", self.paint(DIM, "·"))?;
                        for line in output.lines() {
                            writeln!(out, "        {}", self.paint(DIM, line))?;
                        }
                    }
                }
                Ok(())
            }
        }
    }

    fn tag_suffix(&self, test: &crate::runner::TestOutcome) -> String {
        if test.groups.is_empty() {
            String::new()
        } else {
            let tags: Vec<String> = test
                .groups
                .iter()
                .map(|g| format!("[{}]", g))
                .collect();
            self.paint(DIM, &tags.join(" "))
        }
    }

    fn render_summary(&self, out: &mut impl Write, outcome: &SuiteOutcome) -> std::io::Result<()> {
        let total = outcome.tests().count();
        let passed = outcome.passed();
        let failed = outcome.failed_count();
        let skipped = outcome.skipped();
        let file_errors = outcome.files.iter().filter(|f| f.error.is_some()).count();

        writeln!(
            out,
            "{}",
            self.paint(DIM, &"─".repeat(52))
        )?;
        let summary_line = format!(
            "{} test: {} {}, {} {}, {} {} ({})",
            total,
            passed,
            self.paint(GREEN, "geçti"),
            failed,
            self.paint(if failed > 0 { RED } else { DIM }, "başarısız"),
            skipped,
            self.paint(YELLOW, "atlandı"),
            format_duration(outcome.duration)
        );
        if failed > 0 || file_errors > 0 || outcome.coverage_threshold_failed {
            writeln!(out, "{}", self.paint(RED, &summary_line))?;
        } else {
            writeln!(out, "{}", self.paint(GREEN, &summary_line))?;
        }

        if file_errors > 0 {
            writeln!(
                out,
                "{} {} dosya yüklenemedi",
                self.paint(RED, "✗"),
                file_errors
            )?;
        }

        if outcome.aborted {
            writeln!(
                out,
                "{} fail-fast: ilk başarısızda duruldu, kalan testler koşulmadı",
                self.paint(YELLOW, "!»")
            )?;
        }

        if self.verbose {
            let mut slowest: Vec<&crate::runner::TestOutcome> =
                outcome.tests().collect();
            slowest.sort_by(|a, b| b.duration.cmp(&a.duration));
            writeln!(out, "{}", self.paint(DIM, "en yavaşlar:"))?;
            for test in slowest.iter().take(5) {
                writeln!(
                    out,
                    "  {} {:<40} {}",
                    self.paint(DIM, "·"),
                    test.name,
                    self.paint(DIM, &format_duration(test.duration))
                )?;
            }
        }

        if let Some(coverage) = &outcome.coverage {
            writeln!(
                out,
                "Coverage: {} satır, {} fonksiyon kapsandı",
                self.paint(
                    if coverage.overall_line_percent >= 100.0 { GREEN } else { YELLOW },
                    &format!("{:.1}%", coverage.overall_line_percent)
                ),
                self.paint(
                    CYAN,
                    &format!(
                        "{}/{}",
                        coverage.functions.iter().filter(|f| f.covered).count(),
                        coverage.functions.len()
                    )
                )
            )?;
            if outcome.coverage_threshold_failed {
                writeln!(
                    out,
                    "{}",
                    self.paint(RED, "Coverage eşiği aşılamadı")
                )?;
            }
        }
        Ok(())
    }
}

fn short_path(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string()
}

fn format_duration(duration: std::time::Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1_000 {
        format!("{}µs", micros)
    } else if micros < 1_000_000 {
        format!("{:.1}ms", micros as f64 / 1_000.0)
    } else {
        format!("{:.2}s", micros as f64 / 1_000_000.0)
    }
}

fn indent_continuations(message: &str) -> String {
    message
        .lines()
        .enumerate()
        .map(|(i, line)| if i == 0 { line.to_string() } else { format!("      {}", line) })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_rendering_avoids_ansi_codes() {
        use crate::runner::{FileOutcome, Outcome, TestOutcome};
        use std::time::Duration;
        let suite = SuiteOutcome {
            files: vec![FileOutcome {
                path: std::path::PathBuf::from("/t/tests/math/test_a.hud"),
                group: "math".into(),
                tests: vec![TestOutcome {
                    name: "test_x".into(),
                    outcome: Outcome::Passed,
                    duration: Duration::from_micros(12),
                    groups: vec!["math".into()],
                    line: 3,
                    output: None,
                }],
                error: None,
                fail_fast_stopped: false,
            }],
            duration: Duration::from_millis(5),
            ..Default::default()
        };
        let reporter = ConsoleReporter::new(false, false, false);
        let mut buf: Vec<u8> = Vec::new();
        reporter.render(&mut buf, &suite).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(!text.contains("\x1b["));
        assert!(text.contains("test_x"));
        assert!(text.contains("1 test: 1 geçti"));
    }
}
