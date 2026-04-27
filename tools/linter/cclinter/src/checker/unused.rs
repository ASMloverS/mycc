use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::UnusedConfig;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

static DECL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
        \b(?:(?:const|static|volatile|register|extern|signed|unsigned)\s+)*
        (?:int|char|float|double|short|long|\w+_t)
        (?:\s+(?:int|char|long|short))*
        [\s*]+
        (\w+)
        \s*[=;]
        ",
    )
    .unwrap()
});

static USE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\w+)\b").unwrap());

static DEFINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"#define\s+(\w+)").unwrap());

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

fn strip_line_comment(code: &str) -> &str {
    match code.find("//") {
        Some(pos) => &code[..pos],
        None => code,
    }
}

// KNOWN LIMITATION: This checker has no scope awareness. Variables declared
// in inner scopes (e.g., inside if/for blocks) may produce false positives
// or false negatives if the same name appears in multiple scopes.

pub fn check_unused(source: &SourceFile, config: &UnusedConfig) -> Vec<Diagnostic> {
    if !config.enabled {
        return vec![];
    }
    let mut diags = Vec::new();
    diags.extend(check_unused_vars(source));
    diags.extend(check_unused_macros(source));
    diags
}

fn check_unused_vars(source: &SourceFile) -> Vec<Diagnostic> {
    let mut declared: HashMap<String, usize> = HashMap::new();
    let mut id_counts: HashMap<String, usize> = HashMap::new();
    let lines = source.lines();

    for (i, line) in lines.iter().enumerate() {
        let masked = mask_exclusions(line);
        let code = strip_line_comment(&masked);
        for caps in DECL_RE.captures_iter(code) {
            let name = caps[1].to_string();
            declared.entry(name).or_insert(i);
        }
        for caps in USE_RE.captures_iter(code) {
            *id_counts.entry(caps[1].to_string()).or_insert(0) += 1;
        }
    }

    let mut diags = Vec::new();
    for (name, &line_idx) in &declared {
        let count = id_counts.get(name).copied().unwrap_or(0);
        if count <= 1 {
            diags.push(Diagnostic::new_with_source(
                source.path.to_string_lossy().to_string(),
                line_idx + 1,
                1,
                Severity::Warning,
                "bugprone-unused-variable",
                &format!("Variable '{}' is unused", name),
                lines[line_idx],
            ));
        }
    }
    diags
}

fn check_unused_macros(source: &SourceFile) -> Vec<Diagnostic> {
    let mut defined: HashMap<String, usize> = HashMap::new();
    let lines = source.lines();

    for (i, line) in lines.iter().enumerate() {
        if let Some(caps) = DEFINE_RE.captures(line) {
            let name = caps[1].to_string();
            defined.entry(name).or_insert(i);
        }
    }

    let mut id_counts: HashMap<String, usize> = HashMap::new();
    for (i, line) in lines.iter().enumerate() {
        if defined.values().any(|&idx| idx == i) {
            continue;
        }
        let masked = mask_exclusions(line);
        let code = strip_line_comment(&masked);
        for caps in USE_RE.captures_iter(code) {
            *id_counts.entry(caps[1].to_string()).or_insert(0) += 1;
        }
    }

    let mut diags = Vec::new();
    for (name, &line_idx) in &defined {
        let count = id_counts.get(name).copied().unwrap_or(0);
        if count == 0 {
            diags.push(Diagnostic::new_with_source(
                source.path.to_string_lossy().to_string(),
                line_idx + 1,
                1,
                Severity::Warning,
                "bugprone-unused-macro",
                &format!("Macro '{}' is unused", name),
                lines[line_idx],
            ));
        }
    }
    diags
}
