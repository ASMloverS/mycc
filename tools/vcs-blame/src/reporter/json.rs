use std::io::Write;

use crate::blame::BlameResult;
use crate::reporter::Reporter;
use crate::util::{AppError, IoResultExt, short_commit_id};

pub struct JsonReporter;

impl Reporter for JsonReporter {
    fn render(&self, result: &BlameResult, out: &mut dyn Write) -> Result<(), AppError> {
        let mut root = serde_json::Map::new();

        let json_entries: Vec<serde_json::Value> = result
            .entries
            .iter()
            .map(|e| {
                let mut obj = serde_json::Map::new();
                obj.insert("file".into(), serde_json::Value::String(e.file.clone()));
                obj.insert("line".into(), serde_json::Value::Number(e.line.into()));
                obj.insert("author".into(), serde_json::Value::String(e.author.clone()));
                obj.insert("author_mail".into(), serde_json::Value::String(e.author_mail.clone()));
                obj.insert(
                    "date".into(),
                    serde_json::Value::String(e.author_time.format("%Y-%m-%d").to_string()),
                );
                let commit_str = match e.vcs {
                    crate::blame::VcsKind::Git => format!("git:{}", short_commit_id(&e.commit_id)),
                    crate::blame::VcsKind::Svn => format!("svn:r{}", e.commit_id),
                };
                obj.insert("commit".into(), serde_json::Value::String(commit_str));
                obj.insert("summary".into(), serde_json::Value::String(e.summary.clone()));
                obj.insert("content".into(), serde_json::Value::String(e.content.clone()));
                serde_json::Value::Object(obj)
            })
            .collect();

        let json_summary: Vec<serde_json::Value> = result
            .summary
            .iter()
            .map(|s| {
                let mut obj = serde_json::Map::new();
                obj.insert("author".into(), serde_json::Value::String(s.author.clone()));
                obj.insert("mail".into(), serde_json::Value::String(s.mail.clone()));
                obj.insert(
                    "commit_count".into(),
                    serde_json::Value::Number(s.commit_count.into()),
                );
                obj.insert(
                    "score".into(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(s.score).unwrap_or(serde_json::Number::from(0)),
                    ),
                );
                obj.insert(
                    "latest_commit".into(),
                    serde_json::Value::String(s.latest_commit.clone()),
                );
                obj.insert(
                    "latest_date".into(),
                    serde_json::Value::String(s.latest_time.format("%Y-%m-%d").to_string()),
                );
                obj.insert("files".into(), serde_json::Value::Array(
                    s.files.iter().map(|f| serde_json::Value::String(f.clone())).collect(),
                ));
                obj.insert("lines".into(), serde_json::Value::Array(
                    s.lines.iter().map(|l| serde_json::Value::Number((*l).into())).collect(),
                ));
                serde_json::Value::Object(obj)
            })
            .collect();

        root.insert("entries".into(), serde_json::Value::Array(json_entries));
        root.insert("summary".into(), serde_json::Value::Array(json_summary));
        if let Some(ref r) = result.suggested_responsible {
            root.insert(
                "suggested_responsible".into(),
                serde_json::Value::String(r.clone()),
            );
        }

        let json = serde_json::Value::Object(root);
        writeln!(out, "{}", serde_json::to_string_pretty(&json).unwrap()).map_io_err()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blame::{AuthorSummary, BlameEntry, BlameResult, VcsKind};

    fn make_result() -> BlameResult {
        BlameResult {
            entries: vec![BlameEntry {
                file: "test.py".into(),
                line: 10,
                author: "alice".into(),
                author_mail: "alice@ex.com".into(),
                author_time: chrono::DateTime::from_timestamp(1710460800, 0).map(|dt| dt.naive_utc()).unwrap(),
                vcs: VcsKind::Git,
                commit_id: "abcdef1234567890".into(),
                summary: "fix".into(),
                content: "code".into(),
            }],
            summary: vec![AuthorSummary {
                author: "alice".into(),
                mail: "alice@ex.com".into(),
                commit_count: 1,
                score: 0.75,
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
    fn test_json_reporter() {
        let reporter = JsonReporter;
        let result = make_result();
        let mut buf = Vec::new();
        reporter.render(&result, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["entries"][0]["author"], "alice");
        assert_eq!(parsed["entries"][0]["commit"], "git:abcdef1");
        assert_eq!(parsed["suggested_responsible"], "alice");
    }
}
