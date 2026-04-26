use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::{IncludeGuardConfig, IncludeGuardStyle};
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static INCLUDE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"#\s*include\s+[<"]([^>"]+)[>"]"#).unwrap());
static IFNDEF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"#ifndef\s+(\w+)"#).unwrap());
static PRAGMA_ONCE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"#\s*pragma\s+once\b").unwrap());

pub fn check_include_guard(source: &SourceFile, config: &IncludeGuardConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let lines = source.lines();
    let is_header = source
        .path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e, "h" | "hpp" | "hh" | "hxx"))
        .unwrap_or(false);

    let mut seen: HashSet<String> = HashSet::new();
    for (line_num, line) in lines.iter().enumerate() {
        if let Some(caps) = INCLUDE_RE.captures(line) {
            let header = caps[1].to_string();
            if seen.contains(&header) {
                diags.push(Diagnostic::new_with_source(
                    source.path.to_string_lossy().to_string(),
                    line_num + 1,
                    1,
                    Severity::Warning,
                    "bugprone-duplicate-include",
                    &format!("Duplicate include: {}", header),
                    line,
                ));
            }
            seen.insert(header);
        }
    }

    if is_header && !lines.is_empty() {
        let has_pragma = lines.iter().any(|l| PRAGMA_ONCE_RE.is_match(l.trim()));
        let has_ifndef = lines.iter().take(10).any(|l| IFNDEF_RE.is_match(l));
        let has_guard = match config.style {
            IncludeGuardStyle::PragmaOnce => has_pragma,
            IncludeGuardStyle::Ifndef => has_ifndef,
        };
        if !has_guard {
            diags.push(Diagnostic::new(
                source.path.to_string_lossy().to_string(),
                1,
                1,
                Severity::Warning,
                "bugprone-missing-include-guard",
                "Header file is missing an include guard",
            ));
        }
    }

    diags
}
