use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::{mask_code_line, SourceFile};
use regex::Regex;
use std::collections::HashSet;

const DEFAULT_PROHIBITED: &[&str] = &[
    "strcpy",
    "strcat",
    "sprintf",
    "vsprintf",
    "gets",
    "scanf",
];

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
        let masked = mask_code_line(line);
        for (fn_name, re) in &patterns {
            if re.is_match(&masked) {
                diags.push(Diagnostic::new_with_source(
                    source.display_path(),
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
