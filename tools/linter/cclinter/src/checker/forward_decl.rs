use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::{mask_code_line, SourceFile};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

static FORWARD_DECL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\w+[\s*]+(\w+)\s*\([^)]*\)\s*;").unwrap()
});

static FUNC_DEF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\w+[\s*]+(\w+)\s*\([^)]*\)\s*\{").unwrap()
});

static FUNC_SIG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\w+[\s*]+(\w+)\s*\([^)]*\)\s*$").unwrap()
});

static CALL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\w+)\s*\(").unwrap());

static C_KEYWORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "if", "else", "for", "while", "do", "switch", "case", "default",
        "break", "continue", "return", "goto", "sizeof", "typeof",
        "alignof", "offsetof", "struct", "union", "enum",
    ])
});

fn is_keyword(name: &str) -> bool {
    C_KEYWORDS.contains(name)
}

pub fn check_forward_decl(source: &SourceFile) -> Vec<Diagnostic> {
    let lines = source.lines();
    let masked_lines: Vec<String> = lines.iter().map(|l| mask_code_line(l)).collect();
    let mut forward_decls: HashMap<String, usize> = HashMap::new();
    let mut func_defs: HashMap<String, usize> = HashMap::new();
    let mut pending_sig: Option<(String, usize)> = None;

    for (i, masked) in masked_lines.iter().enumerate() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        if let Some((name, sig_i)) = pending_sig.take() {
            if trimmed.starts_with('{') {
                func_defs.entry(name).or_insert(sig_i);
            } else if trimmed.is_empty() {
                pending_sig = Some((name, sig_i));
            }
        }

        if let Some(caps) = FORWARD_DECL_RE.captures(masked) {
            let name = caps[1].to_string();
            if !is_keyword(&name) {
                forward_decls.entry(name).or_insert(i);
            }
        }
        if let Some(caps) = FUNC_DEF_RE.captures(masked) {
            let name = caps[1].to_string();
            if !is_keyword(&name) {
                func_defs.entry(name).or_insert(i);
            }
        } else if let Some(caps) = FUNC_SIG_RE.captures(masked) {
            let name = caps[1].to_string();
            if !is_keyword(&name) {
                pending_sig = Some((name, i));
            }
        }
    }

    let mut diags = Vec::new();
    let mut reported: HashSet<String> = HashSet::new();

    for (i, masked) in masked_lines.iter().enumerate() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        for caps in CALL_RE.captures_iter(masked) {
            let name = caps[1].to_string();
            if is_keyword(&name) {
                continue;
            }
            if reported.contains(&name) {
                continue;
            }
            let fwd_line = forward_decls.get(&name);
            let def_line = func_defs.get(&name);
            match (fwd_line, def_line) {
                (Some(_), _) => {}
                (None, None) => {}
                (None, Some(&def_i)) if def_i > i => {
                    reported.insert(name.clone());
                    diags.push(Diagnostic::new_with_source(
                        source.display_path(),
                        i + 1,
                        1,
                        Severity::Warning,
                        "bugprone-missing-forward-declaration",
                        &format!(
                            "Function '{}' called before declaration",
                            name
                        ),
                        lines[i],
                    ));
                }
                _ => {}
            }
        }
    }
    diags
}
