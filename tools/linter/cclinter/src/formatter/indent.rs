use crate::common::source::SourceFile;
use crate::common::string_utils::split_outside_strings;
use crate::config::FormatConfig;

struct LineScan {
    brace_opens: i32,
    brace_closes: i32,
    paren_opens: i32,
    paren_closes: i32,
    in_block_comment: bool,
}

fn scan_line(line: &str, in_block_comment: bool) -> LineScan {
    let parts = split_outside_strings(line);
    let mut brace_opens = 0i32;
    let mut brace_closes = 0i32;
    let mut paren_opens = 0i32;
    let mut paren_closes = 0i32;
    let mut local_in_block = in_block_comment;

    for part in &parts {
        if part.starts_with('"') || part.starts_with('\'') {
            continue;
        }
        let chars: Vec<char> = part.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if local_in_block {
                if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '/' {
                    local_in_block = false;
                    i += 2;
                    continue;
                }
                i += 1;
            } else {
                if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
                    local_in_block = true;
                    i += 2;
                    continue;
                }
                if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
                    break;
                }
                match chars[i] {
                    '{' => brace_opens += 1,
                    '}' => brace_closes += 1,
                    '(' => paren_opens += 1,
                    ')' => paren_closes += 1,
                    _ => {}
                }
                i += 1;
            }
        }
    }

    LineScan {
        brace_opens,
        brace_closes,
        paren_opens,
        paren_closes,
        in_block_comment: local_in_block,
    }
}

pub fn fix_indent(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.use_tabs {
        return Ok(());
    }
    let indent_width = config.indent_width;
    if indent_width == 0 {
        return Ok(());
    }

    let had_newline = source.content.ends_with('\n');
    let lines: Vec<&str> = source.content.lines().collect();
    let mut depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    let mut in_block_comment = false;
    let mut result: Vec<String> = Vec::with_capacity(lines.len());

    for line in &lines {
        let trimmed = line.trim_start();

        if trimmed.is_empty() {
            result.push(String::new());
            continue;
        }

        if in_block_comment {
            result.push(line.to_string());
            let scan = scan_line(trimmed, in_block_comment);
            in_block_comment = scan.in_block_comment;
            continue;
        }

        if trimmed.starts_with('#') {
            result.push(line.to_string());
            let scan = scan_line(trimmed, false);
            in_block_comment = scan.in_block_comment;
            continue;
        }

        let scan = scan_line(trimmed, false);
        in_block_comment = scan.in_block_comment;

        let extra = if paren_depth > 0 { indent_width } else { 0 };

        if trimmed.starts_with('}') {
            let line_depth = (depth - scan.brace_closes).max(0);
            result.push(format!(
                "{}{}",
                " ".repeat(line_depth as usize * indent_width + extra),
                trimmed
            ));
        } else {
            result.push(format!(
                "{}{}",
                " ".repeat(depth as usize * indent_width + extra),
                trimmed
            ));
        }

        depth += scan.brace_opens - scan.brace_closes;
        paren_depth += scan.paren_opens - scan.paren_closes;
        if depth < 0 {
            depth = 0;
        }
        if paren_depth < 0 {
            paren_depth = 0;
        }
    }

    let mut out = result.join("\n");
    if had_newline && !out.is_empty() {
        out.push('\n');
    }
    source.content = out;
    Ok(())
}
