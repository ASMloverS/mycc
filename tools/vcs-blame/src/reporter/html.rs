use std::io::Write;

use crate::blame::BlameResult;
use crate::reporter::Reporter;
use crate::util::{AppError, IoResultExt, short_commit_id};

pub struct HtmlReporter;

impl Reporter for HtmlReporter {
    fn render(&self, result: &BlameResult, out: &mut dyn Write) -> Result<(), AppError> {
        writeln!(out, "<!DOCTYPE html>").map_io_err()?;
        writeln!(out, "<html><head><meta charset=\"utf-8\">").map_io_err()?;
        writeln!(out, "<title>vcs-blame report</title>").map_io_err()?;
        writeln!(out, "<style>").map_io_err()?;
        write!(out, "{}", CSS).map_io_err()?;
        writeln!(out, "</style>").map_io_err()?;
        writeln!(out, "</head><body>").map_io_err()?;
        writeln!(out, "<h1>vcs-blame Report</h1>").map_io_err()?;

        if !result.entries.is_empty() {
            let file = &result.entries[0].file;
            writeln!(out, "<h2>File: {}</h2>", html_escape(file)).map_io_err()?;

            writeln!(
                out,
                "<table><thead><tr><th>Line</th><th>Author</th><th>Date</th><th>Commit</th><th>Summary</th></tr></thead><tbody>"
            )
            .map_io_err()?;

            for e in &result.entries {
                let date_str = e.author_time.format("%Y-%m-%d").to_string();
                writeln!(
                    out,
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    e.line,
                    html_escape(&e.author),
                    date_str,
                    html_escape(short_commit_id(&e.commit_id)),
                    html_escape(&e.summary),
                )
                .map_io_err()?;
            }

            writeln!(out, "</tbody></table>").map_io_err()?;
        }

        if !result.summary.is_empty() {
            writeln!(out, "<h2>Summary</h2>").map_io_err()?;
            writeln!(out, "<table><thead><tr><th>#</th><th>Author</th><th>Commits</th><th>Score</th><th>Latest</th></tr></thead><tbody>")
                .map_io_err()?;

            for (i, s) in result.summary.iter().enumerate() {
                let date_str = s.latest_time.format("%Y-%m-%d").to_string();
                writeln!(
                    out,
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:.3}</td><td>{}</td></tr>",
                    i + 1,
                    html_escape(&s.author),
                    s.commit_count,
                    s.score,
                    date_str,
                )
                .map_io_err()?;
            }

            writeln!(out, "</tbody></table>").map_io_err()?;
        }

        if let Some(ref responsible) = result.suggested_responsible {
            writeln!(
                out,
                "<p><strong>Suggested responsible:</strong> {}</p>",
                html_escape(responsible)
            )
            .map_io_err()?;
        }

        writeln!(out, "</body></html>").map_io_err()?;

        Ok(())
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const CSS: &str = "\
body { font-family: -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, sans-serif; margin: 2em; }
table { border-collapse: collapse; width: 100%; margin-bottom: 1em; }
th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
th { background-color: #f5f5f5; cursor: pointer; }
tr:nth-child(even) { background-color: #fafafa; }
tr:hover { background-color: #f0f0f0; }
h1 { color: #333; }
h2 { color: #555; }
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blame::{BlameEntry, BlameResult, VcsKind};

    fn make_result() -> BlameResult {
        BlameResult {
            entries: vec![BlameEntry {
                file: "test.py".into(),
                line: 10,
                author: "alice".into(),
                author_mail: "a@a.com".into(),
                author_time: chrono::DateTime::from_timestamp(1710460800, 0).map(|dt| dt.naive_utc()).unwrap(),
                vcs: VcsKind::Git,
                commit_id: "abcd1234".into(),
                summary: "fix <bug>".into(),
                content: "code".into(),
            }],
            summary: vec![],
            suggested_responsible: Some("alice".into()),
            uncommitted_lines: vec![],
        }
    }

    #[test]
    fn test_html_reporter() {
        let reporter = HtmlReporter;
        let result = make_result();
        let mut buf = Vec::new();
        reporter.render(&result, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("fix &lt;bug&gt;"));
        assert!(output.contains("Suggested responsible"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a&b"), "a&amp;b");
    }
}
