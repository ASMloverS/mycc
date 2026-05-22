use crate::checks::utils;
use crate::cleanse::CleansedLines;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixKind {
    TrailingWhitespace,
    Utf8Bom,
    Crlf,
    BlockComments,
}

pub struct FixEngine;

impl FixEngine {
    pub fn apply(content: &[u8], fixes: &[FixKind]) -> Vec<u8> {
        let mut data = content.to_vec();

        if fixes.contains(&FixKind::Utf8Bom) {
            data = fix_utf8_bom(&data);
        }
        if fixes.contains(&FixKind::Crlf) {
            data = fix_crlf(&data);
        }

        let can_convert_to_str = std::str::from_utf8(&data).is_ok();
        if fixes.contains(&FixKind::TrailingWhitespace) && can_convert_to_str {
            let s = std::str::from_utf8(&data).unwrap();
            data = fix_trailing_whitespace(s).into_bytes();
        }
        if fixes.contains(&FixKind::BlockComments) && can_convert_to_str {
            let s = std::str::from_utf8(&data).unwrap();
            let cleansed = CleansedLines::from_source(s);
            data = fix_block_to_line_comments(s, &cleansed).into_bytes();
        }

        data
    }
}

pub fn fix_trailing_whitespace(content: &str) -> String {
    content
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        + if content.ends_with('\n') { "\n" } else { "" }
}

pub fn fix_utf8_bom(content: &[u8]) -> Vec<u8> {
    if content.len() >= 3 && content[0] == 0xEF && content[1] == 0xBB && content[2] == 0xBF {
        content[3..].to_vec()
    } else {
        content.to_vec()
    }
}

pub fn fix_crlf(content: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(content.len());
    let mut i = 0;
    while i < content.len() {
        if content[i] == b'\r' && i + 1 < content.len() && content[i + 1] == b'\n' {
            result.push(b'\n');
            i += 2;
        } else {
            result.push(content[i]);
            i += 1;
        }
    }
    result
}

pub fn fix_block_to_line_comments(content: &str, lines: &CleansedLines) -> String {
    let raw = lines.raw_lines();
    let cleansed = lines.lines();

    let mut in_block = false;
    let mut block_start_indent = String::new();
    let mut result_lines: Vec<String> = Vec::new();

    for i in 1..raw.len().saturating_sub(1) {
        let raw_line = &raw[i];
        let cleansed_line = &cleansed[i];

        let raw_has_block_start = utils::contains_outside_string(raw_line, "/*");
        let raw_has_block_end = utils::contains_outside_string(raw_line, "*/");
        let cleansed_has_block_start = cleansed_line.contains("/*");
        let cleansed_has_block_end = cleansed_line.contains("*/");

        if in_block {
            let trimmed = raw_line.trim();
            if raw_has_block_end && !cleansed_has_block_end {
                let end_content = strip_block_prefix(find_block_end_content(raw_line).trim());
                push_comment_line(&mut result_lines, &block_start_indent, end_content);
                in_block = false;
            } else {
                push_comment_line(&mut result_lines, &block_start_indent, strip_block_prefix(trimmed));
            }
            continue;
        }

        if raw_has_block_start && cleansed_has_block_start {
            result_lines.push(raw_line.clone());
            continue;
        }

        if raw_has_block_start && !cleansed_has_block_start {
            let single_line = raw_has_block_end && !cleansed_has_block_end;
            if single_line {
                result_lines.push(raw_line.clone());
                continue;
            }

            let indent = raw_line.len() - raw_line.trim_start().len();
            block_start_indent = raw_line[..indent].to_string();

            let comment_text = find_after_block_start(raw_line).trim();
            push_comment_line(&mut result_lines, &block_start_indent, comment_text);
            in_block = true;
            continue;
        }

        result_lines.push(raw_line.clone());
    }

    let trailing_newline = content.ends_with('\n');
    let mut result = result_lines.join("\n");
    if trailing_newline {
        result.push('\n');
    }
    result
}

fn strip_block_prefix(s: &str) -> &str {
    s.strip_prefix(" * ")
        .or_else(|| s.strip_prefix(" *"))
        .or_else(|| s.strip_prefix("* "))
        .or_else(|| s.strip_prefix("*"))
        .unwrap_or(s)
}

