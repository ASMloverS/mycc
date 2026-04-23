use crate::common::source::SourceFile;
use crate::common::string_utils::split_outside_strings;
use crate::config::FormatConfig;
use regex::Regex;
use std::sync::LazyLock;

static SWITCH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*switch\s*\(").unwrap());

static CASE_PREFIX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*)(case\s+.+:|default:)").unwrap());

static BLOCK_COMMENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/\*.*?\*/").unwrap());

fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|&c| c == ' ').count()
}

fn strip_inline_block_comments(s: &str) -> String {
    BLOCK_COMMENT_RE.replace_all(s, "").to_string()
}

fn count_braces_outside_strings(line: &str) -> (i32, i32) {
    let parts = split_outside_strings(line);
    let mut opens = 0i32;
    let mut closes = 0i32;
    for part in &parts {
        if part.starts_with('"') || part.starts_with('\'') {
            continue;
        }
        let cleaned = strip_inline_block_comments(part);
        opens += cleaned.matches('{').count() as i32;
        closes += cleaned.matches('}').count() as i32;
    }
    (opens, closes)
}

fn is_line_comment_or_blank(trimmed: &str) -> bool {
    trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with("* ")
        || trimmed == "*"
}

fn reindent(line: &str, target: usize) -> String {
    let current = leading_spaces(line);
    let rest = &line[current..];
    format!("{}{}", " ".repeat(target), rest)
}

pub fn fix_switch_indent(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.switch_case_indent {
        return Ok(());
    }

    let indent_width = config.indent_width;
    if indent_width == 0 {
        return Ok(());
    }

    let had_newline = source.content.ends_with('\n');
    let lines: Vec<&str> = source.content.lines().collect();
    let mut brace_depth: i32 = 0;
    let mut switch_stack: Vec<(i32, usize)> = Vec::new();
    let mut result: Vec<String> = Vec::with_capacity(lines.len());

    for line in &lines {
        let trimmed = line.trim_start();
        let (opens, closes) = count_braces_outside_strings(line);

        if !switch_stack.is_empty() {
            let (entry_depth, switch_indent) = *switch_stack.last().unwrap();
            if trimmed.starts_with('}') && brace_depth - closes < entry_depth + 1 {
                brace_depth -= closes;
                brace_depth += opens;
                switch_stack.pop();
                let current = leading_spaces(line);
                if current != switch_indent {
                    result.push(reindent(line, switch_indent));
                } else {
                    result.push(line.to_string());
                }
                continue;
            }
        }

        if trimmed.starts_with('}') && !switch_stack.is_empty() {
            let (_, switch_indent) = *switch_stack.last().unwrap();
            let body_indent = switch_indent + indent_width;
            let current = leading_spaces(line);
            if current != body_indent {
                result.push(reindent(line, body_indent));
            } else {
                result.push(line.to_string());
            }
            brace_depth += opens;
            brace_depth -= closes;
            continue;
        }

        if SWITCH_RE.is_match(line) {
            let si = leading_spaces(line);
            let mut effective_si = si;
            let mut output_line = line.to_string();

            if !switch_stack.is_empty() {
                let (_, outer_si) = *switch_stack.last().unwrap();
                let outer_case_indent = outer_si + indent_width;
                if si < outer_case_indent {
                    output_line = reindent(line, outer_case_indent);
                    effective_si = outer_case_indent;
                }
            }

            brace_depth += opens;
            brace_depth -= closes;
            if opens > 0 {
                switch_stack.push((brace_depth - opens, effective_si));
            } else {
                switch_stack.push((brace_depth, effective_si));
            }
            if opens > 0 && closes > 0 && brace_depth <= switch_stack.last().unwrap().0 {
                switch_stack.pop();
            }
            result.push(output_line);
            continue;
        }

        brace_depth += opens;
        brace_depth -= closes;

        if switch_stack.is_empty() {
            result.push(line.to_string());
            continue;
        }

        let (_entry_depth, switch_indent) = *switch_stack.last().unwrap();
        let body_indent = switch_indent + indent_width;
        let case_indent = body_indent;
        let content_indent = body_indent + indent_width;

        if is_line_comment_or_blank(trimmed) {
            result.push(line.to_string());
            continue;
        }

        if CASE_PREFIX_RE.is_match(line) {
            let current = leading_spaces(line);
            if current != case_indent {
                result.push(reindent(line, case_indent));
            } else {
                result.push(line.to_string());
            }
            continue;
        }

        let current = leading_spaces(line);
        if current < content_indent {
            result.push(reindent(line, content_indent));
        } else {
            result.push(line.to_string());
        }
    }

    let mut out = result.join("\n");
    if had_newline && !out.is_empty() {
        out.push('\n');
    }
    source.content = out;
    Ok(())
}
