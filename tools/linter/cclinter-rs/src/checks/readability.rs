#![allow(clippy::needless_range_loop)]
use std::sync::LazyLock;

use regex::Regex;

use crate::error::ErrorCategory;
use crate::lint_context::LintContext;

static C_STYLE_CAST_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\(\s*(?:int|float|double|long|short|unsigned|char|void|bool)\s*\*?\s*\)\s*[^=]").unwrap());

static SINGLE_ARG_CTOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:(?:inline|const|static|virtual)\s+)*(\w+)\s*\(\s*([^,)]+)\s*\)\s*(?::\s*\w+\([^)]*\)\s*)*\s*\{").unwrap());

static ALT_TOKENS: &[(&str, &str)] = &[
    ("and", "&&"),
    ("or", "||"),
    ("not", "!"),
    ("xor", "^"),
    ("bitor", "|"),
    ("bitand", "&"),
    ("compl", "~"),
    ("and_eq", "&="),
    ("or_eq", "|="),
    ("xor_eq", "^="),
    ("not_eq", "!="),
];

static ALT_TOKEN_RES: LazyLock<Vec<(&'static str, &'static str, Regex)>> = LazyLock::new(|| {
    ALT_TOKENS.iter().map(|(alt, standard)| {
        (*alt, *standard, Regex::new(&format!(r"\b{}\b", alt)).unwrap())
    }).collect()
});

static TODO_READABILITY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"//\s*(TODO|FIXME|XXX)\b").unwrap());

static TODO_WITH_USER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"//\s*(TODO|FIXME|XXX)\([^)]+\)").unwrap());

static INHERITANCE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*class\s+\w+\s*:\s*").unwrap());

pub fn check_casting(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        if C_STYLE_CAST_RE.is_match(line)
            && !line.contains("static_cast") && !line.contains("const_cast")
                && !line.contains("reinterpret_cast") && !line.contains("dynamic_cast")
            {
                ctx.report(i, ErrorCategory::ReadabilityCasting, 1,
                    "Using C-style cast. Use static_cast<>() instead");
            }
        if line.contains("reinterpret_cast") {
            ctx.report(i, ErrorCategory::ReadabilityCasting, 1,
                "Do not use reinterpret_cast");
        }
    }
}

pub fn check_constructors(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        if !trimmed.contains("explicit") {
            if let Some(caps) = SINGLE_ARG_CTOR_RE.captures(trimmed) {
                let _name = &caps[1];
                let args = &caps[2];
                if !args.contains(',') && !args.trim().is_empty() {
                    ctx.report(i, ErrorCategory::ReadabilityConstructors, 1,
                        "Single-argument constructors should be marked explicit");
                }
            }
        }
    }
}

pub fn check_fn_size(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    let mut in_function = false;
    let mut brace_depth = 0i32;

    for i in 1..last {
        let line = &elided[i];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if !in_function {
            if !is_function_definition(trimmed) {
                continue;
            }
            in_function = true;
            brace_depth = 0;
            let is_test = trimmed.contains("TEST") || trimmed.contains("test");
            ctx.function_state.begin(trimmed, i, is_test);
        } else {
            ctx.function_state.count_line();
        }

        for c in trimmed.chars() {
            match c {
                '{' => brace_depth += 1,
                '}' => {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        if let Some(v) = ctx.function_state.end() {
                            ctx.report(v.linenum, v.category, v.confidence, &v.message);
                        }
                        in_function = false;
                    }
                }
                _ => {}
            }
        }
    }
}

fn is_function_definition(line: &str) -> bool {
    if !line.contains('(') || !line.contains(')') {
        return false;
    }
    if line.contains("class ") || line.contains("struct ") || line.contains("namespace ") {
        return false;
    }
    if line.starts_with("typedef ") || line.starts_with("using ") {
        return false;
    }
    if line.contains('=') && !line.contains("operator=") {
        return false;
    }
    if let Some(brace_pos) = line.find('{') {
        let before_brace = &line[..brace_pos];
        if before_brace.contains(')') {
            return true;
        }
    }
    false
}

pub fn check_braces_readability(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last.saturating_sub(1) {
        let line = elided[i].trim();
        let next_line = raw.get(i + 1).map(|s| s.trim()).unwrap_or("");
        if line.is_empty() || line.contains('{') || line.starts_with('#') {
            continue;
        }
        if next_line.is_empty() {
            continue;
        }
        let is_control = line.starts_with("if ")
            || line.starts_with("if(")
            || line.starts_with("else if ")
            || line.starts_with("else if(")
            || line.starts_with("for ")
            || line.starts_with("for(")
            || line.starts_with("while ")
            || line.starts_with("while(")
            || line == "else"
            || line == "do";
        if is_control && !line.ends_with('{') && !next_line.starts_with('{') {
            ctx.report(i, ErrorCategory::ReadabilityBraces, 1,
                "If/else/for/while body should use braces");
        }
    }
}

