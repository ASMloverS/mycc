#![allow(clippy::needless_range_loop)]
use std::sync::LazyLock;

use regex::Regex;

use crate::error::ErrorCategory;
use crate::lint_context::LintContext;

static FOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*for\s*\(").unwrap());

static IF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(if|while|switch|catch)\s*\(").unwrap());

static ELIF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*else\s+if\s*\(").unwrap());

static RANGE_FOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"for\s*\([^)]*:[^)]*\)").unwrap());

static TODO_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"TODO\(([^)]*)\)").unwrap());

static URL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://\S+").unwrap());

pub fn check_tab(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        if raw[i].contains('\t') {
            ctx.report(i, ErrorCategory::WhitespaceTab, 1, "Tab found; better to use spaces");
        }
    }
}

pub fn check_indent(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let line = &raw[i];
        if line.is_empty() {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('/') || trimmed.starts_with('*') {
            continue;
        }
        let indent: &str = &line[..line.len() - trimmed.len()];
        if indent.contains('\t') && indent.contains(' ') {
            ctx.report(i, ErrorCategory::WhitespaceIndent, 1, "mixed tab and space indent");
        }
    }
}

pub fn check_indent_namespace(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    let mut local_nesting = crate::nesting::NestingState::new();
    for i in 1..last {
        let line = &raw[i];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        local_nesting.update(trimmed, i);
        if local_nesting.in_namespace() && !local_nesting.in_class() {
            let indent_len = line.len() - trimmed.len();
            if indent_len > 0 && !trimmed.starts_with('}') && !trimmed.starts_with("namespace") {
                ctx.report(i, ErrorCategory::WhitespaceIndentNamespace, 1, "namespace should not be indented");
            }
        }
    }
}

pub fn check_end_of_line(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let line = &raw[i];
        if line.ends_with(' ') || line.ends_with('\t') {
            ctx.report(i, ErrorCategory::WhitespaceEndOfLine, 1, "Line ends in whitespace");
        }
    }
}

pub fn check_line_length(ctx: &mut LintContext) {
    let limit = ctx.config.effective_line_length();
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let line = &raw[i];
        let width = unicode_width(line);
        if width > limit {
            if is_url_comment_line(line) {
                continue;
            }
            ctx.report(
                i,
                ErrorCategory::WhitespaceLineLength,
                1,
                &format!("Lines should be <= {} characters long", limit),
            );
        }
    }
}

pub fn check_braces(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if line.contains("){") {
            ctx.report(i, ErrorCategory::WhitespaceBraces, 1, "Missing space before {");
        }
        if line.contains("}else") {
            ctx.report(i, ErrorCategory::WhitespaceBraces, 1, "Missing space before else");
        }
        if line.contains("}if") || line.contains("}for") || line.contains("}while") || line.contains("}switch") {
            ctx.report(i, ErrorCategory::WhitespaceBraces, 1, "Missing space after }");
        }
        let trimmed = line.trim();
        if trimmed.starts_with("else{") || trimmed.starts_with("do{") || trimmed.starts_with("try{") {
            ctx.report(i, ErrorCategory::WhitespaceBraces, 1, "Missing space before {");
        }
    }
}

pub fn check_blank_line(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    let mut prev_blank = false;
    for i in 1..last {
        let trimmed = raw[i].trim();
        let is_blank = trimmed.is_empty();
        if is_blank && prev_blank {
            ctx.report(i, ErrorCategory::WhitespaceBlankLine, 1, "Consecutive blank lines");
        }
        prev_blank = is_blank;
        if trimmed == "{"
            && i + 1 < last && raw[i + 1].trim().is_empty() {
                ctx.report(i + 1, ErrorCategory::WhitespaceBlankLine, 1, "Redundant blank line at start of code block");
            }
        if trimmed == "}" && i > 1 && raw[i - 1].trim().is_empty() {
            ctx.report(i, ErrorCategory::WhitespaceBlankLine, 1, "Redundant blank line at end of code block");
        }
    }
}

