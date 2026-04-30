use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::ComplexityConfig;
use regex::Regex;
use std::sync::LazyLock;

static FN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\s*(?:(?:static|extern|const|inline)\s+)*(?:void|int|char|float|double|long|short|unsigned|signed|struct\s+\w+|enum\s+\w+|\w+_t)\s*\*?\s*\w+\s*\("
    ).unwrap()
});

pub fn check_complexity(source: &SourceFile, config: &ComplexityConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let lines = source.lines();
    if lines.len() > config.max_file_lines {
        diags.push(Diagnostic::new_with_source(
            source.display_path(),
            1,
            1,
            Severity::Warning,
            "readability-file-size",
            &format!(
                "File has {} lines (max {})",
                lines.len(),
                config.max_file_lines
            ),
            lines.first().unwrap_or(&""),
        ));
    }
    diags.extend(check_function_lengths(&lines, source, config.max_function_lines));
    diags.extend(check_nesting_depth(&lines, source, config.max_nesting_depth));
    diags
}

fn check_function_lengths(
    lines: &[&str],
    source: &SourceFile,
    max_lines: usize,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let mut fn_start: Option<usize> = None;
    let mut pending_sig: Option<usize> = None;
    let mut brace_depth: i32 = 0;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if fn_start.is_none() {
            if pending_sig.is_none() && FN_RE.is_match(trimmed) {
                if trimmed.contains('{') {
                    fn_start = Some(i);
                    brace_depth = trimmed.matches('{').count() as i32
                        - trimmed.matches('}').count() as i32;
                    if brace_depth <= 0 {
                        emit_fn_diag(&mut diags, source, lines, i, i, max_lines);
                        fn_start = None;
                    }
                    continue;
                } else {
                    pending_sig = Some(i);
                    continue;
                }
            }
            if pending_sig.is_some() && trimmed.starts_with('{') {
                fn_start = pending_sig.take();
                brace_depth = trimmed.matches('{').count() as i32
                    - trimmed.matches('}').count() as i32;
                if brace_depth <= 0 {
                    let start = fn_start.unwrap();
                    emit_fn_diag(&mut diags, source, lines, start, i, max_lines);
                    fn_start = None;
                }
                continue;
            }
            if pending_sig.is_some() && !trimmed.is_empty() && !trimmed.starts_with('{') {
                pending_sig = None;
            }
        }
        if let Some(start) = fn_start {
            brace_depth += trimmed.matches('{').count() as i32;
            brace_depth -= trimmed.matches('}').count() as i32;
            if brace_depth <= 0 {
                emit_fn_diag(&mut diags, source, lines, start, i, max_lines);
                fn_start = None;
            }
        }
    }
    if let Some(start) = fn_start {
        emit_fn_diag(&mut diags, source, lines, start, lines.len() - 1, max_lines);
    }
    diags
}

fn emit_fn_diag(
    diags: &mut Vec<Diagnostic>,
    source: &SourceFile,
    lines: &[&str],
    start: usize,
    end: usize,
    max_lines: usize,
) {
    let len = end - start;
    if len > max_lines {
        diags.push(Diagnostic::new_with_source(
            source.display_path(),
            start + 1,
            1,
            Severity::Warning,
            "readability-function-size",
            &format!("Function spans {} lines (max {})", len, max_lines),
            lines.get(start).unwrap_or(&""),
        ));
    }
}

fn check_nesting_depth(
    lines: &[&str],
    source: &SourceFile,
    max_depth: usize,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let mut depth: usize = 0;
    let mut was_over = false;
    for (i, line) in lines.iter().enumerate() {
        let opens = line.matches('{').count();
        let closes = line.matches('}').count();
        depth += opens;
        if depth > max_depth && !was_over {
            diags.push(Diagnostic::new_with_source(
                source.display_path(),
                i + 1,
                1,
                Severity::Warning,
                "readability-deep-nesting",
                &format!(
                    "Nesting depth {} exceeds max {}",
                    depth, max_depth
                ),
                line,
            ));
        }
        was_over = depth > max_depth;
        depth = depth.saturating_sub(closes);
    }
    diags
}