pub fn check_strings(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if !count_char(line, '"').is_multiple_of(2) {
            ctx.report(i, ErrorCategory::ReadabilityMultilineString, 1,
                "Multi-line string literal");
        }
    }
}

fn count_char(s: &str, c: char) -> usize {
    let mut count = 0;
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            in_escape = false;
            continue;
        }
        if ch == '\\' {
            in_escape = true;
            continue;
        }
        if ch == c {
            count += 1;
        }
    }
    count
}

pub fn check_todo_readability(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let line = &raw[i];
        if TODO_READABILITY_RE.is_match(line) && !TODO_WITH_USER_RE.is_match(line) {
            ctx.report(i, ErrorCategory::ReadabilityTodo, 1,
                "TODO/FIXME/XXX should have a username: TODO(username)");
        }
    }
}

pub fn check_namespace_readability(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = elided[i].trim();
        if line.contains("using namespace std") {
            ctx.report(i, ErrorCategory::ReadabilityNamespace, 1,
                "Do not use using namespace std");
        }
    }
}

pub fn check_alt_tokens(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        for (alt, standard, re) in ALT_TOKEN_RES.iter() {
            if re.is_match(line) {
                ctx.report(i, ErrorCategory::ReadabilityAltTokens, 1,
                    &format!("Use {} instead of {}", standard, alt));
            }
        }
    }
}

pub fn check_check(ctx: &mut LintContext) {
    if ctx.file_info.is_header {
        let raw = ctx.lines.raw_lines();
        let last = raw.len().saturating_sub(1);
        for i in 1..last {
            let trimmed = raw[i].trim();
            if trimmed.contains("#pragma once") {
                ctx.report(i, ErrorCategory::ReadabilityCheck, 1,
                    "#pragma once is non-standard; use #ifndef guard instead");
            }
        }
    }
}

pub fn check_inheritance(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if INHERITANCE_RE.is_match(line) {
            let colon_pos = line.find(':').unwrap();
            let after_colon = &line[colon_pos + 1..];
            let base_count = count_bases(after_colon);
            if base_count > 1 {
                ctx.report(i, ErrorCategory::ReadabilityInheritance, 1,
                    "Multiple inheritance is discouraged");
            }
        }
    }
}

fn count_bases(s: &str) -> usize {
    let mut depth = 0i32;
    let mut count = 1;
    for c in s.chars() {
        match c {
            '<' | '(' => depth += 1,
            '>' | ')' => depth -= 1,
            '{' => break,
            ',' if depth == 0 => count += 1,
            _ => {}
        }
    }
    count
}

pub fn check_multiline_comment(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let line = &raw[i];
        if line.contains("/*") && line.contains("*/") {
            continue;
        }
        if line.contains("/*") && !line.trim().starts_with('*') && !line.trim().starts_with("/*") {
            ctx.report(i, ErrorCategory::ReadabilityMultilineComment, 1,
                "Multi-line comment should use // style");
        }
    }
}

pub fn check_nolint(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let line = &raw[i];
        if line.contains("NOLINT") && !line.contains("NOLINT(") {
            if let Some(comment_pos) = line.find("//") {
                let comment = &line[comment_pos..];
                if comment.contains("NOLINT") && !comment.contains("NOLINT(") {
                    ctx.report(i, ErrorCategory::ReadabilityNolint, 1,
                        "NOLINT without category — consider adding NOLINT(category)");
                }
            }
        }
    }
}

pub fn check_nul(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if line.contains("NUL") && !line.contains("NULL") && !line.contains("nullptr") && !line.contains("NULLED") {
            ctx.report(i, ErrorCategory::ReadabilityNul, 1,
                "Use NULL or nullptr instead of NUL");
        }
    }
}

pub fn check_utf8(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let line = &raw[i];
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            continue;
        }
        let in_string = starts_inside_string(line);
        let char_iter = line.char_indices().peekable();
        let mut in_str = in_string;
        for (idx, ch) in char_iter {
            if ch == '"' && idx > 0 && line.as_bytes().get(idx - 1) != Some(&b'\\') {
                in_str = !in_str;
            }
            if !in_str && ch as u32 > 127 {
                ctx.report(i, ErrorCategory::ReadabilityUtf8, 1,
                    "Non-ASCII character in source code");
                break;
            }
        }
    }
}

