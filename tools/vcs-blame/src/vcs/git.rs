use chrono::NaiveDateTime;

use crate::blame::{BlameEntry, LineSpec, VcsKind};
use crate::util::AppError;
use crate::vcs::{FileDiff, VcsBackend};

pub struct GitBackend;

impl VcsBackend for GitBackend {
    fn name(&self) -> &str {
        "git"
    }

    fn kind(&self) -> VcsKind {
        VcsKind::Git
    }

    fn blame_file(&self, file: &str, lines: &LineSpec) -> Result<Vec<BlameEntry>, AppError> {
        let mut cmd = std::process::Command::new("git");
        cmd.arg("blame").arg("--porcelain");
        if file.starts_with('-') {
            cmd.arg("--");
        }
        cmd.arg(file);

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::vcs_not_found("git not found in PATH")
            } else {
                AppError::error(format!("failed to run git blame: {}", e))
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("no such path") || stderr.contains("Not a valid object") {
                return Err(AppError::file_not_found(format!(
                    "file not found or not under VCS: {}",
                    file
                )));
            }
            return Err(AppError::error(format!("git blame failed: {}", stderr.trim())));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let all_entries = parse_porcelain(&stdout, file);

        let filtered: Vec<BlameEntry> = all_entries
            .into_iter()
            .filter(|e| lines.contains(e.line))
            .collect();

        if filtered.is_empty() {
            return Err(AppError::empty("no blame entries matched the specified lines"));
        }

        Ok(filtered)
    }

    fn diff_revisions(&self, base: &str, head: &str) -> Result<Vec<FileDiff>, AppError> {
        crate::util::validate_ref(base)?;
        crate::util::validate_ref(head)?;

        let output = std::process::Command::new("git")
            .arg("diff")
            .arg(format!("{}..{}", base, head))
            .output()
            .map_err(|e| AppError::error(format!("failed to run git diff: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::error(format!("git diff failed: {}", stderr.trim())));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        crate::parser::diff::parse_unified_diff(&stdout)
    }
}

struct PendingEntry {
    commit_id: String,
    final_line: usize,
    author: String,
    author_mail: String,
    author_time: NaiveDateTime,
    summary: String,
}

fn parse_header(line: &str) -> Option<(String, usize)> {
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    if parts.len() < 3 {
        return None;
    }
    if parts[0].len() != 40 || !parts[0].chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let final_line: usize = parts[2].parse().ok()?;
    Some((parts[0].to_string(), final_line))
}

pub fn parse_porcelain(output: &str, file: &str) -> Vec<BlameEntry> {
    let mut entries = Vec::new();
    let mut pending: Option<PendingEntry> = None;

    for raw_line in output.lines() {
        if raw_line.is_empty() {
            continue;
        }

        if let Some(content) = raw_line.strip_prefix('\t') {
            if let Some(p) = pending.take() {
                entries.push(BlameEntry {
                    file: file.to_string(),
                    line: p.final_line,
                    author: p.author,
                    author_mail: p.author_mail,
                    author_time: p.author_time,
                    vcs: VcsKind::Git,
                    commit_id: p.commit_id,
                    summary: p.summary,
                    content: content.to_string(),
                });
            }
            continue;
        }

        if let Some((commit_id, final_line)) = parse_header(raw_line) {
            pending = Some(PendingEntry {
                commit_id,
                final_line,
                author: String::new(),
                author_mail: String::new(),
                author_time: NaiveDateTime::default(),
                summary: String::new(),
            });
            continue;
        }

        if let Some(ref mut p) = pending {
            if let Some(v) = raw_line.strip_prefix("author ") {
                p.author = v.to_string();
            } else if let Some(v) = raw_line.strip_prefix("author-mail ") {
                p.author_mail = v
                    .trim_matches(|c| c == '<' || c == '>' || c == ' ')
                    .to_string();
            } else if let Some(v) = raw_line.strip_prefix("author-time ") {
                if let Ok(ts) = v.parse::<i64>() {
                    p.author_time =
                        chrono::DateTime::from_timestamp(ts, 0)
                            .map(|dt| dt.naive_utc())
                            .unwrap_or_default();
                }
            } else if let Some(v) = raw_line.strip_prefix("summary ") {
                p.summary = v.to_string();
            }
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PORCELAIN: &str = "\
abcdef1234567890abcdef1234567890abcdef12 1 1 1
author John Doe
author-mail <john@example.com>
author-time 1710460800
summary initial commit
\tline one content
fedcba2109876543210fedcba2109876543210fe 2 2 1
author Jane Smith
author-mail <jane@example.com>
author-time 1710547200
summary fix bug
\tline two content";

    #[test]
    fn test_parse_porcelain() {
        let entries = parse_porcelain(SAMPLE_PORCELAIN, "test.txt");
        assert_eq!(entries.len(), 2);

        assert_eq!(entries[0].line, 1);
        assert_eq!(entries[0].author, "John Doe");
        assert_eq!(entries[0].author_mail, "john@example.com");
        assert_eq!(entries[0].commit_id, "abcdef1234567890abcdef1234567890abcdef12");
        assert_eq!(entries[0].summary, "initial commit");
        assert_eq!(entries[0].content, "line one content");

        assert_eq!(entries[1].line, 2);
        assert_eq!(entries[1].author, "Jane Smith");
        assert_eq!(entries[1].content, "line two content");
    }

    #[test]
    fn test_parse_porcelain_author_time() {
        let entries = parse_porcelain(SAMPLE_PORCELAIN, "test.txt");
        assert_eq!(
            entries[0].author_time,
            chrono::DateTime::from_timestamp(1710460800, 0).map(|dt| dt.naive_utc()).unwrap()
        );
    }

    #[test]
    fn test_parse_header() {
        let (hash, line) = parse_header("abcdef1234567890abcdef1234567890abcdef12 1 1 1").unwrap();
        assert_eq!(hash, "abcdef1234567890abcdef1234567890abcdef12");
        assert_eq!(line, 1);
    }

    #[test]
    fn test_parse_header_invalid() {
        assert!(parse_header("short 1 1").is_none());
        assert!(parse_header("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz 1 1").is_none());
    }

    #[test]
    fn test_parse_porcelain_uncommitted() {
        let input = "\
0000000000000000000000000000000000000000 1 1 1
author Not Committed Yet
author-mail <not.committed.yet>
author-time 1710460800
summary 
\tnew line";
        let entries = parse_porcelain(input, "test.txt");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].author, "Not Committed Yet");
        assert_eq!(entries[0].commit_id, "0".repeat(40));
    }
}
