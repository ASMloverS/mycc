use std::io::Write;

use colored::Colorize;

use crate::blame::BlameResult;
use crate::reporter::Reporter;
use crate::util::{AppError, IoResultExt, short_commit_id};

pub struct TextReporter {
    pub no_color: bool,
}

impl Reporter for TextReporter {
    fn render(&self, result: &BlameResult, out: &mut dyn Write) -> Result<(), AppError> {
        if self.no_color {
            colored::control::set_override(false);
        }

        if result.entries.is_empty() {
            return Ok(());
        }

        let file = &result.entries[0].file;
        writeln!(out, "File: {}", file).map_io_err()?;
        writeln!(out, "{}", "─".repeat(51)).map_io_err()?;

        for e in &result.entries {
            let date_str = e.author_time.format("%Y-%m-%d").to_string();
            let short_id = short_commit_id(&e.commit_id);
            writeln!(
                out,
                "Line {} | {} | {} | {} | {}",
                e.line,
                e.author.cyan(),
                date_str.green(),
                short_id.yellow(),
                e.summary.dimmed(),
            )
            .map_io_err()?;
            writeln!(out, "        | {}", e.content.dimmed()).map_io_err()?;
        }

        writeln!(out, "{}", "─".repeat(51)).map_io_err()?;

        if !result.summary.is_empty() {
            writeln!(out).map_io_err()?;
            writeln!(out, "Summary ({} lines):", result.entries.len()).map_io_err()?;

            for (i, s) in result.summary.iter().enumerate() {
                let date_str = s.latest_time.format("%Y-%m-%d").to_string();
                writeln!(
                    out,
                    "  #{} {} ({}) - {} commit{}, latest {}",
                    i + 1,
                    s.author.bold(),
                    s.mail.dimmed(),
                    s.commit_count,
                    if s.commit_count > 1 { "s" } else { "" },
                    date_str,
                )
                .map_io_err()?;
            }
        }

        if let Some(ref responsible) = result.suggested_responsible {
            writeln!(out).map_io_err()?;
            writeln!(
                out,
                "Suggested responsible: {} (most recent change)",
                responsible.bold(),
            )
            .map_io_err()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blame::{AuthorSummary, BlameEntry, VcsKind};

    fn make_result() -> BlameResult {
        BlameResult {
            entries: vec![BlameEntry {
                file: "test.py".into(),
                line: 10,
                author: "alice".into(),
                author_mail: "alice@example.com".into(),
                author_time: chrono::DateTime::from_timestamp(1710460800, 0).map(|dt| dt.naive_utc()).unwrap(),
                vcs: VcsKind::Git,
                commit_id: "abcdef1234567890abcdef1234567890abcdef12".into(),
                summary: "fix: add feature".into(),
                content: "import os".into(),
            }],
            summary: vec![AuthorSummary {
                author: "alice".into(),
                mail: "alice@example.com".into(),
                commit_count: 1,
                score: 0.8,
                latest_time: chrono::DateTime::from_timestamp(1710460800, 0).map(|dt| dt.naive_utc()).unwrap(),
                latest_commit: "abcdef12".into(),
                files: vec!["test.py".into()],
                lines: vec![10],
            }],
            suggested_responsible: Some("alice".into()),
            uncommitted_lines: vec![],
        }
    }

    #[test]
    fn test_text_reporter_no_color() {
        let reporter = TextReporter { no_color: true };
        let result = make_result();
        let mut buf = Vec::new();
        reporter.render(&result, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("File: test.py"));
        assert!(output.contains("alice"));
        assert!(output.contains("Suggested responsible"));
    }
}
