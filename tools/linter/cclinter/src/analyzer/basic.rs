use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::AnalysisConfig;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static FN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\s*(?:static\s+|extern\s+|inline\s+)*(int|char|float|double|long|short|unsigned|signed|\w+_t)\s+\*?\s*(\w+)\s*\([^)]*\)\s*\{"
    ).unwrap()
});

static IMPLICIT_CONV_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(float|double)\s+(\w+)\s*=\s*(-?\d+)\s*;").unwrap()
});

static UNINIT_DECL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(?:int|char|float|double|long|short|unsigned|signed|\w+_t)\s+\*?\s*(\w+)\s*;"
    ).unwrap()
});

static ASSIGN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\w+)\s*=[^=]").unwrap());

fn strip_line_comment(line: &str) -> &str {
    match line.find("//") {
        Some(pos) => &line[..pos],
        None => line,
    }
}

fn line_has_return(line: &str) -> bool {
    line.contains("return ") || line.contains("return;")
}

pub fn check(source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_missing_return(source));
    diags.extend(check_implicit_conversion(source));
    diags.extend(check_uninit_hints(source));
    diags
}

fn check_missing_return(source: &SourceFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let lines = source.lines();
    let mut in_fn = false;
    let mut fn_line = 0;
    let mut brace_depth = 0i32;
    let mut has_return = false;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let code = strip_line_comment(trimmed);
        if !in_fn {
            if let Some(caps) = FN_RE.captures(code) {
                let return_type = caps[1].to_string();
                if return_type != "void" {
                    in_fn = true;
                    fn_line = i + 1;
                    has_return = line_has_return(code);
                    brace_depth = code.matches('{').count() as i32
                        - code.matches('}').count() as i32;
                    if brace_depth <= 0 {
                        if !has_return {
                            diags.push(Diagnostic::new(
                                source.path.to_string_lossy().to_string(),
                                fn_line,
                                1,
                                Severity::Warning,
                                "bugprone-missing-return",
                                "Non-void function may be missing return statement",
                            ));
                        }
                        in_fn = false;
                    }
                }
            }
        } else {
            brace_depth += code.matches('{').count() as i32;
            brace_depth -= code.matches('}').count() as i32;
            if line_has_return(code) {
                has_return = true;
            }
            if brace_depth <= 0 {
                if !has_return {
                    diags.push(Diagnostic::new(
                        source.path.to_string_lossy().to_string(),
                        fn_line,
                        1,
                        Severity::Warning,
                        "bugprone-missing-return",
                        "Non-void function may be missing return statement",
                    ));
                }
                in_fn = false;
            }
        }
    }
    diags
}

fn check_implicit_conversion(source: &SourceFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let lines = source.lines();
    for (i, line) in lines.iter().enumerate() {
        let code = strip_line_comment(line);
        if let Some(caps) = IMPLICIT_CONV_RE.captures(code) {
            let val: i64 = caps[3].parse().unwrap_or(0);
            if val != 0 {
                diags.push(Diagnostic::new_with_source(
                    source.path.to_string_lossy().to_string(),
                    i + 1,
                    1,
                    Severity::Warning,
                    "bugprone-implicit-conversion",
                    &format!("Implicit integer to {} conversion", &caps[1]),
                    line,
                ));
            }
        }
    }
    diags
}

fn check_uninit_hints(source: &SourceFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let lines = source.lines();
    let mut uninit_decls: Vec<(usize, String)> = Vec::new();
    let mut assigned: HashSet<String> = HashSet::new();
    for (i, line) in lines.iter().enumerate() {
        let code = strip_line_comment(line);
        for caps in UNINIT_DECL_RE.captures_iter(code) {
            uninit_decls.push((i, caps[1].to_string()));
        }
        for caps in ASSIGN_RE.captures_iter(code) {
            assigned.insert(caps[1].to_string());
        }
    }
    for (line_idx, name) in &uninit_decls {
        if !assigned.contains(name) {
            diags.push(Diagnostic::new_with_source(
                source.path.to_string_lossy().to_string(),
                line_idx + 1,
                1,
                Severity::Warning,
                "bugprone-uninit",
                &format!("Variable '{}' may be used uninitialized", name),
                lines[*line_idx],
            ));
        }
    }
    diags
}
