use crate::config::FormatConfig;
use crate::cst::{CSTLine, CSTSource, IndentInfo};

pub fn fix_line_length(
    source: &mut CSTSource,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let limit = config.column_limit;
    let indent_width = config.indent_width;
    let mut new_lines = Vec::new();
    for line in source.lines.drain(..) {
        if should_skip(&line) || code_width(&line) <= limit {
            new_lines.push(line);
            continue;
        }
        let base_indent = line.indent.raw.clone();
        new_lines.extend(wrap_line(&line, limit, indent_width, 0, &base_indent));
    }
    source.lines = new_lines;
    Ok(())
}

fn code_width(line: &CSTLine) -> usize {
    line.indent.width + line.code.chars().count()
}

fn should_skip(line: &CSTLine) -> bool {
    if line.is_blank || line.code.is_empty() {
        return true;
    }
    let trimmed = line.code.trim();
    trimmed.starts_with("http://") || trimmed.starts_with("https://")
}

fn extract_ending(raw: &str) -> &str {
    if raw.ends_with("\r\n") {
        "\r\n"
    } else if raw.ends_with('\n') {
        "\n"
    } else if raw.ends_with('\r') {
        "\r"
    } else {
        "\n"
    }
}

fn is_operator_char(ch: char) -> bool {
    matches!(ch, '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^')
}

fn find_break_point(code: &str, available: usize, initial_depth: i32) -> Option<(usize, i32)> {
    let chars: Vec<(usize, char)> = code.char_indices().collect();
    let n = chars.len();
    let mut best_comma: Option<(usize, i32)> = None;
    let mut best_comma_col = 0usize;
    let mut best_lbracket: Option<(usize, i32)> = None;
    let mut best_lbracket_col = 0usize;
    let mut best_operator: Option<(usize, i32)> = None;
    let mut best_operator_col = 0usize;
    let mut best_space: Option<(usize, i32)> = None;
    let mut best_space_col = 0usize;
    let mut paren_depth = initial_depth;
    let mut in_string = false;
    let mut string_quote = '\0';
    let mut col = 0usize;
    let mut i = 0usize;

    while i < n {
        let (byte_pos, ch) = chars[i];
        if col >= available {
            break;
        }

        if in_string {
            if ch == '\\' && i + 1 < n {
                i += 2;
                col += 2;
                continue;
            }
            if ch == string_quote {
                in_string = false;
            }
            col += 1;
            i += 1;
            continue;
        }

        match ch {
            '\'' | '"' => {
                if i + 2 < n && chars[i + 1].1 == ch && chars[i + 2].1 == ch {
                    i += 3;
                    col += 3;
                    while i + 2 < n {
                        if chars[i].1 == '\\' && i + 1 < n {
                            i += 2;
                            col += 2;
                            continue;
                        }
                        if chars[i].1 == ch && chars[i + 1].1 == ch && chars[i + 2].1 == ch {
                            i += 3;
                            col += 3;
                            break;
                        }
                        col += 1;
                        i += 1;
                    }
                } else {
                    in_string = true;
                    string_quote = ch;
                    col += 1;
                    i += 1;
                }
                continue;
            }
            '(' | '[' | '{' => {
                paren_depth += 1;
                if col + 1 <= available && col + 1 > best_lbracket_col {
                    best_lbracket = Some((byte_pos + ch.len_utf8(), paren_depth));
                    best_lbracket_col = col + 1;
                }
            }
            ')' | ']' | '}' => {
                paren_depth = paren_depth.saturating_sub(1);
            }
            ',' => {
                if paren_depth > 0 && col + 1 <= available && col + 1 > best_comma_col {
                    best_comma = Some((byte_pos + ch.len_utf8(), paren_depth));
                    best_comma_col = col + 1;
                }
            }
            _ => {
                if paren_depth > 0 {
                    if is_operator_char(ch)
                        && col > 0
                        && col + 1 <= available
                        && col > best_operator_col
                    {
                        best_operator = Some((byte_pos, paren_depth));
                        best_operator_col = col;
                    }
                    if ch == ' ' && col > 0 && col <= available && col > best_space_col {
                        best_space = Some((byte_pos, paren_depth));
                        best_space_col = col;
                    }
                }
            }
        }
        col += 1;
        i += 1;
    }

    best_comma
        .or(best_lbracket)
        .or(best_operator)
        .or(best_space)
}

