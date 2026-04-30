use crate::common::source::SourceFile;
use crate::config::{CommentStyle, FormatConfig};

pub fn fix_comments(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.comment_style != CommentStyle::DoubleSlash {
        return Ok(());
    }
    if source.content.is_empty() {
        return Ok(());
    }
    source.content = convert_block_comments(&source.content);
    Ok(())
}

fn convert_block_comments(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let mut result = String::with_capacity(content.len());
    let mut i = 0;
    let mut in_string = false;
    while i < chars.len() {
        if in_string {
            if chars[i] == '\\' && i + 1 < chars.len() {
                result.push(chars[i]);
                result.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if chars[i] == '"' {
                in_string = false;
            }
            result.push(chars[i]);
            i += 1;
            continue;
        }
        if chars[i] == '"' {
            in_string = true;
            result.push(chars[i]);
            i += 1;
            continue;
        }
        if chars[i] == '\'' {
            result.push(chars[i]);
            i += 1;
            if i < chars.len() && chars[i] == '\\' {
                result.push(chars[i]);
                i += 1;
                if i < chars.len() {
                    result.push(chars[i]);
                    i += 1;
                }
            } else if i < chars.len() {
                result.push(chars[i]);
                i += 1;
            }
            if i < chars.len() && chars[i] == '\'' {
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }
        if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            let start = i + 2;
            let mut end = start;
            while end + 1 < chars.len() {
                if chars[end] == '*' && chars[end + 1] == '/' {
                    break;
                }
                end += 1;
            }
            if end + 1 >= chars.len() {
                for &c in &chars[i..] {
                    result.push(c);
                }
                break;
            }
            let body: String = chars[start..end].iter().collect();
            result.push_str(&convert_block_body(&body));
            i = end + 2;
            continue;
        }
        if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn convert_block_body(body: &str) -> String {
    let lines: Vec<&str> = body.lines().collect();
    if lines.is_empty() {
        return "//".to_string();
    }
    if lines.len() == 1 {
        let text = strip_comment_prefix(lines[0].trim());
        if text.is_empty() {
            return "//".to_string();
        }
        return format!("// {}", text);
    }
    let converted: Vec<String> = lines
        .iter()
        .map(|line| {
            let after_star = strip_comment_prefix(line.trim_start());
            let text = after_star.trim_end();
            if text.is_empty() {
                "//".to_string()
            } else {
                format!("// {}", text)
            }
        })
        .collect();
    converted.join("\n")
}

fn strip_comment_prefix(line: &str) -> &str {
    let trimmed = line.trim_start();
    if trimmed.starts_with("* ") || trimmed.starts_with("*\t") {
        &trimmed[2..]
    } else if trimmed == "*" {
        ""
    } else {
        trimmed
    }
}
