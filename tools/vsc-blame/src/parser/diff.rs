use std::sync::LazyLock;

use crate::util::AppError;
use crate::vcs::{FileDiff, Hunk};

static HUNK_HEADER_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"-\d+(?:,\d+)?\s+\+(\d+)(?:,(\d+))?").unwrap()
});

pub fn parse_unified_diff(input: &str) -> Result<Vec<FileDiff>, AppError> {
    let mut diffs: Vec<FileDiff> = Vec::new();
    let mut _current_file: Option<String> = None;
    let mut current_hunk: Option<HunkBuilder> = None;

    for line in input.lines() {
        if let Some(filename) = line.strip_prefix("+++ b/") {
            if let Some(h) = current_hunk.take() {
                if let Some(ref mut diff) = diffs.last_mut() {
                    diff.hunks.push(h.build());
                }
            }
            _current_file = Some(filename.to_string());
            diffs.push(FileDiff {
                file: filename.to_string(),
                hunks: Vec::new(),
            });
            continue;
        }

        if let Some(header) = line.strip_prefix("@@ ") {
            if let Some(h) = current_hunk.take() {
                if let Some(ref mut diff) = diffs.last_mut() {
                    diff.hunks.push(h.build());
                }
            }
            if let Some(hunk) = parse_hunk_header(header) {
                current_hunk = Some(HunkBuilder::new(hunk));
            }
            continue;
        }

        if let Some(ref mut h) = current_hunk {
            if let Some(content) = line.strip_prefix('+') {
                if !content.starts_with("+++") {
                    h.added_lines.push(h.new_line);
                }
                h.new_line += 1;
            } else if line.starts_with('-') {
                // removed line, don't advance new_line
            } else {
                // context line or no-newline-at-end
                h.new_line += 1;
            }
        }
    }

    if let Some(h) = current_hunk.take() {
        if let Some(ref mut diff) = diffs.last_mut() {
            diff.hunks.push(h.build());
        }
    }

    Ok(diffs)
}

struct ParsedHunkHeader {
    new_start: usize,
    new_count: usize,
}

fn parse_hunk_header(header: &str) -> Option<ParsedHunkHeader> {
    let caps = HUNK_HEADER_RE.captures(header)?;
    let new_start: usize = caps[1].parse().ok()?;
    let new_count: usize = caps
        .get(2)
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(1);
    Some(ParsedHunkHeader {
        new_start,
        new_count,
    })
}

struct HunkBuilder {
    new_start: usize,
    new_count: usize,
    new_line: usize,
    added_lines: Vec<usize>,
}

impl HunkBuilder {
    fn new(h: ParsedHunkHeader) -> Self {
        Self {
            new_start: h.new_start,
            new_count: h.new_count,
            new_line: h.new_start,
            added_lines: Vec::new(),
        }
    }

    fn build(self) -> Hunk {
        Hunk {
            old_start: 0,
            old_count: 0,
            new_start: self.new_start,
            new_count: self.new_count,
            added_lines: self.added_lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_DIFF: &str = "\
diff --git a/src/main.py b/src/main.py
index abc1234..def5678 100644
--- a/src/main.py
+++ b/src/main.py
@@ -10,7 +10,9 @@ def foo():
     old line 1
     old line 2
-    removed line
+    added line 1
+    added line 2
+    added line 3
     context line
@@ -25,3 +27,4 @@ def bar():
     context
+    new line
     end";

    #[test]
    fn test_parse_unified_diff() {
        let diffs = parse_unified_diff(SAMPLE_DIFF).unwrap();
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].file, "src/main.py");
        assert_eq!(diffs[0].hunks.len(), 2);
    }

    #[test]
    fn test_parse_hunk_added_lines() {
        let diffs = parse_unified_diff(SAMPLE_DIFF).unwrap();
        let hunk1 = &diffs[0].hunks[0];
        assert_eq!(hunk1.new_start, 10);
        assert_eq!(hunk1.added_lines.len(), 3);
        assert!(hunk1.added_lines.contains(&12));
        assert!(hunk1.added_lines.contains(&13));
        assert!(hunk1.added_lines.contains(&14));

        let hunk2 = &diffs[0].hunks[1];
        assert_eq!(hunk2.added_lines.len(), 1);
        assert_eq!(hunk2.added_lines[0], 28);
    }

    #[test]
    fn test_parse_empty_diff() {
        let diffs = parse_unified_diff("").unwrap();
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_parse_multiple_files() {
        let diff = "\
--- a/foo.py
+++ b/foo.py
@@ -1,3 +1,4 @@
 line1
+added
 line2
 line3
--- a/bar.py
+++ b/bar.py
@@ -5,3 +5,4 @@
 ctx
+new
 end";
        let diffs = parse_unified_diff(diff).unwrap();
        assert_eq!(diffs.len(), 2);
        assert_eq!(diffs[0].file, "foo.py");
        assert_eq!(diffs[1].file, "bar.py");
    }
}