fn wrap_line(
    line: &CSTLine,
    limit: usize,
    indent_width: usize,
    inherited_depth: i32,
    base_indent: &str,
) -> Vec<CSTLine> {
    let available = limit.saturating_sub(line.indent.width);
    let (break_pos, depth_at_break) = match find_break_point(&line.code, available, inherited_depth)
    {
        Some(r) => r,
        None => return vec![line.clone()],
    };

    let first = line.code[..break_pos].trim_end();
    let rest = line.code[break_pos..].trim_start();

    if first.is_empty() || rest.is_empty() {
        return vec![line.clone()];
    }

    let ending = extract_ending(&line.raw_content);
    let cont_indent = format!("{}{}", base_indent, " ".repeat(indent_width));
    let comment = line.comment.clone();
    let cont_ws = if comment.is_some() {
        "  ".to_string()
    } else {
        String::new()
    };

    let mut first_line = line.clone();
    first_line.code = first.to_string();
    first_line.trailing_ws.clear();
    first_line.comment = None;
    first_line.raw_content = format!("{}{}{}", line.indent.raw, first, ending);

    let cont_raw = format!(
        "{}{}{}{}{}",
        cont_indent,
        rest,
        cont_ws,
        comment.as_deref().unwrap_or(""),
        ending
    );

    let cont = CSTLine {
        num: 0,
        indent: IndentInfo {
            level: line.indent.level.saturating_add(1),
            raw: cont_indent.clone(),
            width: cont_indent.chars().count(),
            uses_tabs: false,
        },
        tokens: Vec::new(),
        raw_content: cont_raw,
        code: rest.to_string(),
        trailing_ws: cont_ws,
        comment,
        is_blank: false,
    };

    let mut result = vec![first_line];
    if code_width(&cont) > limit {
        result.extend(wrap_line(&cont, limit, indent_width, depth_at_break, base_indent));
    } else {
        result.push(cont);
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::config::FormatConfig;
    use crate::cst::CSTSource;

    fn wrapped(input: &str, limit: usize) -> String {
        let mut config = FormatConfig::default();
        config.column_limit = limit;
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_line_length(&mut cst, &config).unwrap();
        cst.regenerate()
    }

    #[test]
    fn no_wrap_short_line() {
        assert_eq!(wrapped("x = 1\n", 120), "x = 1\n");
    }

    #[test]
    fn no_wrap_at_limit() {
        assert_eq!(wrapped("x = 1\n", 5), "x = 1\n");
    }

    #[test]
    fn wrap_long_function_call() {
        let input = "result = func(a1, a2, a3, a4, a5, a6, a7, a8)\n";
        let result = wrapped(input, 30);
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() > 1);
        assert!(lines[0].trim_end().ends_with(','));
    }

    #[test]
    fn skip_comment_only_line() {
        let long = format!("# {}\n", "x".repeat(200));
        assert_eq!(wrapped(&long, 80), long);
    }

    #[test]
    fn skip_blank_line() {
        assert_eq!(wrapped("\n", 80), "\n");
    }

    #[test]
    fn wrap_preserves_indent() {
        let input = "def f():\n    result = func(a1, a2, a3, a4, a5, a6, a7)\n";
        let result = wrapped(input, 40);
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() > 2);
        assert!(lines[2].starts_with("        "));
    }

    #[test]
    fn idempotent_wrapping() {
        let input = "result = func(a1, a2, a3, a4, a5, a6, a7, a8)\n";
        let first = wrapped(input, 30);
        let second = wrapped(&first, 30);
        assert_eq!(first, second);
    }

    #[test]
    fn no_break_point_no_wrap() {
        let input = format!("x = {}\n", "a".repeat(200));
        assert_eq!(wrapped(&input, 80), input);
    }

    #[test]
    fn comment_on_last_line_after_wrap() {
        let input = "result = func(a1, a2, a3, a4, a5, a6)  # comment\n";
        let result = wrapped(input, 30);
        assert!(result.lines().last().unwrap().contains("# comment"));
    }

    #[test]
    fn break_after_opening_bracket() {
        let input = format!("x = long_name({})\n", "'arg', ".repeat(10));
        let result = wrapped(&input, 30);
        assert!(result.contains('\n'));
    }

    #[test]
    fn string_content_not_broken() {
        let input = "f('a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q')\n";
        let result = wrapped(input, 30);
        let lines: Vec<&str> = result.lines().collect();
        for l in &lines {
            let in_string = l.contains("a,b,c");
            assert!(!in_string || l.contains("'a,b,c"));
        }
    }

    #[test]
    fn multiple_long_lines() {
        let input = "r1 = f(a1, a2, a3, a4, a5, a6, a7, a8, a9)\nr2 = g(b1, b2, b3, b4, b5, b6, b7, b8, b9)\n";
        let result = wrapped(input, 30);
        assert!(result.lines().count() > 2);
    }

    #[test]
    fn no_wrap_empty_file() {
        assert_eq!(wrapped("", 80), "");
    }

    #[test]
    fn no_wrap_code_with_long_comment() {
        let input = format!("x = 1  # {}\n", "c".repeat(200));
        let result = wrapped(&input, 80);
        assert_eq!(result, input);
    }

    #[test]
    fn wrap_at_operator() {
        let input = "result = compute(a1 + a2 + a3 + a4 + a5 + a6 + a7)\n";
        let result = wrapped(input, 30);
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() > 1);
    }

    #[test]
    fn wrap_at_operator_idempotent() {
        let input = "result = compute(a1 + a2 + a3 + a4 + a5 + a6 + a7)\n";
        let first = wrapped(input, 30);
        let second = wrapped(&first, 30);
        assert_eq!(first, second);
    }

    #[test]
    fn wrap_at_space_fallback() {
        let input = "f(some_very_long_name_that_exceeds_limit_by_a_lot, other)\n";
        let result = wrapped(input, 30);
        assert!(result.contains('\n'));
    }
}
