#![allow(clippy::needless_range_loop)]
use std::sync::LazyLock;

use regex::Regex;

use crate::error::ErrorCategory;
use crate::lint_context::LintContext;

use super::utils;

static PRINTF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(printf|fprintf|sprintf|snprintf)\s*\(").unwrap());

static LONG_TYPE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(long\s+long|long\s+int|unsigned\s+int|unsigned\s+long|short\s+int)\b").unwrap());

static EXPLICIT_CTOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:(?:inline|const|static|virtual)\s+)*(\w+)\s*\(\s*([^,)]+)\s*\)\s*\{").unwrap());

static OPERATOR_OVERLOAD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"operator\s*(&&|\|\||,)\s*\(").unwrap());

static MEMBER_STRING_REF_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"const\s+std::string\s*&\s*\w+\s*;").unwrap());

static THREADSAFE_FNS_RE: LazyLock<Regex> = LazyLock::new(|| {
    let pattern = format!(
        r"\b({})\s*\(",
        ["strtok", "asctime", "ctime", "gmtime", "localtime",
         "rand", "getgrgid", "getgrnam", "getlogin", "getpwuid", "ttyname"].join("|")
    );
    Regex::new(&pattern).unwrap()
});

static MEMSET_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bmemset\s*\(").unwrap());

static STATIC_STRING_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*static\s+(?:std::)?string\s+\w+").unwrap());

static INVALID_INCREMENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\w+\+\+").unwrap());

static VLOG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bVLOG\s*\(").unwrap());

pub fn check_references(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        if let Some(pos) = find_non_const_ref(line) {
            if !is_in_string(line, pos) {
                ctx.report(i, ErrorCategory::RuntimeReferences, 1,
                    "Non-const reference parameter — use pointer or const reference");
            }
        }
    }
}

fn find_non_const_ref(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut paren_depth = 0i32;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => paren_depth += 1,
            b')' => {
                paren_depth -= 1;
                if paren_depth < 0 {
                    return None;
                }
            }
            b'&' if paren_depth > 0 => {
                if i > 0 && bytes[i - 1] == b' ' {
                    let before = &line[..i].trim_end_matches(' ');
                    if !before.ends_with("const") && !before.ends_with("static")
                        && !before.ends_with("return") && !before.ends_with("&&")
                    {
                        return Some(i);
                    }
                }
            }
            b'"' => {
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if bytes[i] == b'"' {
                        break;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn is_in_string(line: &str, pos: usize) -> bool {
    let mut in_str = false;
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < pos && i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            in_str = !in_str;
        }
        i += 1;
    }
    in_str
}

pub fn check_string(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if PRINTF_RE.is_match(line) {
            ctx.report(i, ErrorCategory::RuntimeString, 1,
                "Use streams or logging instead of printf");
        }
    }
}

pub fn check_printf(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if PRINTF_RE.is_match(line) {
            ctx.report(i, ErrorCategory::RuntimePrintf, 1,
                "printf/scanf should not be used; use streams or logging");
        }
    }
}

pub fn check_printf_format(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let line = &raw[i];
        if !PRINTF_RE.is_match(line) {
            continue;
        }
        if let Some(fmt_start) = line.find('"') {
            let fmt_end = utils::find_closing_quote(line, fmt_start);
            if fmt_end > fmt_start {
                let fmt = &line[fmt_start + 1..fmt_end];
                let spec_count = utils::count_format_specs(fmt);
                let rest = line[fmt_end + 1..].trim_start_matches([',', ' ', '\t']);
                let arg_count = utils::count_args(rest);
                if spec_count > 0 && arg_count > 0 && spec_count != arg_count {
                    ctx.report(i, ErrorCategory::RuntimePrintfFormat, 1,
                        &format!("printf format has {} specifiers but {} arguments", spec_count, arg_count));
                }
            }
        }
    }
}

pub fn check_int(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        if line.contains("long long") {
            ctx.report(i, ErrorCategory::RuntimeInt, 1,
                "Use int64_t instead of long long");
        } else if LONG_TYPE_RE.is_match(line) {
            ctx.report(i, ErrorCategory::RuntimeInt, 1,
                "Use sized integer types (int16_t, int32_t, etc.) instead of long/short");
        }
    }
}

pub fn check_explicit(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        if !trimmed.contains("explicit") {
            if let Some(caps) = EXPLICIT_CTOR_RE.captures(trimmed) {
                let _name = &caps[1];
                let args = &caps[2];
                if !args.contains(',') && !args.trim().is_empty() {
                    ctx.report(i, ErrorCategory::RuntimeExplicit, 1,
                        "Single-argument constructors should be marked explicit");
                }
            }
        }
        if line.contains("operator bool()") && !line.contains("explicit") {
            ctx.report(i, ErrorCategory::RuntimeExplicit, 1,
                "Conversion operators should be marked explicit");
        }
    }
}

pub fn check_casting(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if line.contains("static_cast<void>") {
            ctx.report(i, ErrorCategory::RuntimeCasting, 1,
                "Do not use static_cast<void> — use (void) or [[maybe_unused]]");
        }
    }
}

pub fn check_memset(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if MEMSET_RE.is_match(line)
            && line.contains("sizeof(") {
                let after_memset = &line[line.find("memset").unwrap() + 6..];
                if let Some(first_comma) = after_memset.find(',') {
                    let second_arg = after_memset[first_comma + 1..].trim();
                    if second_arg.starts_with('0') {
                        ctx.report(i, ErrorCategory::RuntimeMemset, 1,
                            "memset on non-POD type may be dangerous");
                    }
                }
            }
    }
}

