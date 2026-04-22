use crate::common::source::SourceFile;
use crate::config::{BraceStyle, FormatConfig};

fn contains_close_comment(line: &str) -> bool {
    if let Some(pos) = line.find("*/") {
        let before = &line[..pos];
        let open_count = before.matches("/*").count();
        return open_count == 0;
    }
    false
}

fn contains_open_comment_no_close(line: &str) -> bool {
    if let Some(pos) = line.find("/*") {
        let after = &line[pos + 2..];
        return !after.contains("*/");
    }
    false
}

fn is_protected_line(line: &str, in_block_comment: bool) -> (bool, bool) {
    let trimmed = line.trim_start();
    if in_block_comment {
        let ends = contains_close_comment(line);
        return (true, !ends);
    }
    if trimmed.starts_with("//") || trimmed.starts_with('#') {
        return (true, false);
    }
    if contains_open_comment_no_close(line) {
        return (true, true);
    }
    (false, false)
}

fn brace_in_string_literal(line: &str, brace_pos: usize) -> bool {
    let bytes = line.as_bytes();
    let mut in_string = false;
    let mut i = 0;
    while i < brace_pos.min(bytes.len()) {
        if bytes[i] == b'\\' && in_string {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            in_string = !in_string;
        }
        i += 1;
    }
    in_string
}

fn attach_braces(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut i = 0;
    let mut in_block_comment = false;
    while i < lines.len() {
        let line = lines[i];
        let (protected, new_in_block) = is_protected_line(line, in_block_comment);
        in_block_comment = new_in_block;
        if !protected && i + 1 < lines.len() {
            let next = lines[i + 1];
            let (next_protected, _) = is_protected_line(next, in_block_comment);
            if !next_protected {
                let trimmed_current = line.trim_end();
                let trimmed_next = next.trim();
                if trimmed_next == "{"
                    && should_attach(trimmed_current)
                {
                    result.push(format!("{} {{", trimmed_current));
                    i += 2;
                    continue;
                }
            }
        }
        result.push(line.to_string());
        i += 1;
    }
    let had_newline = content.ends_with('\n');
    let mut out = result.join("\n");
    if had_newline && !out.is_empty() {
        out.push('\n');
    }
    out
}

fn should_attach(line: &str) -> bool {
    line.ends_with(')')
        || line.ends_with("else")
        || line.ends_with("do")
        || line.ends_with("try")
        || line.ends_with("catch")
        || is_type_keyword_line(line)
}

fn is_type_keyword_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("struct ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("union ")
        || trimmed == "struct"
        || trimmed == "enum"
        || trimmed == "union"
}

fn breakout_braces(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    for line in &lines {
        let (protected, new_in_block) = is_protected_line(line, in_block_comment);
        in_block_comment = new_in_block;
        if !protected {
            if let Some(pos) = find_attach_brace(line) {
                let before = &line[..pos];
                let indent = line.len() - line.trim_start().len();
                let prefix = &line[..indent];
                result.push(before.to_string());
                result.push(format!("{}{{", prefix));
                continue;
            }
        }
        result.push(line.to_string());
    }
    let had_newline = content.ends_with('\n');
    let mut out = result.join("\n");
    if had_newline && !out.is_empty() {
        out.push('\n');
    }
    out
}

fn find_attach_brace(line: &str) -> Option<usize> {
    let trimmed = line.trim_end();
    if !trimmed.ends_with(" {") {
        return None;
    }
    let brace_pos = trimmed.rfind(" {").unwrap();
    let before = &trimmed[..brace_pos];
    if before.is_empty() {
        return None;
    }
    if brace_in_string_literal(line, brace_pos + 1) {
        return None;
    }
    let last_char = before.chars().last().unwrap();
    if last_char == ')' || last_char == '"' {
        return Some(brace_pos);
    }
    let tokens = ["else", "do", "try", "catch"];
    for tok in &tokens {
        if before.ends_with(tok) {
            let before_tok = &before[..before.len() - tok.len()];
            if before_tok.is_empty()
                || !before_tok.chars().last().unwrap().is_alphanumeric()
            {
                return Some(brace_pos);
            }
        }
    }
    if is_type_keyword_line(trimmed) {
        return Some(brace_pos);
    }
    None
}

pub fn fix_braces(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    match config.brace_style {
        BraceStyle::Attach => {
            source.content = attach_braces(&source.content);
        }
        BraceStyle::Breakout => {
            source.content = breakout_braces(&source.content);
        }
        BraceStyle::AttachBreakout => {
            source.content = attach_breakout_hybrid(&source.content);
        }
    }
    Ok(())
}

fn attach_breakout_hybrid(content: &str) -> String {
    let attached = attach_braces(content);
    breakout_type_keywords(&attached)
}

fn breakout_type_keywords(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut i = 0;
    let mut in_block_comment = false;
    while i < lines.len() {
        let line = lines[i];
        let (protected, new_in_block) = is_protected_line(line, in_block_comment);
        in_block_comment = new_in_block;
        if !protected {
            let trimmed = line.trim_end();
            if trimmed.ends_with(" {") {
                let brace_pos = trimmed.rfind(" {").unwrap();
                let before = &trimmed[..brace_pos];
                if is_type_keyword_line(trimmed) {
                    let indent = line.len() - line.trim_start().len();
                    let prefix = &line[..indent];
                    result.push(before.to_string());
                    result.push(format!("{}{{", prefix));
                    i += 1;
                    continue;
                }
            }
        }
        result.push(line.to_string());
        i += 1;
    }
    let had_newline = content.ends_with('\n');
    let mut out = result.join("\n");
    if had_newline && !out.is_empty() {
        out.push('\n');
    }
    out
}
