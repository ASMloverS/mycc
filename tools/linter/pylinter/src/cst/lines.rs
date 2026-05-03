use super::tokens::LocatedToken;
use rustpython_parser::Tok;

#[derive(Clone, Debug)]
pub struct IndentInfo {
    pub level: usize,
    pub raw: String,
    pub width: usize,
    pub uses_tabs: bool,
}

#[derive(Clone, Debug)]
pub struct CSTLine {
    pub num: usize,
    pub indent: IndentInfo,
    pub tokens: Vec<LocatedToken>,
    pub raw_content: String,
    pub code: String,
    pub trailing_ws: String,
    pub comment: Option<String>,
    pub is_blank: bool,
}

pub fn build_lines(tokens: &[LocatedToken], source: &str) -> Vec<CSTLine> {
    let raw_lines = split_lines_preserving(source);
    let n = raw_lines.len();

    // Track indent level, assign tokens and comments to lines
    let mut indent_level = 0usize;
    let mut line_indent_levels = vec![0usize; n + 2]; // 1-indexed
    let mut line_tokens: Vec<Vec<LocatedToken>> = vec![Vec::new(); n + 2];
    let mut line_comments: Vec<Option<(String, usize)>> = vec![None; n + 2]; // (text, start_col)

    for lt in tokens {
        match &lt.tok {
            Tok::Indent => {
                indent_level += 1;
            }
            Tok::Dedent => {
                indent_level = indent_level.saturating_sub(1);
            }
            Tok::Newline | Tok::NonLogicalNewline => {}
            Tok::Comment(text) => {
                if lt.start_line <= n {
                    if line_indent_levels[lt.start_line] == 0 && indent_level > 0 {
                        line_indent_levels[lt.start_line] = indent_level;
                    }
                    line_comments[lt.start_line] = Some((text.clone(), lt.start_col));
                }
            }
            _ => {
                if lt.start_line <= n {
                    if line_indent_levels[lt.start_line] == 0 {
                        line_indent_levels[lt.start_line] = indent_level;
                    }
                    line_tokens[lt.start_line].push(lt.clone());
                }
            }
        }
    }

    // Build CSTLines
    let mut result = Vec::with_capacity(n);
    for (i, raw_content) in raw_lines.iter().enumerate() {
        let line_num = i + 1;

        let content = raw_content
            .strip_suffix("\r\n")
            .or_else(|| raw_content.strip_suffix('\n'))
            .or_else(|| raw_content.strip_suffix('\r'))
            .unwrap_or(raw_content);

        // Indent from leading whitespace
        let indent_str: String = content
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .collect();
        let uses_tabs = indent_str.contains('\t');
        let visible_width = indent_str
            .chars()
            .map(|c| if c == '\t' { 4 } else { 1 })
            .sum();

        let is_blank = content.trim().is_empty();

        let indent = IndentInfo {
            level: line_indent_levels[line_num],
            raw: indent_str.clone(),
            width: visible_width,
            uses_tabs,
        };

        let trailing_ws = compute_trailing_ws(content, line_comments[line_num].as_ref());

        let comment_text_len = line_comments[line_num].as_ref().map_or(0, |(t, _)| t.len());
        let code_end = content
            .len()
            .saturating_sub(trailing_ws.len())
            .saturating_sub(comment_text_len);
        let code = if code_end > indent_str.len() {
            content[indent_str.len()..code_end].to_string()
        } else {
            String::new()
        };

        result.push(CSTLine {
            num: line_num,
            indent,
            tokens: std::mem::take(&mut line_tokens[line_num]),
            raw_content: raw_content.clone(),
            code,
            trailing_ws,
            comment: line_comments[line_num].take().map(|(t, _)| t),
            is_blank,
        });
    }

    let mut last_level = 0usize;
    for line in &mut result {
        if line.is_blank && line.indent.level == 0 {
            line.indent.level = last_level;
        } else if line.indent.level > 0 {
            last_level = line.indent.level;
        }
    }

    result
}

fn compute_trailing_ws(content: &str, comment: Option<&(String, usize)>) -> String {
    let end = match comment {
        Some((_, col)) => *col,
        None => content.len(),
    };
    if end > content.len() {
        return String::new();
    }
    let code_part = &content[..end];
    let trimmed = code_part.trim_end_matches([' ', '\t']);
    if trimmed.is_empty() {
        return String::new();
    }
    code_part[trimmed.len()..].to_string()
}

/// Split source into raw lines, preserving exact line endings.
fn split_lines_preserving(source: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut start = 0;
    let bytes = source.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                lines.push(source[start..=i].to_string());
                start = i + 1;
                i += 1;
            }
            b'\r' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    lines.push(source[start..=i + 1].to_string());
                    start = i + 2;
                    i += 2;
                } else {
                    lines.push(source[start..=i].to_string());
                    start = i + 1;
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    if start < source.len() {
        lines.push(source[start..].to_string());
    }

    lines
}
