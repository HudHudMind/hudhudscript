//! Self-contained HTML report: inline CSS/JS, no external assets. Includes
//! summary cards, collapsible per-group test lists, and per-file source
//! listing with a line-level coverage heat map.

use std::io::Write;
use std::path::Path;

use crate::runner::SuiteOutcome;

/// Render the full HTML document for a finished run.
pub fn render(outcome: &SuiteOutcome) -> String {
    let total = outcome.tests().count();
    let passed = outcome.passed();
    let failed = outcome.failed_count();
    let skipped = outcome.skipped();

    let mut body = String::new();

    // Summary cards.
    body.push_str("<section class=\"cards\">");
    body.push_str(&card("Toplam", &total.to_string(), "neutral"));
    body.push_str(&card("Geçti", &passed.to_string(), "good"));
    body.push_str(&card(
        "Başarısız",
        &failed.to_string(),
        if failed > 0 { "bad" } else { "neutral" },
    ));
    body.push_str(&card("Atlandı", &skipped.to_string(), "warn"));
    if let Some(coverage) = &outcome.coverage {
        body.push_str(&card(
            "Satır Coverage",
            &format!("{:.1}%", coverage.overall_line_percent),
            if coverage.overall_line_percent >= 80.0 { "good" } else { "warn" },
        ));
        let covered = coverage.functions.iter().filter(|f| f.covered).count();
        body.push_str(&card(
            "Fonksiyon Coverage",
            &format!("{}/{}", covered, coverage.functions.len()),
            "neutral",
        ));
    }
    body.push_str("</section>");

    // Test list per group.
    body.push_str("<h2>Testler</h2>");
    for file in &outcome.files {
        if file.tests.is_empty() && file.error.is_none() {
            continue;
        }
        let file_group = escape(&file.group);
        let file_path = escape(&file.path.display().to_string());
        let file_passed = file.tests.iter().filter(|t| t.outcome == crate::runner::Outcome::Passed).count();
        let file_total = file.tests.len();
        let summary_badge = if let Some(error) = &file.error {
            format!("<span class=\"badge bad\">hata: {}</span>", escape(error))
        } else if file_passed == file_total {
            "<span class=\"badge good\">ok</span>".to_string()
        } else {
            format!("<span class=\"badge bad\">{}/{} geçti</span>", file_passed, file_total)
        };
        body.push_str(&format!(
            "<details class=\"group\" open><summary>{} <span class=\"path\">{}</span> {}</summary><table>",
            file_group, file_path, summary_badge
        ));
        if let Some(error) = &file.error {
            body.push_str(&format!(
                "<tr class=\"failed\"><td class=\"status\">✗</td><td colspan=\"3\" class=\"msg\">{}</td></tr>",
                escape(error)
            ));
        }
        for test in &file.tests {
            let (row_class, status, message) = match &test.outcome {
                crate::runner::Outcome::Passed => ("passed", "✓", String::new()),
                crate::runner::Outcome::Skipped => ("skipped", "s", String::new()),
                crate::runner::Outcome::Failed(message) => {
                    ("failed", "✗", escape(message))
                }
            };
            let duration_ms = test.duration.as_secs_f64() * 1000.0;
            let tags = test
                .groups
                .iter()
                .map(|g| format!("<span class=\"tag\">{}</span>", escape(g)))
                .collect::<Vec<_>>()
                .join("");
            body.push_str(&format!(
                "<tr class=\"{}\"><td class=\"status\">{}</td><td class=\"name\">{} {}</td><td class=\"loc\">{}:{}</td><td class=\"time\">{:.2} ms</td></tr>",
                row_class,
                status,
                escape(&test.name),
                tags,
                escape(&file.path.file_name().and_then(|n| n.to_str()).unwrap_or("?")),
                test.line,
                duration_ms
            ));
            if !message.is_empty() {
                body.push_str(&format!(
                    "<tr class=\"{}\"><td></td><td colspan=\"3\" class=\"msg\">{}</td></tr>",
                    row_class, message
                ));
            }
            if let Some(output) = &test.output {
                body.push_str(&format!(
                    "<tr class=\"{}\"><td></td><td colspan=\"3\" class=\"out\">{}</td></tr>",
                    row_class,
                    escape(output)
                ));
            }
        }
        body.push_str("</table></details>");
    }

    // Coverage source views.
    if let Some(coverage) = &outcome.coverage {
        if !coverage.files.is_empty() {
            body.push_str("<h2>Coverage (kaynak)</h2>");
            for file_cov in &coverage.files {
                let executable: std::collections::HashSet<usize> =
                    file_cov.executable_lines.iter().copied().collect();
                let covered: std::collections::HashSet<usize> =
                    file_cov.covered_lines.iter().copied().collect();
                let hits = &file_cov.hit_counts;
                let source = std::fs::read_to_string(&file_cov.path).unwrap_or_default();
                let percent = file_cov.line_percent();
                let badge = if percent >= 80.0 { "good" } else if percent >= 50.0 { "warn" } else { "bad" };
                body.push_str(&format!(
                    "<details class=\"covfile\"><summary>{} <span class=\"badge {}\">{:.1}%</span></summary><pre><code>",
                    escape(&file_cov.path),
                    badge,
                    percent
                ));
                for (idx, line) in source.lines().enumerate() {
                    let line_no = idx + 1;
                    let class = if covered.contains(&line_no) {
                        "cov-hit"
                    } else if executable.contains(&line_no) {
                        "cov-miss"
                    } else {
                        "cov-none"
                    };
                    let heat = hits
                        .get(&line_no)
                        .map(|h| format!(" data-hits=\"{}\"", h))
                        .unwrap_or_default();
                    body.push_str(&format!(
                        "<span class=\"{}\"{}>{:>4}│ {}</span>\n",
                        class,
                        heat,
                        line_no,
                        escape(line)
                    ));
                }
                body.push_str("</code></pre></details>");
            }
        }
        if !coverage.functions.is_empty() {
            body.push_str("<h2>Fonksiyon Coverage</h2><table class=\"fn\">");
            for func in &coverage.functions {
                body.push_str(&format!(
                    "<tr class=\"{}\"><td>{}</td><td class=\"path\">{}</td></tr>",
                    if func.covered { "passed" } else { "failed" },
                    escape(&func.name),
                    escape(&func.file)
                ));
            }
            body.push_str("</table>");
        }
    }

    let html = format!(
        "<!DOCTYPE html><html lang=\"tr\"><head><meta charset=\"utf-8\">\
<title>hudunit raporu</title>\
<style>{STYLE}</style></head><body>\
<h1>hudunit <span class=\"path\">{when}</span></h1>\
{body}\
<footer>hudunit — HudHudScript unit test raporu</footer>\
</body></html>",
        when = escape(&humantime_now()),
        body = body,
        STYLE = STYLE,
    );
    html
}