pub fn check_comma(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let bytes = line.as_bytes();
        let mut in_char = false;
        for j in 0..bytes.len() {
            if bytes[j] == b'\'' {
                in_char = !in_char;
                continue;
            }
            if in_char {
                continue;
            }
            if bytes[j] == b',' {
                if j + 1 < bytes.len() && bytes[j + 1] != b' ' && bytes[j + 1] != b'\n' && bytes[j + 1] != b'\r' && bytes[j + 1] != b')' {
                    ctx.report(i, ErrorCategory::WhitespaceComma, 1, "Missing space after ,");
                }
                if j > 0 && bytes[j - 1] == b' ' {
                    ctx.report(i, ErrorCategory::WhitespaceComma, 1, "Extra space before ,");
                }
            }
        }
    }
}

pub fn check_semicolon(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if FOR_RE.is_match(line) {
            continue;
        }
        let bytes = line.as_bytes();
        for j in 0..bytes.len() {
            if bytes[j] == b';' {
                if j > 0 && bytes[j - 1] == b' ' {
                    ctx.report(i, ErrorCategory::WhitespaceSemicolon, 1, "Extra space before ;");
                }
                if j + 1 < bytes.len() && bytes[j + 1] != b' ' && bytes[j + 1] != b'\n' && bytes[j + 1] != b'\r' && bytes[j + 1] != b')' && bytes[j + 1] != b'}' && bytes[j + 1] != b';' {
                    ctx.report(i, ErrorCategory::WhitespaceSemicolon, 1, "Missing space after ;");
                }
            }
        }
    }
}

pub fn check_comments(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let line = &raw[i];
        let bytes = line.as_bytes();
        let mut in_string = false;
        let mut string_char = b' ';
        let mut j = 0;
        while j < bytes.len() {
            if in_string {
                if bytes[j] == b'\\' && j + 1 < bytes.len() {
                    j += 2;
                    continue;
                }
                if bytes[j] == string_char {
                    in_string = false;
                }
                j += 1;
                continue;
            }
            if bytes[j] == b'"' || bytes[j] == b'\'' {
                in_string = true;
                string_char = bytes[j];
                j += 1;
                continue;
            }
            if j + 1 < bytes.len() && bytes[j] == b'/' && bytes[j + 1] == b'/' {
                let comment_start = j + 2;
                if comment_start < bytes.len() {
                    let rest = &line[comment_start..];
                    if !rest.starts_with('/') && !rest.starts_with(' ') && !rest.trim().is_empty()
                        && !line[..j].contains("://") {
                            ctx.report(i, ErrorCategory::WhitespaceComments, 1, "Missing space after //");
                        }
                }
                break;
            }
            j += 1;
        }
    }
}

pub fn check_operators(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        check_operators_in_line(ctx, i, line);
    }
}

fn check_operators_in_line(ctx: &mut LintContext, linenum: usize, line: &str) {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut j = 0;
    while j < len {
        if bytes[j] == b'<' && j + 1 < bytes.len() && bytes[j + 1] == b'<' {
            if j > 0 && j + 2 < len {
                let before = bytes[j - 1];
                let after = bytes[j + 2];
                if before != b' ' && before != b'\t' && before != b'(' && before != b'=' && !is_op_char(before)
                    || after != b' ' && after != b'\t' && after != b'=' && !is_op_char(after)
                {
                    ctx.report(linenum, ErrorCategory::WhitespaceOperators, 1, "Missing spaces around <<");
                }
            }
            j += 2;
            continue;
        }
        if j + 1 < len && bytes[j] == b'=' && bytes[j + 1] == b'=' {
            j += 2;
            continue;
        }
        if j + 1 < len && bytes[j] == b'!' && bytes[j + 1] == b'=' {
            j += 2;
            continue;
        }
        if bytes[j] == b'=' && j > 0 && j + 1 < len {
            let before = bytes[j - 1];
            let after = bytes[j + 1];
            if before == b'=' || after == b'=' {
                j += 1;
                continue;
            }
            let no_space_before = before != b' ' && before != b'\t' && before != b'(' && !is_op_char(before) && !is_assign_prefix(before);
            let no_space_after = after != b' ' && after != b'\t' && !is_op_char(after);
            if no_space_before || no_space_after {
                ctx.report(linenum, ErrorCategory::WhitespaceOperators, 1, "Missing spaces around =");
            }
        }
        j += 1;
    }
}