fn push_comment_line(result: &mut Vec<String>, indent: &str, content: &str) {
    if content.is_empty() {
        result.push(format!("{}//", indent));
    } else {
        result.push(format!("{}// {}", indent, content));
    }
}

fn find_after_block_start(line: &str) -> &str {
    if let Some(pos) = line.find("/*") {
        &line[pos + 2..]
    } else {
        ""
    }
}

fn find_block_end_content(line: &str) -> &str {
    if let Some(pos) = line.find("*/") {
        &line[..pos]
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_trailing_whitespace() {
        let input = "int x = 1;   \nint y = 2;\t\n";
        let result = fix_trailing_whitespace(input);
        assert_eq!(result, "int x = 1;\nint y = 2;\n");
        assert!(!result.contains("   "));
    }

    #[test]
    fn test_fix_trailing_whitespace_no_trailing() {
        let input = "int x = 1;\nint y = 2;\n";
        let result = fix_trailing_whitespace(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_fix_trailing_whitespace_no_final_newline() {
        let input = "int x = 1;  ";
        let result = fix_trailing_whitespace(input);
        assert_eq!(result, "int x = 1;");
    }

    #[test]
    fn test_fix_utf8_bom_present() {
        let mut input = vec![0xEF, 0xBB, 0xBF];
        input.extend_from_slice(b"int x = 1;");
        let result = fix_utf8_bom(&input);
        assert_eq!(result, b"int x = 1;");
    }

    #[test]
    fn test_fix_utf8_bom_absent() {
        let input = b"int x = 1;".to_vec();
        let result = fix_utf8_bom(&input);
        assert_eq!(result, b"int x = 1;");
    }

    #[test]
    fn test_fix_crlf() {
        let input = b"line1\r\nline2\r\nline3\r\n";
        let result = fix_crlf(input);
        assert_eq!(result, b"line1\nline2\nline3\n");
    }

    #[test]
    fn test_fix_crlf_no_crlf() {
        let input = b"line1\nline2\n";
        let result = fix_crlf(input);
        assert_eq!(result, input.as_slice());
    }

    #[test]
    fn test_fix_crlf_bare_cr() {
        let input = b"line1\r\nline2\rnope\n";
        let result = fix_crlf(input);
        assert_eq!(result, b"line1\nline2\rnope\n");
    }

    #[test]
    fn test_fix_block_to_line_comments_basic() {
        let source = "/* line1\n * line2\n */";
        let cleansed = CleansedLines::from_source(source);
        let result = fix_block_to_line_comments(source, &cleansed);
        assert!(result.contains("// line1"));
        assert!(result.contains("// line2"));
        assert!(!result.contains("/*"));
    }

    #[test]
    fn test_fix_block_to_line_comments_single_line_preserved() {
        let source = "int x = /* inline */ 1;";
        let cleansed = CleansedLines::from_source(source);
        let result = fix_block_to_line_comments(source, &cleansed);
        assert!(result.contains("/* inline */"));
    }

    #[test]
    fn test_fix_block_to_line_comments_string_not_touched() {
        let source = r#"const char* s = "/* not a comment */";"#;
        let cleansed = CleansedLines::from_source(source);
        let result = fix_block_to_line_comments(source, &cleansed);
        assert!(result.contains("/* not a comment */"));
    }

    #[test]
    fn test_fix_engine_all() {
        let mut input = vec![0xEF, 0xBB, 0xBF];
        input.extend_from_slice(b"int x = 1;   \r\nint y = 2;\r\n");
        let result = FixEngine::apply(&input, &[
            FixKind::Utf8Bom,
            FixKind::TrailingWhitespace,
            FixKind::Crlf,
            FixKind::BlockComments,
        ]);
        let s = std::str::from_utf8(&result).unwrap();
        assert!(!s.starts_with("\u{FEFF}"));
        assert!(!s.contains("\r\n"));
        assert!(!s.contains("   \n"));
    }

    #[test]
    fn test_fix_engine_selective() {
        let input = b"line\r\n";
        let result = FixEngine::apply(input, &[FixKind::Crlf]);
        assert_eq!(result, b"line\n");
    }

    #[test]
    fn test_fix_block_to_line_comments_empty_lines() {
        let source = "/* line1\n *\n * line3\n */";
        let cleansed = CleansedLines::from_source(source);
        let result = fix_block_to_line_comments(source, &cleansed);
        assert!(result.contains("// line1"));
        assert!(result.contains("// line3"));
    }
}
