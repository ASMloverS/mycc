use std::io::Write;

use crate::blame::BlameResult;
use crate::reporter::Reporter;
use crate::util::{AppError, IoResultExt, short_commit_id};

pub struct MarkdownReporter;

impl Reporter for MarkdownReporter {
    fn render(&self, result: &BlameResult, out: &mut dyn Write) -> Result<(), AppError> {
        if result.entries.is_empty() {
            return Ok(());
        }

        writeln!(out, "| Line | Author | Date | Commit | Summary |").map_io_err()?;
        writeln!(out, "|------|--------|------|--------|---------|").map_io_err()?;

        for e in &result.entries {
            let date_str = e.author_time.format("%Y-%m-%d").to_string();
            writeln!(
                out,
                "| {} | {} | {} | {} | {} |",
                e.line, e.author, date_str, short_commit_id(&e.commit_id), e.summary
            )
            .map_io_err()?;
        }

        if let Some(ref responsible) = result.suggested_responsible {
            writeln!(out).map_io_err()?;
            writeln!(out, "**Suggested responsible:** {}", responsible).map_io_err()?;
        }

        Ok(())
    }
}

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
                author_mail: "alice@ex.com".into(),
                author_time: chrono::DateTime::from_timestamp(1710460800, 0).map(|dt| dt.naive_utc()).unwrap(),
                vcs: VcsKind::Git,
                commit_id: "abcdef12".into(),
                summary: "fix".into(),
                content: "code".into(),
            }],
            summary: vec![],
            suggested_responsible: Some("alice".into()),
            uncommitted_lines: vec![],
        }
    }

    #[test]
    fn test_markdown_reporter() {
        let reporter = MarkdownReporter;
        let result = make_result();
        let mut buf = Vec::new();
        reporter.render(&result, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("| Line | Author | Date | Commit | Summary |"));
        assert!(output.contains("| 10 | alice |"));
        assert!(output.contains("**Suggested responsible:** alice"));
    }
}