pub fn check_init(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if STATIC_STRING_RE.is_match(line) {
            ctx.report(i, ErrorCategory::RuntimeInit, 1,
                "Non-trivial static initialization — consider using lazy initialization");
        }
    }
}

pub fn check_operator(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if OPERATOR_OVERLOAD_RE.is_match(line) {
            ctx.report(i, ErrorCategory::RuntimeOperator, 1,
                "Do not overload operator &&, ||, or ,");
        }
    }
}

pub fn check_arrays(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        if let Some(bracket_pos) = line.find('[') {
            let inside = &line[bracket_pos + 1..line.find(']').unwrap_or(line.len())];
            if !inside.is_empty() {
                let trimmed_inside = inside.trim();
                if !trimmed_inside.is_empty()
                    && trimmed_inside.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
                    && !trimmed_inside.starts_with("N")
                    && !trimmed_inside.contains("const")
                    && !trimmed_inside.contains("constexpr")
                {
                    ctx.report(i, ErrorCategory::RuntimeArrays, 1,
                        "Do not use variable-length arrays; use std::vector or std::array");
                }
            }
        }
    }
}

pub fn check_invalid_increment(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if INVALID_INCREMENT_RE.is_match(line) {
            ctx.report(i, ErrorCategory::RuntimeInvalidIncrement, 1,
                "Confusing pointer arithmetic: *foo++ — consider parentheses");
        }
    }
}

pub fn check_member_string_references(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        if MEMBER_STRING_REF_RE.is_match(&elided[i]) {
            ctx.report(i, ErrorCategory::RuntimeMemberStringReferences, 1,
                "Do not store const string& as member; use std::string or std::string_view");
        }
    }
}

pub fn check_threadsafe_fn(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        if THREADSAFE_FNS_RE.is_match(&elided[i]) {
            ctx.report(i, ErrorCategory::RuntimeThreadsafeFn, 1,
                "Function is not threadsafe; use reentrant alternative");
        }
    }
}

pub fn check_vlog(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if VLOG_RE.is_match(line) {
            if let Some(open) = line.find("VLOG(") {
                let rest = &line[open + 5..];
                if let Some(close) = rest.find(')') {
                    let arg = rest[..close].trim();
                    if arg.contains('(') || arg.contains("++") || arg.contains("--") {
                        ctx.report(i, ErrorCategory::RuntimeVlog, 1,
                            "VLOG argument should not have side effects");
                    }
                }
            }
        }
    }
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

    #[test]
    fn test_printf_usage() {
        check_test!(check_printf, r#"printf("hello %s", name);"#, 1);
    }

    #[test]
    fn test_printf_format_mismatch() {
        check_test!(check_printf_format, r#"printf("hello %s %d", name);"#, 1);
    }

    #[test]
    fn test_printf_format_ok() {
        check_test!(check_printf_format, r#"printf("hello %s", name);"#, 0);
    }

    #[test]
    fn test_int_long_long() {
        check_test!(check_int, "long long x = 0;", 1);
    }

    #[test]
    fn test_int_ok() {
        check_test!(check_int, "int x = 0;", 0);
    }

    #[test]
    fn test_explicit_missing() {
        check_test!(check_explicit, "Foo(int x) {", 1);
    }

    #[test]
    fn test_explicit_present() {
        check_test!(check_explicit, "explicit Foo(int x) {", 0);
    }

    #[test]
    fn test_casting_void() {
        check_test!(check_casting, "static_cast<void>(x);", 1);
    }

    #[test]
    fn test_memset_usage() {
        check_test!(check_memset, "memset(&obj, 0, sizeof(obj));", 1);
    }

    #[test]
    fn test_static_string_init() {
        check_test!(check_init, "static string s = \"hello\";", 1);
    }

    #[test]
    fn test_operator_overload_bad() {
        check_test!(check_operator, "bool operator&&(const T& a, const T& b);", 1);
    }

    #[test]
    fn test_operator_overload_ok() {
        check_test!(check_operator, "bool operator==(const T& a, const T& b);", 0);
    }

    #[test]
    fn test_arrays_vla() {
        check_test!(check_arrays, "int arr[n];", 1);
    }

    #[test]
    fn test_arrays_const_size() {
        check_test!(check_arrays, "int arr[10];", 0);
    }

    #[test]
    fn test_invalid_increment() {
        check_test!(check_invalid_increment, "*ptr++;", 1);
    }

    #[test]
    fn test_threadsafe_fn() {
        check_test!(check_threadsafe_fn, "char* t = strtok(s, \",\");", 1);
    }

    #[test]
    fn test_threadsafe_ok() {
        check_test!(check_threadsafe_fn, "int x = foo();", 0);
    }

    #[test]
    fn test_vlog_side_effects() {
        check_test!(check_vlog, "VLOG(foo());", 1);
    }

    #[test]
    fn test_vlog_ok() {
        check_test!(check_vlog, "VLOG(1);", 0);
    }

    #[test]
    fn test_string_printf() {
        check_test!(check_string, r#"printf("hello");"#, 1);
    }

    #[test]
    fn test_count_format_specs() {
        assert_eq!(utils::count_format_specs("%d %s %f"), 3);
        assert_eq!(utils::count_format_specs("%%"), 0);
        assert_eq!(utils::count_format_specs("hello"), 0);
    }

    #[test]
    fn test_count_args() {
        assert_eq!(utils::count_args(")"), 0);
        assert_eq!(utils::count_args("a, b)"), 2);
        assert_eq!(utils::count_args("a)"), 1);
    }

    #[test]
    fn test_references_non_const() {
        check_test!(check_references, "void foo(int &x);", 1);
    }

    #[test]
    fn test_references_const() {
        check_test!(check_references, "void foo(const int& x);", 0);
    }
}
