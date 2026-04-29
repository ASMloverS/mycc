use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::{mask_string_literals, strip_line_comment, SourceFile};
use crate::config::AnalysisConfig;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static GETS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bgets\s*\(").unwrap());

static SCANF_UNBOUNDED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\bscanf\s*\(\s*"%s"\s*,\s*\w+"#).unwrap()
});

static ARR_DECL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(?:int|char|float|double|long|short|unsigned|signed|void|\w+_t)\s+(\w+)\s*\[\s*\d+\s*\]",
    )
    .unwrap()
});

static ARR_ACCESS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\w+)\s*\[\s*(\w+)\s*\]").unwrap());

static NULL_PTR_INIT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(?:int|char|float|double|long|short|unsigned|signed|void|\w+_t)\s*\*\s*(\w+)\s*=\s*NULL\s*;",
    )
    .unwrap()
});

static DEREF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\s*(\w+)").unwrap());

const NULL_CHECK_LOOKBACK: usize = 5;

fn is_deref_context(text: &str, match_start: usize) -> bool {
    if match_start == 0 {
        return true;
    }
    let before = text[..match_start].trim_end();
    if before.is_empty() {
        return true;
    }
    let last = before.chars().last().unwrap();
    if last.is_alphanumeric() || last == '_' {
        let word_start = before
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let word = &before[word_start..];
        matches!(word, "return" | "sizeof" | "case" | "default")
    } else {
        !matches!(last, ')' | ']')
    }
}

pub fn check(source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    let lines = source.lines();
    let mut diags = Vec::new();
    diags.extend(check_buffer_overflow_patterns(&lines, source));
    diags.extend(check_null_deref_patterns(&lines, source));
    diags
}

fn is_variable_index(s: &str) -> bool {
    !s.chars().all(|c| c.is_ascii_digit())
}

fn check_buffer_overflow_patterns(lines: &[&str], source: &SourceFile) -> Vec<Diagnostic> {
    let path_str = source.path.to_string_lossy().to_string();
    let mut diags = Vec::new();
    let mut arrays: HashSet<String> = HashSet::new();
    let mut seen: HashSet<(usize, String)> = HashSet::new();

    for line in lines.iter() {
        let code = strip_line_comment(line);
        if let Some(caps) = ARR_DECL_RE.captures(code) {
            arrays.insert(caps[1].to_string());
        }
    }

    for (i, line) in lines.iter().enumerate() {
        let code = strip_line_comment(line);

        if GETS_RE.is_match(code) {
            diags.push(Diagnostic::new_with_source(
                path_str.clone(),
                i + 1,
                1,
                Severity::Error,
                "bugprone-buffer-overflow-risk",
                "gets() has no bounds checking — use fgets() instead",
                line,
            ));
        }

        if SCANF_UNBOUNDED_RE.is_match(code) {
            diags.push(Diagnostic::new_with_source(
                path_str.clone(),
                i + 1,
                1,
                Severity::Warning,
                "bugprone-buffer-overflow-risk",
                "scanf(\"%s\", ...) has no bounds checking",
                line,
            ));
        }

        for caps in ARR_ACCESS_RE.captures_iter(code) {
            let arr_name = &caps[1];
            let idx = &caps[2];
            if arrays.contains(arr_name) && is_variable_index(idx) {
                let key = (i + 1, arr_name.to_string());
                if seen.contains(&key) {
                    continue;
                }
                let bounds_pattern = format!(
                    r"\b{}\s*(?:<=?|>=?|==|!=)\s*\w+|\w+\s*(?:<=?|>=?|==|!=)\s*{}\b",
                    idx, idx
                );
                let has_bounds = code.contains("sizeof")
                    || Regex::new(&bounds_pattern).unwrap().is_match(&code);
                if !has_bounds {
                    seen.insert(key);
                    diags.push(Diagnostic::new_with_source(
                        path_str.clone(),
                        i + 1,
                        1,
                        Severity::Warning,
                        "bugprone-buffer-overflow-risk",
                        &format!("Array '{}' access without bounds check", arr_name),
                        line,
                    ));
                }
            }
        }
    }
    diags
}

fn check_null_deref_patterns(lines: &[&str], source: &SourceFile) -> Vec<Diagnostic> {
    let path_str = source.path.to_string_lossy().to_string();
    let mut diags = Vec::new();
    let mut null_ptrs: HashSet<String> = HashSet::new();
    let mut brace_depth: i32 = 0;

    for (i, line) in lines.iter().enumerate() {
        let code = strip_line_comment(line);
        let masked = mask_string_literals(&code);
        let opens = code.matches('{').count() as i32;
        let closes = code.matches('}').count() as i32;
        let prev_depth = brace_depth;
        brace_depth += opens - closes;

        if prev_depth == 0 && brace_depth > 0 {
            null_ptrs.clear();
        }

        if NULL_PTR_INIT_RE.is_match(code) {
            for caps in NULL_PTR_INIT_RE.captures_iter(code) {
                null_ptrs.insert(caps[1].to_string());
            }
            continue;
        }

        for caps in DEREF_RE.captures_iter(&masked) {
            let ptr_name = &caps[1];
            let mat = caps.get(0).unwrap();
            if !is_deref_context(&masked, mat.start()) {
                continue;
            }
            if null_ptrs.contains(ptr_name) {
                let has_null_check = lines[..i].iter().rev().take(NULL_CHECK_LOOKBACK).any(|l| {
                    let cl = strip_line_comment(l);
                    cl.contains("if") && cl.contains(ptr_name)
                });
                if !has_null_check {
                    diags.push(Diagnostic::new_with_source(
                        path_str.clone(),
                        i + 1,
                        1,
                        Severity::Warning,
                        "bugprone-null-deref-risk",
                        &format!("Potential null pointer dereference: *{}", ptr_name),
                        line,
                    ));
                }
            }
        }
    }
    diags
}