fn is_op_char(c: u8) -> bool {
    matches!(c, b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^' | b'<' | b'>' | b'=' | b'!')
}

fn is_assign_prefix(c: u8) -> bool {
    matches!(c, b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^' | b'<' | b'>')
}

pub fn check_parens(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        let trimmed = line.trim();
        if IF_RE.is_match(trimmed) || ELIF_RE.is_match(trimmed) || FOR_RE.is_match(trimmed) {
            let bytes = trimmed.as_bytes();
            let paren_pos = trimmed.find('(').unwrap_or(trimmed.len());
            if paren_pos > 0 && bytes[paren_pos - 1] != b' ' {
                let keyword = if FOR_RE.is_match(trimmed) { "for" } else { "if/while" };
                ctx.report(i, ErrorCategory::WhitespaceParens, 1, &format!("Missing space before ( in {}", keyword));
            }
            if paren_pos + 1 < bytes.len() && bytes[paren_pos + 1] == b' ' {
                ctx.report(i, ErrorCategory::WhitespaceParens, 1, "Extra space after (");
            }
            if let Some(close_pos) = find_matching_paren(bytes, paren_pos) {
                if close_pos > 0 && bytes[close_pos - 1] == b' ' {
                    ctx.report(i, ErrorCategory::WhitespaceParens, 1, "Extra space before )");
                }
            }
        }
    }
}

fn find_matching_paren(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    for j in open..bytes.len() {
        match bytes[j] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn check_empty_body(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    let checks: &[(&str, &str, ErrorCategory, &str)] = &[
        ("if ", "if(", ErrorCategory::WhitespaceEmptyIfBody, "Empty if body"),
        ("for ", "for(", ErrorCategory::WhitespaceEmptyLoopBody, "Empty for loop body"),
        ("while ", "while(", ErrorCategory::WhitespaceEmptyLoopBody, "Empty while loop body"),
        ("switch ", "switch(", ErrorCategory::WhitespaceEmptyConditionalBody, "Empty switch body"),
    ];
    for i in 1..last {
        let line = elided[i].trim();
        for &(pfx1, pfx2, category, msg) in checks {
            if (line.starts_with(pfx1) || line.starts_with(pfx2))
                && (line.contains("){") || line.contains(") {"))
            {
                let inner = extract_brace_content(line);
                if inner.trim().is_empty() {
                    ctx.report(i, category, 1, msg);
                }
            }
        }
    }
}

fn extract_brace_content(line: &str) -> &str {
    if let Some(open) = line.find('{') {
        if let Some(close) = line.rfind('}') {
            if close > open {
                return &line[open + 1..close];
            }
        }
    }
    ""
}

pub fn check_newline(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = elided[i].trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        let mut semicolon_count = 0u32;
        let mut in_for = false;
        let bytes = line.as_bytes();
        let mut j = 0;
        while j < bytes.len() {
            if bytes[j] == b'(' && j >= 3 {
                let prefix = &line[..j];
                if prefix.ends_with("for") {
                    in_for = true;
                }
            }
            match bytes[j] {
                b';' => {
                    if !in_for {
                        semicolon_count += 1;
                    }
                }
                b'{' | b'}' => {
                    in_for = false;
                }
                _ => {}
            }
            j += 1;
        }
        if semicolon_count > 1 {
            ctx.report(i, ErrorCategory::WhitespaceNewline, 1, "More than one statement on a single line");
        }
    }
}

pub fn check_ending_newline(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    if raw.len() < 2 {
        ctx.report(0, ErrorCategory::WhitespaceEndingNewline, 1, "Could not find a newline character at the end of the file");
        return;
    }
    if !ctx.lines.ends_with_newline() {
        let last_src = raw.len().saturating_sub(1);
        ctx.report(last_src, ErrorCategory::WhitespaceEndingNewline, 1, "Could not find a newline character at the end of the file");
    }
}

pub fn check_forcolon(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if RANGE_FOR_RE.is_match(line) {
            let bytes = line.as_bytes();
            for j in 0..bytes.len() {
                if bytes[j] == b':' {
                    let before_ok = j > 0 && bytes[j - 1] == b' ';
                    let after_ok = j + 1 < bytes.len() && bytes[j + 1] == b' ';
                    if !before_ok || !after_ok {
                        ctx.report(i, ErrorCategory::WhitespaceForcolon, 1, "Missing space around : in range-based for");
                    }
                    break;
                }
            }
        }
    }
}

pub fn check_todo(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let line = &raw[i];
        if !line.contains("TODO") && !line.contains("FIXME") && !line.contains("XXX") {
            continue;
        }
        if let Some(comment_start) = line.find("//") {
            let comment = &line[comment_start..];
            let trimmed_comment = comment.trim();
            if let Some(caps) = TODO_RE.captures(trimmed_comment) {
                let username = &caps[1];
                if username.is_empty() {
                    ctx.report(i, ErrorCategory::WhitespaceTodo, 1, "TODO should have a non-empty username: TODO(username)");
                }
            } else if trimmed_comment.contains("TODO") && !trimmed_comment.contains("TODO(") {
                ctx.report(i, ErrorCategory::WhitespaceTodo, 1, "TODO should include a username: TODO(username)");
            }
        }
    }
}

