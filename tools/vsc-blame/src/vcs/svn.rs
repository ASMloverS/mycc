use std::sync::LazyLock;

use chrono::NaiveDateTime;
use regex::Regex;

use crate::blame::{BlameEntry, LineSpec, VcsKind};
use crate::util::AppError;
use crate::vcs::{FileDiff, VcsBackend};

static SVN_BLAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(\d+)\s+(\S+)\s(.*)$").unwrap()
});
static SVN_LOGENTRY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<logentry\s+revision="([^"]+)">([\s\S]*?)</logentry>"#).unwrap()
});
static SVN_AUTHOR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<author>([^<]+)</author>").unwrap()
});
static SVN_DATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<date>([^<]+)</date>").unwrap()
});
static SVN_MSG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<msg>([\s\S]*?)</msg>").unwrap()
});

pub struct SvnBackend;

impl VcsBackend for SvnBackend {
    fn name(&self) -> &str {
        "svn"
    }

    fn kind(&self) -> VcsKind {
        VcsKind::Svn
    }

    fn blame_file(&self, file: &str, lines: &LineSpec) -> Result<Vec<BlameEntry>, AppError> {
        let mut cmd = std::process::Command::new("svn");
        cmd.arg("blame").arg("-v");
        if file.starts_with('-') {
            cmd.arg("--");
        }
        cmd.arg(file);

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::vcs_not_found("svn not found in PATH")
            } else {
                AppError::error(format!("failed to run svn blame: {}", e))
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not found") || stderr.contains("path not found") {
                return Err(AppError::file_not_found(format!(
                    "file not found: {}",
                    file
                )));
            }
            return Err(AppError::error(format!("svn blame failed: {}", stderr.trim())));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let blame_lines = parse_svn_blame(&stdout);
        let min_rev = blame_lines
            .iter()
            .filter_map(|l| l.revision.parse::<u64>().ok())
            .min()
            .unwrap_or(1);
        let max_rev = blame_lines
            .iter()
            .filter_map(|l| l.revision.parse::<u64>().ok())
            .max()
            .unwrap_or(1);

        let log_metadata = self.fetch_log_metadata(file, min_rev, max_rev)?;

        let entries: Vec<BlameEntry> = blame_lines
            .into_iter()
            .filter(|l| lines.contains(l.line_number))
            .map(|l| {
                let meta = log_metadata.get(&l.revision);
                BlameEntry {
                    file: file.to_string(),
                    line: l.line_number,
                    author: meta.map(|m| m.author.clone()).unwrap_or(l.author),
                    author_mail: String::new(),
                    author_time: meta
                        .map(|m| m.date)
                        .unwrap_or_default(),
                    vcs: VcsKind::Svn,
                    commit_id: l.revision,
                    summary: meta.map(|m| m.message.clone()).unwrap_or_default(),
                    content: l.content,
                }
            })
            .collect();

        if entries.is_empty() {
            return Err(AppError::empty("no blame entries matched the specified lines"));
        }

        Ok(entries)
    }

    fn diff_revisions(&self, base: &str, head: &str) -> Result<Vec<FileDiff>, AppError> {
        crate::util::validate_ref(base)?;
        crate::util::validate_ref(head)?;

        let output = std::process::Command::new("svn")
            .arg("diff")
            .arg(format!("-r{}:{}", base, head))
            .output()
            .map_err(|e| AppError::error(format!("failed to run svn diff: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::error(format!("svn diff failed: {}", stderr.trim())));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        crate::parser::diff::parse_unified_diff(&stdout)
    }
}

impl SvnBackend {
    fn fetch_log_metadata(
        &self,
        file: &str,
        min_rev: u64,
        max_rev: u64,
    ) -> Result<std::collections::HashMap<String, LogEntry>, AppError> {
        let output = std::process::Command::new("svn")
            .arg("log")
            .arg("-r")
            .arg(format!("{}:{}", min_rev, max_rev))
            .arg("--xml")
            .arg(file)
            .output()
            .map_err(|e| AppError::error(format!("failed to run svn log: {}", e)))?;

        if !output.status.success() {
            return Ok(std::collections::HashMap::new());
        }

        let xml = String::from_utf8_lossy(&output.stdout);
        Ok(parse_svn_log_xml(&xml))
    }
}

struct BlameLine {
    line_number: usize,
    revision: String,
    author: String,
    content: String,
}

struct LogEntry {
    author: String,
    date: NaiveDateTime,
    message: String,
}

fn parse_svn_blame(output: &str) -> Vec<BlameLine> {
    output
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            SVN_BLAME_RE.captures(line).map(|cap| BlameLine {
                line_number: i + 1,
                revision: cap[1].to_string(),
                author: cap[2].to_string(),
                content: cap[3].to_string(),
            })
        })
        .collect()
}

fn parse_svn_log_xml(xml: &str) -> std::collections::HashMap<String, LogEntry> {
    let mut map = std::collections::HashMap::new();

    for cap in SVN_LOGENTRY_RE.captures_iter(xml) {
        let rev = cap[1].to_string();
        let body = &cap[2];

        let author = SVN_AUTHOR_RE
            .captures(body)
            .map(|c| c[1].to_string())
            .unwrap_or_default();

        let date = SVN_DATE_RE
            .captures(body)
            .and_then(|c| {
                let ds = &c[1];
                let cleaned = ds.split('.').next().unwrap_or(ds);
                NaiveDateTime::parse_from_str(cleaned, "%Y-%m-%dT%H:%M:%S").ok()
            })
            .unwrap_or_default();

        let message = SVN_MSG_RE
            .captures(body)
            .map(|c| c[1].trim().to_string())
            .unwrap_or_default();

        map.insert(rev, LogEntry { author, date, message });
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_svn_blame() {
        let input = "\
     5    john line one
     8    jane line two
    12    john line three";
        let lines = parse_svn_blame(input);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].revision, "5");
        assert_eq!(lines[0].author, "john");
        assert_eq!(lines[0].content, "line one");
        assert_eq!(lines[1].revision, "8");
        assert_eq!(lines[2].line_number, 3);
    }

    #[test]
    fn test_parse_svn_log_xml() {
        let xml = r#"<?xml version="1.0"?>
<log>
<logentry revision="5">
<author>john</author>
<date>2024-03-15T10:00:00.000000Z</date>
<msg>initial commit</msg>
</logentry>
<logentry revision="8">
<author>jane</author>
<date>2024-04-02T14:30:00.000000Z</date>
<msg>fix bug</msg>
</logentry>
</log>"#;
        let map = parse_svn_log_xml(xml);
        assert_eq!(map.len(), 2);
        assert_eq!(map["5"].author, "john");
        assert_eq!(map["5"].message, "initial commit");
        assert_eq!(map["8"].author, "jane");
    }
}
