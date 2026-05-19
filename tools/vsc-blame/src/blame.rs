use std::collections::HashMap;

use chrono::NaiveDateTime;
use serde::Serialize;

use crate::util::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VcsKind {
    Git,
    Svn,
}

#[derive(Debug, Clone)]
pub enum LineSpec {
    All,
    Single(usize),
    Range(usize, usize),
    Multi(Vec<(usize, usize)>),
}

impl LineSpec {
    pub fn contains(&self, line: usize) -> bool {
        match self {
            LineSpec::All => true,
            LineSpec::Single(l) => *l == line,
            LineSpec::Range(s, e) => *s <= line && line <= *e,
            LineSpec::Multi(segs) => segs.iter().any(|(s, e)| *s <= line && line <= *e),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BlameEntry {
    pub file: String,
    pub line: usize,
    pub author: String,
    pub author_mail: String,
    #[serde(skip)]
    pub author_time: NaiveDateTime,
    pub vcs: VcsKind,
    pub commit_id: String,
    pub summary: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthorSummary {
    pub author: String,
    pub mail: String,
    pub commit_count: usize,
    pub score: f64,
    #[serde(skip)]
    pub latest_time: NaiveDateTime,
    pub latest_commit: String,
    pub files: Vec<String>,
    pub lines: Vec<usize>,
}

#[derive(Debug)]
pub struct BlameResult {
    pub entries: Vec<BlameEntry>,
    pub summary: Vec<AuthorSummary>,
    pub suggested_responsible: Option<String>,
    pub uncommitted_lines: Vec<(String, usize)>,
}

const W_COMMIT: f64 = 0.4;
const W_RECENCY: f64 = 0.6;
const TAU_DAYS: f64 = 90.0;

pub fn aggregate(
    entries: Vec<BlameEntry>,
    aliases: &HashMap<String, Vec<String>>,
) -> BlameResult {
    let (committed, uncommitted_lines) = filter_uncommitted(entries);

    if committed.is_empty() {
        return BlameResult {
            entries: committed,
            summary: vec![],
            suggested_responsible: None,
            uncommitted_lines,
        };
    }

    let total = committed.len() as f64;
    let now = chrono::Local::now().naive_utc();

    let mut groups: HashMap<String, Vec<&BlameEntry>> = HashMap::new();
    for e in &committed {
        let key = resolve_alias(&e.author, &e.author_mail, aliases);
        groups.entry(key).or_default().push(e);
    }

    let mut summary: Vec<AuthorSummary> = groups
        .into_iter()
        .map(|(author, entries)| {
            let commit_count = entries.len();
            let commit_norm = commit_count as f64 / total;
            let latest = entries.iter().max_by_key(|e| e.author_time).unwrap();
            let dt = now - latest.author_time;
            let dt_days = dt.num_seconds() as f64 / 86400.0;
            let recency = (-dt_days.abs() / TAU_DAYS).exp();
            let score = W_COMMIT * commit_norm + W_RECENCY * recency;

            let mut files: Vec<String> = entries.iter().map(|e| e.file.clone()).collect();
            files.sort();
            files.dedup();
            let mut lines: Vec<usize> = entries.iter().map(|e| e.line).collect();
            lines.sort();

            AuthorSummary {
                mail: latest.author_mail.clone(),
                author,
                commit_count,
                score,
                latest_time: latest.author_time,
                latest_commit: latest.commit_id.clone(),
                files,
                lines,
            }
        })
        .collect();

    summary.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let suggested = summary.first().map(|s| s.author.clone());

    BlameResult {
        entries: committed,
        summary,
        suggested_responsible: suggested,
        uncommitted_lines,
    }
}

const ZERO_HASH: &str = "0000000000000000000000000000000000000000";

fn filter_uncommitted(entries: Vec<BlameEntry>) -> (Vec<BlameEntry>, Vec<(String, usize)>) {
    let mut committed = Vec::new();
    let mut uncommitted = Vec::new();

    for e in entries {
        if e.commit_id == ZERO_HASH || e.author == "Not Committed Yet" {
            uncommitted.push((e.file, e.line));
        } else {
            committed.push(e);
        }
    }

    (committed, uncommitted)
}

fn resolve_alias(
    author: &str,
    mail: &str,
    aliases: &HashMap<String, Vec<String>>,
) -> String {
    let author_lower = author.to_lowercase();
    let mail_lower = mail.to_lowercase();

    for (canonical, alias_list) in aliases {
        for alias in alias_list {
            let alias_lower = alias.to_lowercase();
            if author_lower == alias_lower || mail_lower == alias_lower {
                return canonical.clone();
            }
        }
        if author_lower == canonical.to_lowercase() {
            return canonical.clone();
        }
    }

    author.to_string()
}

pub fn parse_file_spec(spec: &str) -> Result<(String, LineSpec), AppError> {
    let parts: Vec<&str> = spec.rsplitn(2, ':').collect();
    if parts.len() == 1 {
        return Ok((parts[0].to_string(), LineSpec::All));
    }

    let file = parts[1].to_string();
    let line_spec = parts[0];
    if line_spec.contains('-') {
        let range_parts: Vec<&str> = line_spec.splitn(2, '-').collect();
        let start: usize = range_parts[0]
            .parse()
            .map_err(|_| AppError::usage("invalid line number"))?;
        let end: usize = range_parts[1]
            .parse()
            .map_err(|_| AppError::usage("invalid line number"))?;
        if start == 0 || end == 0 || start > end {
            return Err(AppError::usage("invalid line range"));
        }
        Ok((file, LineSpec::Range(start, end)))
    } else {
        let line: usize = line_spec
            .parse()
            .map_err(|_| AppError::usage("invalid line number"))?;
        if line == 0 {
            return Err(AppError::usage("line numbers are 1-based"));
        }
        Ok((file, LineSpec::Single(line)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_spec_all() {
        let (f, spec) = parse_file_spec("foo.py").unwrap();
        assert_eq!(f, "foo.py");
        assert!(matches!(spec, LineSpec::All));
    }

    #[test]
    fn test_parse_file_spec_single() {
        let (f, spec) = parse_file_spec("foo.py:10").unwrap();
        assert_eq!(f, "foo.py");
        assert!(matches!(spec, LineSpec::Single(10)));
    }

    #[test]
    fn test_parse_file_spec_range() {
        let (f, spec) = parse_file_spec("foo.py:10-20").unwrap();
        assert_eq!(f, "foo.py");
        assert!(matches!(spec, LineSpec::Range(10, 20)));
    }

    #[test]
    fn test_parse_file_spec_invalid() {
        assert!(parse_file_spec("foo.py:0").is_err());
        assert!(parse_file_spec("foo.py:5-3").is_err());
        assert!(parse_file_spec("foo.py:abc").is_err());
    }

    #[test]
    fn test_line_spec_contains() {
        assert!(LineSpec::All.contains(42));
        assert!(LineSpec::Single(5).contains(5));
        assert!(!LineSpec::Single(5).contains(6));
        assert!(LineSpec::Range(3, 7).contains(5));
        assert!(!LineSpec::Range(3, 7).contains(2));
        assert!(LineSpec::Multi(vec![(1, 3), (7, 9)]).contains(8));
        assert!(!LineSpec::Multi(vec![(1, 3), (7, 9)]).contains(5));
    }

    fn make_entry(author: &str, mail: &str, commit: &str, day: u32) -> BlameEntry {
        BlameEntry {
            file: "a.py".into(),
            line: 1,
            author: author.into(),
            author_mail: mail.into(),
            author_time: chrono::NaiveDate::from_ymd_opt(2024, 1, day)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            vcs: VcsKind::Git,
            commit_id: commit.into(),
            summary: "msg".into(),
            content: "".into(),
        }
    }

    #[test]
    fn test_aggregate_basic() {
        let entries = vec![
            make_entry("alice", "a@a.com", &"a".repeat(40), 1),
            make_entry("bob", "b@b.com", &"b".repeat(40), 15),
        ];
        let result = aggregate(entries, &HashMap::new());
        assert_eq!(result.summary.len(), 2);
        assert!(result.suggested_responsible.is_some());
        assert_eq!(result.uncommitted_lines.len(), 0);
    }

    #[test]
    fn test_filter_uncommitted() {
        let entries = vec![make_entry("Not Committed Yet", "x@x.com", &"0".repeat(40), 1)];
        let result = aggregate(entries, &HashMap::new());
        assert_eq!(result.entries.len(), 0);
        assert_eq!(result.uncommitted_lines.len(), 1);
        assert!(result.suggested_responsible.is_none());
    }

    #[test]
    fn test_alias_resolution() {
        let mut aliases = HashMap::new();
        aliases.insert(
            "alice".to_string(),
            vec!["al".to_string(), "alice@old.com".to_string()],
        );

        let entries = vec![
            make_entry("al", "al@new.com", &"a".repeat(40), 1),
            make_entry("alice", "alice@old.com", &"b".repeat(40), 2),
        ];
        let result = aggregate(entries, &aliases);
        assert_eq!(result.summary.len(), 1);
        assert_eq!(result.summary[0].author, "alice");
        assert_eq!(result.summary[0].commit_count, 2);
    }

    #[test]
    fn test_scoring_order() {
        let entries = vec![
            make_entry("old_author", "old@x.com", &"a".repeat(40), 1),
            make_entry("new_author", "new@x.com", &"b".repeat(40), 15),
        ];
        let result = aggregate(entries, &HashMap::new());
        assert_eq!(result.suggested_responsible.unwrap(), "new_author");
    }
}
