use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

const DEFAULT_PROHIBITED: &[&str] = &[
    "strcpy",
    "strcat",
    "sprintf",
    "vsprintf",
    "gets",
    "scanf",
];

static STRING_CHAR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'"#).unwrap()
});

static BLOCK_COMMENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/\*.*?\*/").unwrap());

fn mask_exclusions(line: &str) -> String {
    let s = STRING_CHAR_RE
        .replace_all(line, |caps: &regex::Captures| {
            " ".repeat(caps[0].len())
        });
    BLOCK_COMMENT_RE
        .replace_all(&s, |caps: &regex::Captures| {
            " ".repeat(caps[0].len())
        })
        .into_owned()
}

pub fn check_prohibited(
    source: &SourceFile,
    use_default: bool,
    extra: &[String],
    remove: &[String],
) -> Vec<Diagnostic> {
    let mut fns: HashSet<String> = HashSet::new();
    if use_default {
        fns.extend(DEFAULT_PROHIBITED.iter().map(|s| s.to_string()));
    }
    for e in extra {
        fns.insert(e.clone());
    }
    for r in remove {
        fns.remove(r);
    }

    let patterns: Vec<(&String, Regex)> = fns
        .iter()
        .map(|name| {
            let pattern = format!(r"\b{}\s*\(", regex::escape(name));
            (
                name,
                Regex::new(&pattern)
                    .expect("regex from escaped name should always be valid"),
            )
        })
        .collect();

    let mut diags = Vec::new();
    let lines = source.lines();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        let masked = mask_exclusions(line);
        for (fn_name, re) in &patterns {
            if re.is_match(&masked) {
                diags.push(Diagnostic::new_with_source(
                    source.path.to_string_lossy().to_string(),
                    i + 1,
                    1,
                    Severity::Error,
                    "bugprone-prohibited-function",
                    &format!("Use of prohibited function: {}", fn_name),
                    line,
                ));
            }
        }
    }
    diags
}
