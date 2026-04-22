use std::sync::LazyLock;

use crate::common::source::SourceFile;
use crate::config::FormatConfig;
use regex::Regex;

pub fn fix_blank_lines(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if source.content.is_empty() {
        return Ok(());
    }
    let had_trailing_newline = source.content.ends_with('\n');
    let mut lines: Vec<String> = source.content.lines().map(|l| l.to_string()).collect();
    lines = trim_leading_blanks(&lines);
    lines = normalize_whitespace_lines(&lines);
    lines = collapse_blank_lines(&lines, config.max_consecutive_blank_lines);
    lines = ensure_blank_after_includes(&lines, config.blank_lines_after_include);
    lines = ensure_blank_before_functions(&lines, config.blank_lines_before_function);
    lines = trim_trailing_blanks(&lines);
    let mut result = lines.join("\n");
    if had_trailing_newline && !result.is_empty() {
        result.push('\n');
    }
    source.content = result;
    Ok(())
}

fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

fn trim_leading_blanks(lines: &[String]) -> Vec<String> {
    let start = lines.iter().position(|l| !is_blank(l)).unwrap_or(lines.len());
    lines[start..].to_vec()
}

fn normalize_whitespace_lines(lines: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    let mut in_whitespace_run = false;
    for line in lines {
        let blank = is_blank(line);
        let is_whitespace_only = blank && !line.is_empty();
        if is_whitespace_only {
            if !in_whitespace_run {
                result.push(String::new());
            }
            in_whitespace_run = true;
        } else {
            result.push(line.clone());
            in_whitespace_run = false;
        }
    }
    result
}

fn collapse_blank_lines(lines: &[String], max: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut consecutive = 0usize;
    for line in lines {
        if is_blank(line) {
            consecutive += 1;
            if consecutive <= max {
                result.push(line.clone());
            }
        } else {
            consecutive = 0;
            result.push(line.clone());
        }
    }
    result
}

fn is_include_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('#') && trimmed.contains("include")
}

fn ensure_blank_after_includes(lines: &[String], count: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut in_include_block = false;
    let mut blanks_since_include = 0usize;
    for line in lines {
        if is_include_line(line) {
            in_include_block = true;
            blanks_since_include = 0;
            result.push(line.clone());
            continue;
        }
        if in_include_block {
            if is_blank(line) {
                blanks_since_include += 1;
                if blanks_since_include <= count {
                    result.push(line.clone());
                }
            } else {
                for _ in blanks_since_include..count {
                    result.push(String::new());
                }
                result.push(line.clone());
                in_include_block = false;
                blanks_since_include = 0;
            }
        } else {
            result.push(line.clone());
        }
    }
    result
}

static FUNC_DEF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(static\s+)?(inline\s+)?\w+\s+\*?\w+\s*\(").unwrap()
});

fn is_function_def(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.starts_with('#') || trimmed.starts_with("//") || trimmed.starts_with("/*") {
        return false;
    }
    FUNC_DEF_RE.is_match(line)
}

fn count_preceding_blanks(lines: &[String], idx: usize) -> usize {
    let mut count = 0;
    let mut i = idx;
    while i > 0 {
        i -= 1;
        if is_blank(&lines[i]) {
            count += 1;
        } else {
            break;
        }
    }
    count
}

fn ensure_blank_before_functions(lines: &[String], count: usize) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 && is_function_def(line) && !is_function_def(&lines[i - 1]) {
            let preceding = count_preceding_blanks(lines, i);
            if preceding != count {
                let first_content = result.len() - preceding;
                result.truncate(first_content);
                for _ in 0..count {
                    result.push(String::new());
                }
            }
        }
        result.push(line.clone());
    }
    result
}

fn trim_trailing_blanks(lines: &[String]) -> Vec<String> {
    let end = lines.iter().rposition(|l| !is_blank(l)).map_or(0, |p| p + 1);
    lines[..end].to_vec()
}