fn unicode_width(s: &str) -> usize {
    let mut w = 0usize;
    for ch in s.chars() {
        w += if ch as u32 > 0x7F { 2 } else { 1 };
    }
    w
}

fn is_url_comment_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with("//") {
        return false;
    }
    URL_RE.is_match(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cleanse::CleansedLines;
    use crate::config::Config;
    use crate::filter::FilterSet;
    use crate::lint_context::LintContext;

    macro_rules! check_test {
        ($check_fn:ident, $source:expr, $expected_count:expr) => {{
            let lines = CleansedLines::from_source($source);
            let filter = FilterSet::new();
            let config = Config::default();
            let mut ctx = LintContext::new("test.cc", &lines, &filter, &config);
            $check_fn(&mut ctx);
            assert_eq!(ctx.violations.len(), $expected_count, "Expected {} violations, got {}: {:?}", $expected_count, ctx.violations.len(), ctx.violations.iter().map(|v| &v.message).collect::<Vec<_>>());
        }};
    }

    #[test]
    fn test_tab_found() {
        check_test!(check_tab, "int x;\tint y;", 1);
    }

    #[test]
    fn test_no_tab() {
        check_test!(check_tab, "int x; int y;", 0);
    }

    #[test]
    fn test_end_of_line_space() {
        check_test!(check_end_of_line, "int x = 1; ", 1);
    }

    #[test]
    fn test_end_of_line_clean() {
        check_test!(check_end_of_line, "int x = 1;", 0);
    }

    #[test]
    fn test_line_length_ok() {
        let line = "x".repeat(80);
        check_test!(check_line_length, &line, 0);
    }

    #[test]
    fn test_line_length_too_long() {
        let line = "x".repeat(81);
        check_test!(check_line_length, &line, 1);
    }

    #[test]
    fn test_line_length_url_exempt() {
        let url = format!("// https://example.com/{}", "x".repeat(100));
        check_test!(check_line_length, &url, 0);
    }

    #[test]
    fn test_blank_line_consecutive() {
        check_test!(check_blank_line, "int x;\n\n\nint y;", 1);
    }

    #[test]
    fn test_blank_line_none() {
        check_test!(check_blank_line, "int x;\nint y;", 0);
    }

    #[test]
    fn test_comma_missing_space() {
        check_test!(check_comma, "foo(a,b)", 1);
    }

    #[test]
    fn test_comma_ok() {
        check_test!(check_comma, "foo(a, b)", 0);
    }

    #[test]
    fn test_semicolon_in_for() {
        check_test!(check_semicolon, "for (int i = 0; i < n; i++) {}", 0);
    }

    #[test]
    fn test_semicolon_missing_space() {
        check_test!(check_semicolon, "int x = 1;int y = 2;", 1);
    }

    #[test]
    fn test_comments_no_space() {
        check_test!(check_comments, "//bad comment", 1);
    }

    #[test]
    fn test_comments_ok() {
        check_test!(check_comments, "// good comment", 0);
    }

    #[test]
    fn test_comments_url_ok() {
        check_test!(check_comments, "http://example.com //comment", 1);
    }

    #[test]
    fn test_braces_no_space() {
        check_test!(check_braces, "if (x){", 1);
    }

    #[test]
    fn test_braces_ok() {
        check_test!(check_braces, "if (x) {", 0);
    }

    #[test]
    fn test_empty_if_body() {
        check_test!(check_empty_body, "if (x) {}", 1);
    }

    #[test]
    fn test_empty_for_body() {
        check_test!(check_empty_body, "for (;;) {}", 1);
    }

    #[test]
    fn test_non_empty_if_body() {
        check_test!(check_empty_body, "if (x) { return; }", 0);
    }

    #[test]
    fn test_newline_multi_statement() {
        check_test!(check_newline, "int x = 1; int y = 2;", 1);
    }

    #[test]
    fn test_newline_ok() {
        check_test!(check_newline, "int x = 1;", 0);
    }

    #[test]
    fn test_ending_newline_ok() {
        check_test!(check_ending_newline, "int x = 1;\n", 0);
    }

    #[test]
    fn test_ending_newline_missing() {
        check_test!(check_ending_newline, "int x = 1;", 1);
    }

    #[test]
    fn test_forcolon_ok() {
        check_test!(check_forcolon, "for (auto x : vec) {}", 0);
    }

    #[test]
    fn test_forcolon_no_space() {
        check_test!(check_forcolon, "for (auto x:vec) {}", 1);
    }

    #[test]
    fn test_todo_ok() {
        check_test!(check_todo, "// TODO(user): fix this", 0);
    }

    #[test]
    fn test_todo_no_username() {
        check_test!(check_todo, "// TODO: fix this", 1);
    }

    #[test]
    fn test_indent_mixed() {
        check_test!(check_indent, " \tint x = 1;", 1);
    }

    #[test]
    fn test_indent_spaces_only() {
        check_test!(check_indent, "    int x = 1;", 0);
    }

    #[test]
    fn test_parens_extra_space_after_open() {
        check_test!(check_parens, "if ( x ) {", 2);
    }

    #[test]
    fn test_parens_ok() {
        check_test!(check_parens, "if (x) {", 0);
    }

    #[test]
    fn test_operators_no_space_either_side() {
        check_test!(check_operators, "int x=1;", 1);
    }

    #[test]
    fn test_operators_space_only_after() {
        check_test!(check_operators, "int x= 1;", 1);
    }

    #[test]
    fn test_operators_space_only_before() {
        check_test!(check_operators, "int x =1;", 1);
    }

    #[test]
    fn test_operators_ok() {
        check_test!(check_operators, "int x = 1;", 0);
    }

    #[test]
    fn test_comments_escaped_quote() {
        check_test!(check_comments, r#"const char* s = "he said \"hello\""; // ok"#, 0);
    }

    #[test]
    fn test_braces_else_no_space() {
        check_test!(check_braces, "}else {", 1);
    }

    #[test]
    fn test_braces_else_ok() {
        check_test!(check_braces, "} else {", 0);
    }

    #[test]
    fn test_indent_namespace_body() {
        let src = "namespace foo {\n  int x = 1;\n}";
        check_test!(check_indent_namespace, src, 1);
    }

    #[test]
    fn test_indent_namespace_ok() {
        let src = "namespace foo {\nint x = 1;\n}";
        check_test!(check_indent_namespace, src, 0);
    }
}
