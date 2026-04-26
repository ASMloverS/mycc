use crate::common::source::SourceFile;
use crate::config::FormatConfig;

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
    let result: Vec<String> = lines
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
    let cont_indent = format!("{}{}", leading_ws, " ".repeat(indent_width * 2));
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

fn find_break_point(line: &str, limit: usize) -> Option<usize> {
    let candidates = [',', ' ', ';', '(', ')', '|', '&'];
    let spans = string_spans(line);
    let mut best = None;
    for (char_count, (i, ch)) in line.char_indices().enumerate() {
        if char_count >= limit {
            break;
        }
        if candidates.contains(&ch) {
            let in_string = spans.iter().any(|&(s, e)| i >= s && i < e);
            if !in_string {
                best = Some(i + ch.len_utf8());
            }
        }
    }
    best
}
