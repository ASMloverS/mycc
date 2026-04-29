use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::{strip_line_comment, SourceFile};
use crate::config::AnalysisConfig;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static ALLOC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(malloc|calloc|realloc)\s*\(").unwrap());
static FREE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bfree\s*\(").unwrap());
static DEAD_BRANCH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|\belse\s+)if\s*\(\s*(0|false)\s*\)").unwrap()
});
static HASH_IF_ZERO_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*#\s*if\s+0\b").unwrap());
static SUSPICIOUS_CAST_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\(\s*int\s*\)\s*(\w+)").unwrap());
static PTR_DECL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:int|char|float|double|long|short|unsigned|signed|void|\w+_t)\s*\*\s*(\w+)").unwrap()
});
static ALLOC_ASSIGN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\w+)\s*=\s*(?:\([^)]*\)\s*)?(?:malloc|calloc|realloc)\s*\(").unwrap()
});
static RETURN_VAR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\breturn\s+(\w+)\s*;").unwrap());

pub fn check(source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    let lines = source.lines();
    let mut diags = Vec::new();
    diags.extend(check_resource_leaks(&lines, source));
    diags.extend(check_dead_branches(&lines, source));
    diags.extend(check_suspicious_casts(&lines, source));
    diags
}

fn check_resource_leaks(lines: &[&str], source: &SourceFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let mut brace_depth: i32 = 0;
    let mut fn_allocs: Vec<(usize, &str)> = Vec::new();
    let mut fn_frees: usize = 0;
    let mut fn_alloc_vars: HashSet<String> = HashSet::new();
    let mut fn_returns_alloc = false;
    let mut in_fn = false;
    let path_str = source.path.to_string_lossy().to_string();
    for (i, line) in lines.iter().enumerate() {
        let code = strip_line_comment(line);
        let opens = code.matches('{').count() as i32;
        let closes = code.matches('}').count() as i32;
        let prev_depth = brace_depth;
        brace_depth += opens - closes;
        if !in_fn && prev_depth == 0 && opens > 0 {
            in_fn = true;
            fn_allocs.clear();
            fn_frees = 0;
            fn_alloc_vars.clear();
            fn_returns_alloc = false;
        }
        if in_fn {
            if ALLOC_RE.is_match(code) {
                fn_allocs.push((i + 1, *line));
                if let Some(caps) = ALLOC_ASSIGN_RE.captures(code) {
                    fn_alloc_vars.insert(caps[1].to_string());
                }
            }
            if FREE_RE.is_match(code) {
                fn_frees += 1;
            }
            if let Some(caps) = RETURN_VAR_RE.captures(code) {
                if fn_alloc_vars.contains(&caps[1]) {
                    fn_returns_alloc = true;
                }
            }
        }
        if in_fn && brace_depth <= 0 {
            if fn_allocs.len() > fn_frees && !fn_returns_alloc {
                for (ln, lt) in &fn_allocs {
                    diags.push(Diagnostic::new_with_source(
                        path_str.clone(),
                        *ln,
                        1,
                        Severity::Warning,
                        "bugprone-resource-leak",
                        "Allocated memory may not be freed",
                        lt,
                    ));
                }
            }
            in_fn = false;
        }
    }
    diags
}

fn check_dead_branches(lines: &[&str], source: &SourceFile) -> Vec<Diagnostic> {
    let path_str = source.path.to_string_lossy().to_string();
    let mut diags = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let code = strip_line_comment(line);
        if DEAD_BRANCH_RE.is_match(code) {
            diags.push(Diagnostic::new_with_source(
                path_str.clone(),
                i + 1,
                1,
                Severity::Warning,
                "bugprone-dead-branch",
                "Condition is always false",
                line,
            ));
        } else if HASH_IF_ZERO_RE.is_match(code) {
            diags.push(Diagnostic::new_with_source(
                path_str.clone(),
                i + 1,
                1,
                Severity::Warning,
                "bugprone-dead-branch",
                "Preprocessor dead code block",
                line,
            ));
        }
    }
    diags
}

fn check_suspicious_casts(lines: &[&str], source: &SourceFile) -> Vec<Diagnostic> {
    let path_str = source.path.to_string_lossy().to_string();
    let mut diags = Vec::new();
    let mut ptr_vars: HashSet<String> = HashSet::new();
    for line in lines.iter() {
        for caps in PTR_DECL_RE.captures_iter(strip_line_comment(line)) {
            ptr_vars.insert(caps[1].to_string());
        }
    }
    for (i, line) in lines.iter().enumerate() {
        let code = strip_line_comment(line);
        if let Some(caps) = SUSPICIOUS_CAST_RE.captures(code) {
            if ptr_vars.contains(&caps[1]) {
                diags.push(Diagnostic::new_with_source(
                    path_str.clone(),
                    i + 1,
                    1,
                    Severity::Warning,
                    "bugprone-suspicious-cast",
                    "Casting pointer to int may lose data",
                    line,
                ));
            }
        }
    }
    diags
}
