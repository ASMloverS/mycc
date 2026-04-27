use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::MagicNumberConfig;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static NUM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\d+)([uUlL]*)\b").unwrap()
});

static STRING_CHAR_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'"#).unwrap()
});

static BLOCK_COMMENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/\*.*?\*/").unwrap());

pub fn check_magic_number(
    source: &SourceFile,
    config: &MagicNumberConfig,
) -> Vec<Diagnostic> {
    if !config.enabled {
        return vec![];
    }
    let allowed: HashSet<i64> = config.allowed.iter().copied().collect();
    let mut diags = Vec::new();
    let lines = source.lines();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if should_skip_line(trimmed) {
            continue;
        }
        let prefix_len = line.len() - line.trim_start().len();
        let masked = mask_exclusions(trimmed);
        let code = match masked.find("//") {
            Some(pos) => &masked[..pos],
            None => masked.as_ref(),
        };
        for caps in NUM_RE.captures_iter(code) {
            let m = caps.get(1).unwrap();
            let num_str = m.as_str();
            let start = m.start();
            let end = m.end();
            let bytes = masked.as_bytes();
            if end < bytes.len() && bytes[end] == b'.' {
                continue;
            }
            if start > 0 && bytes[start - 1] == b'.' {
                continue;
            }
            if is_scientific_exponent(bytes, start) {
                continue;
            }
            let (val, col) = resolve_value(bytes, start, num_str);
            let val = match val {
                Some(v) => v,
                None => continue,
            };
            if !allowed.contains(&val) {
                diags.push(Diagnostic::new_with_source(
                    source.path.to_string_lossy().to_string(),
                    i + 1,
                    col + 1 + prefix_len,
                    Severity::Warning,
                    "readability-magic-numbers",
                    &format!("Magic number: {}", val),
                    line,
                ));
            }
        }
    }
    diags
}

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

fn should_skip_line(line: &str) -> bool {
    line.starts_with('#')
        || line.starts_with("//")
        || line.starts_with("/*")
        || (line.starts_with('*') && !line.starts_with("*/"))
}

fn is_scientific_exponent(bytes: &[u8], start: usize) -> bool {
    if start == 0 {
        return false;
    }
    let mut check = start - 1;
    if bytes[check] == b'+' || bytes[check] == b'-' {
        if check == 0 {
            return false;
        }
        check -= 1;
    }
    bytes[check] == b'e' || bytes[check] == b'E'
}

fn resolve_value(
    bytes: &[u8],
    start: usize,
    num_str: &str,
) -> (Option<i64>, usize) {
    let positive = match num_str.parse::<i64>() {
        Ok(v) => v,
        Err(_) => return (None, 0),
    };
    if start > 0
        && bytes[start - 1] == b'-'
        && (start < 2
            || !(bytes[start - 2].is_ascii_alphanumeric()
                || bytes[start - 2] == b'_'))
    {
        (Some(-positive), start - 1)
    } else {
        (Some(positive), start)
    }
}
