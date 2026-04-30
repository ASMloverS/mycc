use crate::common::source::SourceFile;
use crate::config::FormatConfig;

fn merge_continuations(lines: &[&str]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut i = 0;
    let mut in_block_comment = false;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if in_block_comment {
            result.push(line.to_string());
            if line.contains("*/") {
                in_block_comment = false;
            }
            i += 1;
            continue;
        }

        if let Some(pos) = line.find("/*") {
            if !line[pos + 2..].contains("*/") {
                in_block_comment = true;
                result.push(line.to_string());
                i += 1;
                continue;
            }
        }

        if trimmed.is_empty() {
            result.push(line.to_string());
            i += 1;
            continue;
        }
        if trimmed.starts_with('#')
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with("/ *")
        {
            result.push(line.to_string());
            i += 1;
            continue;
        }

        let mut merged = line.to_string();
        let mut j = i + 1;

        while j < lines.len() {
            let next = lines[j];
            let next_trimmed = next.trim();

            if next_trimmed.is_empty() {
                break;
            }
            if next_trimmed.starts_with('#')
                || next_trimmed.starts_with("//")
                || next_trimmed.starts_with("/*")
                || next_trimmed.starts_with("/ *")
            {
                break;
            }
            if next_trimmed.starts_with('}') {
                break;
            }

            let acc = merged.trim_end();
            let code = strip_trailing_line_comment(acc);
            let code_trimmed = code.trim_end();
            if code_trimmed.ends_with(';')
                || code_trimmed.ends_with('{')
                || code_trimmed.ends_with('}')
                || code_trimmed.ends_with(')')
                || code_trimmed.ends_with(':')
                || acc.ends_with("*/")
                || acc.ends_with("* /")
            {
                break;
            }

            merged = format!("{} {}", acc, next_trimmed);
            j += 1;
        }

        result.push(merged);
        i = j;
    }

    result
}

pub fn fix_line_length(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let limit = config.column_limit;
    let indent_width = config.indent_width;
    let had_newline = source.content.ends_with('\n');
    let lines: Vec<&str> = source.content.lines().collect();
    if lines.is_empty() {
        return Ok(());
    }
    let merged = merge_continuations(&lines);
    let result: Vec<String> = merged
        .iter()
        .flat_map(|line| {
            if line.chars().count() <= limit {
                return vec![line.to_string()];
            }
            if line.trim_start().starts_with('#') {
                return vec![line.to_string()];
            }
            wrap_line(line, limit, indent_width)
        })
        .collect();
    let joined = result.join("\n");
    source.content = if had_newline {
        format!("{}\n", joined)
    } else {
        joined
    };
    Ok(())
}

fn wrap_line(line: &str, limit: usize, indent_width: usize) -> Vec<String> {
    let leading_ws: String = line.chars().take_while(|c| c.is_whitespace()).collect();
    let cont_indent = format!("{}{}", leading_ws, " ".repeat(indent_width));
    let mut result = Vec::new();
    let mut current = line.to_string();
    while current.chars().count() > limit {
        let break_pos = match find_break_point(&current, limit) {
            Some(pos) => pos,
            None => break,
        };
        if break_pos <= leading_ws.len() + 4 {
            break;
        }
        let before = current[..break_pos].trim_end().to_string();
        let after = format!("{}{}", cont_indent, current[break_pos..].trim_start());
        result.push(before);
        current = after;
    }
    result.push(current);
    result
}

fn string_spans(s: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut in_string = false;
    let mut escape_next = false;
    let mut string_delim = ' ';
    let mut start = 0;
    for (i, ch) in s.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape_next = true;
            continue;
        }
        if in_string {
            if ch == string_delim {
                spans.push((start, i + ch.len_utf8()));
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = true;
            string_delim = ch;
            start = i;
        }
    }
    spans
}

fn strip_trailing_line_comment(line: &str) -> &str {
    let spans = string_spans(line);
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        let in_string = spans.iter().any(|&(s, e)| i >= s && i < e);
        if !in_string && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            return &line[..i];
        }
        i += 1;
    }
    line
}

fn find_break_point(line: &str, limit: usize) -> Option<usize> {
    let spans = string_spans(line);
    let mut best_pos: Option<usize> = None;
    let mut best_priority: i32 = -1;
    for (char_count, (i, ch)) in line.char_indices().enumerate() {
        if char_count >= limit {
            break;
        }
        let in_string = spans.iter().any(|&(s, e)| i >= s && i < e);
        if in_string {
            continue;
        }
        let priority = match ch {
            ',' => 3,
            ' ' | ';' => 2,
            ')' => 1,
            '(' | '|' | '&' => 0,
            _ => continue,
        };
        if priority >= best_priority {
            best_priority = priority;
            best_pos = Some(i + ch.len_utf8());
        }
    }
    best_pos
}
