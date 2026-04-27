use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;

/// Parse MD file → (frontmatter map, body). Mirrors Python parse_md().
pub fn parse_md(md_path: &Path) -> Result<(HashMap<String, String>, String)> {
    let raw = std::fs::read_to_string(md_path)?;
    // Normalize CRLF to match Python's universal newline read_text()
    let text = raw.replace("\r\n", "\n");
    Ok(extract_fm(&text))
}

/// Split text on ^---\s*\n ... \n---\s*\n  (mirrors Python _FM_RE).
fn extract_fm(text: &str) -> (HashMap<String, String>, String) {
    let rest = match text.strip_prefix("---") {
        None => return (HashMap::new(), text.to_string()),
        Some(s) => s,
    };
    let rest = match rest.trim_start_matches(' ').strip_prefix('\n') {
        None => return (HashMap::new(), text.to_string()),
        Some(s) => s,
    };

    let mut i = 0;
    while i < rest.len() {
        let Some(rel) = rest[i..].find("\n---") else { break };
        let j = i + rel;
        let after = &rest[j + 4..]; // bytes after "\n---"
        let sp = after.len() - after.trim_start_matches(' ').len();
        let tail = &after[sp..];
        if tail.starts_with('\n') || tail.is_empty() {
            let fm_raw = &rest[..j];
            let body_skip = 4 + sp + usize::from(tail.starts_with('\n'));
            let body = rest.get(j + body_skip..).unwrap_or("").to_string();
            return (parse_fm_lines(fm_raw), body);
        }
        i = j + 1;
    }
    (HashMap::new(), text.to_string())
}

fn parse_fm_lines(raw: &str) -> HashMap<String, String> {
    let mut fm = HashMap::new();
    for line in raw.lines() {
        if let Some(col) = line.find(':') {
            let key = line[..col].trim();
            let val = line[col + 1..].trim().trim_matches('"');
            if !key.is_empty() {
                fm.insert(key.to_string(), val.to_string());
            }
        }
    }
    fm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_frontmatter() {
        let t = "---\nname: test\nmodel: opus\n---\nBody here\n";
        let (fm, body) = extract_fm(t);
        assert_eq!(fm["name"], "test");
        assert_eq!(fm["model"], "opus");
        assert!(body.contains("Body here"));
    }

    #[test]
    fn no_frontmatter() {
        let t = "Just body\n";
        let (fm, body) = extract_fm(t);
        assert!(fm.is_empty());
        assert_eq!(body, "Just body\n");
    }

    #[test]
    fn quoted_values() {
        let t = "---\ndesc: \"my desc\"\n---\nbody\n";
        let (fm, _) = extract_fm(t);
        assert_eq!(fm["desc"], "my desc");
    }

    #[test]
    fn crlf_normalized() {
        let t = "---\r\nkey: val\r\n---\r\nbody\r\n";
        let normalized = t.replace("\r\n", "\n");
        let (fm, body) = extract_fm(&normalized);
        assert_eq!(fm["key"], "val");
        assert!(body.contains("body"));
    }
}