fn starts_inside_string(s: &str) -> bool {
    let mut count = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            count += 1;
        }
        i += 1;
    }
    count % 2 == 1
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
            assert_eq!(ctx.violations.len(), $expected_count,
                "Expected {} violations, got {}: {:?}", $expected_count, ctx.violations.len(),
                ctx.violations.iter().map(|v| &v.message).collect::<Vec<_>>());
        }};
    }

    macro_rules! check_test_file {
        ($check_fn:ident, $source:expr, $filename:expr, $expected_count:expr) => {{
            let lines = CleansedLines::from_source($source);
            let filter = FilterSet::new();
            let config = Config::default();
            let mut ctx = LintContext::new($filename, &lines, &filter, &config);
            $check_fn(&mut ctx);
            assert_eq!(ctx.violations.len(), $expected_count,
                "Expected {} violations, got {}: {:?}", $expected_count, ctx.violations.len(),
                ctx.violations.iter().map(|v| &v.message).collect::<Vec<_>>());
        }};
    }

    #[test]
    fn test_c_style_cast() {
        check_test!(check_casting, "int x = (int)y;", 1);
    }

    #[test]
    fn test_no_cast() {
        check_test!(check_casting, "int x = static_cast<int>(y);", 0);
    }

    #[test]
    fn test_reinterpret_cast() {
        check_test!(check_casting, "int* p = reinterpret_cast<int*>(v);", 1);
    }

    #[test]
    fn test_fn_size_normal() {
        let mut lines = vec!["void foo() {".to_string()];
        for _ in 0..50 {
            lines.push("  int x = 0;".to_string());
        }
        lines.push("}".to_string());
        check_test!(check_fn_size, &lines.join("\n"), 0);
    }

    #[test]
    fn test_braces_readability() {
        check_test!(check_braces_readability, "if (x)\n  return;", 1);
    }

    #[test]
    fn test_braces_readability_with_braces() {
        check_test!(check_braces_readability, "if (x) {\n  return;\n}", 0);
    }

    #[test]
    fn test_strings_multiline() {
        check_test!(check_strings, r#"const char* s = "hello"#, 1);
    }

    #[test]
    fn test_strings_ok() {
        check_test!(check_strings, r#"const char* s = "hello";"#, 0);
    }

    #[test]
    fn test_todo_no_user() {
        check_test!(check_todo_readability, "// TODO: fix this", 1);
    }

    #[test]
    fn test_todo_with_user() {
        check_test!(check_todo_readability, "// TODO(user): fix this", 0);
    }

    #[test]
    fn test_namespace_using_std() {
        check_test!(check_namespace_readability, "using namespace std;", 1);
    }

    #[test]
    fn test_alt_tokens_and() {
        check_test!(check_alt_tokens, "bool x = a and b;", 1);
    }

    #[test]
    fn test_alt_tokens_ok() {
        check_test!(check_alt_tokens, "bool x = a && b;", 0);
    }

    #[test]
    fn test_nul_usage() {
        check_test!(check_nul, "if (c == NUL)", 1);
    }

    #[test]
    fn test_nul_ok() {
        check_test!(check_nul, "if (p == NULL)", 0);
    }

    #[test]
    fn test_nolint_without_category() {
        check_test!(check_nolint, "int x = 0;  // NOLINT", 1);
    }

    #[test]
    fn test_nolint_with_category() {
        check_test!(check_nolint, "int x = 0;  // NOLINT(whitespace/tab)", 0);
    }

    #[test]
    fn test_count_bases_single() {
        assert_eq!(count_bases("public Base {"), 1);
    }

    #[test]
    fn test_count_bases_multiple() {
        assert_eq!(count_bases("public Base1, public Base2 {"), 2);
    }

    #[test]
    fn test_check_pragma_once() {
        check_test_file!(check_check, "#pragma once", "test.h", 1);
    }

    #[test]
    fn test_check_pragma_once_in_source() {
        check_test_file!(check_check, "#pragma once", "test.cc", 0);
    }

    #[test]
    fn test_multiline_comment_block() {
        check_test!(check_multiline_comment, "/* bad comment */", 0);
    }

    #[test]
    fn test_multiline_comment_start() {
        check_test!(check_multiline_comment, "int x; /* bad", 1);
    }

    #[test]
    fn test_utf8_non_ascii() {
        check_test!(check_utf8, "int résumé = 0;", 1);
    }

    #[test]
    fn test_utf8_ascii_only() {
        check_test!(check_utf8, "int x = 0;", 0);
    }

    #[test]
    fn test_inheritance_multiple() {
        check_test!(check_inheritance, "class Derived : public Base1, public Base2 {", 1);
    }

    #[test]
    fn test_inheritance_single() {
        check_test!(check_inheritance, "class Derived : public Base {", 0);
    }
}