/// Write the HTML report to `path`, creating parent directories as needed.
pub fn write_to(path: &Path, outcome: &SuiteOutcome) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::File::create(path)?;
    file.write_all(render(outcome).as_bytes())?;
    Ok(())
}

fn humantime_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // ISO-8601 via manual civil-from-days conversion; avoids a chrono dep.
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (year, month, day) = civil_from_days(days as i64);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        year, month, day, h, m, s
    )
}

/// Howard Hinnant's civil-from-days algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn card(label: &str, value: &str, class: &str) -> String {
    format!(
        "<div class=\"card {}\"><div class=\"value\">{}</div><div class=\"label\">{}</div></div>",
        class, value, label
    )
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

const STYLE: &str = "\
:root{color-scheme:light dark}\
body{font-family:system-ui,sans-serif;margin:2rem;max-width:60rem}\
h1{font-size:1.5rem}h2{margin-top:2rem}\
.cards{display:flex;gap:1rem;flex-wrap:wrap}\
.card{padding:1rem 1.5rem;border-radius:.5rem;background:rgba(127,127,127,.1)}\
.card .value{font-size:1.8rem;font-weight:700}\
.card .label{font-size:.8rem;opacity:.7}\
.card.good .value{color:#2e9e44}.card.bad .value{color:#c62828}.card.warn .value{color:#b58900}\
table{border-collapse:collapse;width:100%}\
td,th{padding:.25rem .5rem;text-align:left;vertical-align:top}\
tr.passed .status{color:#2e9e44}tr.failed .status{color:#c62828}tr.skipped .status{color:#b58900}\
tr.failed .msg{color:#c62828;white-space:pre-wrap}\
tr .out{font-family:ui-monospace,monospace;font-size:.75rem;white-space:pre-wrap;opacity:.75;background:rgba(127,127,127,.06)}\
.name{font-family:ui-monospace,monospace}\
.path{font-family:ui-monospace,monospace;font-size:.8rem;opacity:.7}\
.loc{font-family:ui-monospace,monospace;font-size:.8rem}\
.time{font-family:ui-monospace,monospace;font-size:.8rem;white-space:nowrap}\
.tag{display:inline-block;margin-left:.3rem;padding:0 .35rem;border-radius:.3rem;background:rgba(127,127,127,.15);font-size:.7rem}\
.badge{margin-left:.5rem;padding:.1rem .5rem;border-radius:1rem;font-size:.75rem}\
.badge.good{background:#2e9e4433}.badge.bad{background:#c6282833}.badge.warn{background:#b5890033}\
details{margin:.75rem 0}summary{cursor:pointer;font-weight:600}\
pre{background:rgba(127,127,127,.08);padding:.75rem;border-radius:.4rem;overflow-x:auto}\
pre code{font-family:ui-monospace,monospace;font-size:.8rem}\
.cov-hit{background:#2e9e4422;display:block}\
.cov-miss{background:#c6282822;display:block}\
.cov-none{display:block}\
footer{margin-top:3rem;opacity:.6;font-size:.8rem}\
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_html() {
        assert_eq!(escape("<script>&'\""), "&lt;script&gt;&amp;&#39;&quot;");
    }

    #[test]
    fn civil_from_days_epoch() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_723), (2024, 1, 1));
    }

    #[test]
    fn html_report_renders_cards() {
        use crate::runner::{FileOutcome, Outcome, TestOutcome};
        use std::time::Duration;
        let suite = SuiteOutcome {
            files: vec![FileOutcome {
                path: std::path::PathBuf::from("/t/tests/test_a.hud"),
                group: "test_a".into(),
                tests: vec![TestOutcome {
                    name: "test_x".into(),
                    outcome: Outcome::Passed,
                    duration: Duration::from_millis(2),
                    groups: vec![],
                    line: 1,
                    output: None,
                }],
                error: None,
                fail_fast_stopped: false,
            }],
            duration: Duration::from_millis(4),
            ..Default::default()
        };
        let html = render(&suite);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("test_x"));
        assert!(html.contains("1 test") || html.contains("Geçti"));
    }
}
