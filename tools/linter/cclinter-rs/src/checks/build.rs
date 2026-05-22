#![allow(clippy::needless_range_loop)]
use std::sync::LazyLock;

use regex::Regex;

use crate::error::ErrorCategory;
use crate::headers::{classify_include, IncludeKind, HEADERS_CONTAINING_TEMPLATES};
use crate::lint_context::LintContext;

use super::utils;

static INCLUDE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\s*#\s*include\s+([<"])(.*?)([>"])"#).unwrap());

static USING_NAMESPACE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"using\s+namespace\s+([\w:]+)").unwrap());

static FORWARD_DECL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:class|struct)\s+([\w:]+)\s*;").unwrap());

static EXPLICIT_MAKE_PAIR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bstd::make_pair\s*<").unwrap());

static REGISTER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bregister\s+\w+").unwrap());

static USING_DECL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"using\s+\w+::\w+").unwrap());

static CPP11_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:nullptr|override|final|noexcept|auto\s+\w+\s*=|enum\s+class|constexpr)\b").unwrap());

static CPP17_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:std::optional|std::variant|std::any|std::string_view|if\s+constexpr|struct\s+\w+\s*\{[^}]*\})\b").unwrap());

static PRINTF_FN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:printf|fprintf|sprintf|snprintf)\s*\(").unwrap());

pub fn check_header_guard(ctx: &mut LintContext) {
    if !ctx.file_info.is_header {
        return;
    }
    let expected_guard = ctx.file_info.header_guard(ctx.config.root.as_deref().unwrap_or(""));
    let raw = ctx.lines.raw_lines();
    let limit = std::cmp::min(100, raw.len());

    let mut found_ifndef = false;
    let mut found_define = false;

    for i in 1..limit {
        let trimmed = raw[i].trim();
        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("#ifndef ") {
            let guard = rest.trim();
            if !found_ifndef {
                found_ifndef = true;
                if guard != expected_guard {
                    ctx.report(
                        i,
                        ErrorCategory::BuildHeaderGuard,
                        1,
                        &format!("{} is wrong header guard style, expected {}", guard, expected_guard),
                    );
                }
            }
        }
        if found_ifndef && !found_define {
            if let Some(rest) = trimmed.strip_prefix("#define ") {
                let guard = rest.trim();
                found_define = true;
                if guard != expected_guard {
                    ctx.report(
                        i,
                        ErrorCategory::BuildHeaderGuard,
                        1,
                        &format!("#define {} does not match #ifndef {}", guard, expected_guard),
                    );
                }
            }
        }
        if found_define {
            break;
        }
    }

    if !found_ifndef {
        ctx.report(
            0,
            ErrorCategory::BuildHeaderGuard,
            1,
            "No #ifndef header guard found",
        );
    }
}

pub fn check_include_order(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        if let Some(caps) = INCLUDE_RE.captures(&raw[i]) {
            let open = &caps[1];
            let path = &caps[2];
            let _close = &caps[3];
            let kind = if open == "<" { IncludeKind::System } else { IncludeKind::Quoted };
            let section = classify_include(path, &ctx.filename, "", kind);
            let violations = ctx.include_state.check_include_order(path, i, section, &ctx.filename);
            for v in violations {
                ctx.report(v.linenum, v.category, v.confidence, &v.message);
            }
        }
    }
}

pub fn check_include(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        if let Some(caps) = INCLUDE_RE.captures(&raw[i]) {
            let open = &caps[1];
            let path = &caps[2];
            if open == "<" {
                if let Some(pos) = path.rfind('.') {
                    let ext = &path[pos + 1..];
                    if ext == "h" {
                        let stem = &path[..pos];
                        if crate::headers::CPP_HEADERS.contains(stem) {
                            ctx.report(
                                i,
                                ErrorCategory::BuildInclude,
                                1,
                                &format!("Include <{}>; use <c{}> instead", path, stem),
                            );
                        }
                    }
                }
            }
            if open == "\"" && !path.contains('/') && ctx.filename.contains('/') {
                let file_dir = &ctx.filename[..ctx.filename.rfind('/').unwrap()];
                ctx.report(i, ErrorCategory::BuildIncludeSubdir, 1,
                    &format!("Include the directory when naming .h files: {} (expected {}/{})",
                        path, file_dir, path));
            }
        }
    }
}

fn is_symbol_used(line: &str, symbol: &str) -> bool {
    let Some(pos) = line.find(symbol) else { return false };
    let end = pos + symbol.len();
    if end >= line.len() {
        return true;
    }
    matches!(line.as_bytes()[end], b'(' | b'<' | b':' | b' ' | b';' | b'=' | b',')
}

pub fn check_include_what_you_use(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);

    let mut included_headers: std::collections::HashSet<String> = std::collections::HashSet::new();
    for i in 1..last {
        if let Some(caps) = INCLUDE_RE.captures(&raw[i]) {
            let path = format!("{}{}{}", &caps[1], &caps[2], &caps[3]);
            included_headers.insert(path);
        }
    }

    for (symbol, headers) in HEADERS_CONTAINING_TEMPLATES.iter() {
        let found_in_code = (1..last).any(|i| is_symbol_used(&elided[i], symbol));
        if found_in_code {
            let all_included = headers.iter().all(|h| included_headers.contains(*h));
            if !all_included {
                for h in headers {
                    if !included_headers.contains(*h) {
                        ctx.report(
                            0,
                            ErrorCategory::BuildIncludeWhatYouUse,
                            1,
                            &format!("Add #include {} for {}", h, symbol),
                        );
                    }
                }
            }
        }
    }
}

pub fn check_namespaces(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = elided[i].trim();
        if let Some(caps) = USING_NAMESPACE_RE.captures(line) {
            let ns = &caps[1];
            if ns == "std" {
                if ctx.file_info.is_header {
                    ctx.report(i, ErrorCategory::BuildNamespaces, 1, "Do not use using namespace std in headers");
                } else {
                    ctx.report(i, ErrorCategory::BuildNamespaces, 1, "Do not use using namespace std");
                }
            }
        }
    }
}

pub fn check_namespaces_headers(ctx: &mut LintContext) {
    if !ctx.file_info.is_header {
        return;
    }
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = elided[i].trim();
        if USING_DECL_RE.is_match(line) && !line.contains("namespace") {
            ctx.report(i, ErrorCategory::BuildNamespacesHeaders, 1, "Using declarations should not be in header files");
        }
    }
}

pub fn check_namespaces_literals(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = elided[i].trim();
        if line.contains("using namespace") && line.contains("literals") {
            ctx.report(i, ErrorCategory::BuildNamespacesLiterals, 1, "Do not use using namespace for literals");
        }
    }
}

pub fn check_cpp11(ctx: &mut LintContext) {
    if ctx.is_c_file {
        return;
    }
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = elided[i].trim();
        if line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        if CPP11_RE.is_match(line) {
            ctx.report(i, ErrorCategory::BuildCpp11, 1, "Use of C++11 feature detected");
        }
    }
}

pub fn check_cpp17(ctx: &mut LintContext) {
    if ctx.is_c_file {
        return;
    }
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = elided[i].trim();
        if line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        if CPP17_RE.is_match(line) {
            ctx.report(i, ErrorCategory::BuildCpp17, 1, "Use of C++17 feature detected");
        }
    }
}

pub fn check_deprecated(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = elided[i].trim();
        if line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        if line.contains("std::auto_ptr") {
            ctx.report(i, ErrorCategory::BuildDeprecated, 1, "std::auto_ptr is deprecated; use std::unique_ptr");
        }
        if REGISTER_RE.is_match(line) {
            ctx.report(i, ErrorCategory::BuildDeprecated, 1, "The 'register' keyword is deprecated");
        }
    }
}

pub fn check_endif_comment(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let trimmed = raw[i].trim();
        if trimmed.starts_with("#endif") {
            let rest = trimmed.strip_prefix("#endif").unwrap().trim();
            if rest.is_empty() {
                ctx.report(i, ErrorCategory::BuildEndifComment, 1, "#endif should have a comment explaining the guard");
            } else if !rest.starts_with("//") && !rest.starts_with("/*") {
                ctx.report(i, ErrorCategory::BuildEndifComment, 1, "#endif comment should use // style");
            }
        }
    }
}

pub fn check_explicit_make_pair(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        if EXPLICIT_MAKE_PAIR_RE.is_match(&elided[i]) {
            ctx.report(i, ErrorCategory::BuildExplicitMakePair, 1, "Omit template arguments for std::make_pair");
        }
    }
}

pub fn check_printf_format(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let line = &elided[i];
        if !PRINTF_FN_RE.is_match(line) {
            continue;
        }
        if let Some(fmt_start) = line.find('"') {
            let fmt_end = utils::find_closing_quote(line, fmt_start);
            if fmt_end > fmt_start {
                let fmt = &line[fmt_start + 1..fmt_end];
                let spec_count = utils::count_format_specs(fmt);
                let rest = &line[fmt_end + 1..];
                let arg_count = utils::count_args(rest);
                if spec_count > 0 && arg_count > 0 && spec_count != arg_count {
                    ctx.report(i, ErrorCategory::BuildPrintfFormat, 1,
                        &format!("printf format has {} specifiers but {} arguments", spec_count, arg_count));
                }
            }
        }
    }
}

pub fn check_storage_class(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = elided[i].trim();
        if line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        if REGISTER_RE.is_match(line) {
            ctx.report(i, ErrorCategory::BuildStorageClass, 1, "The 'register' storage class is deprecated");
        }
        if line.contains("extern") && line.contains("=") && !line.contains("extern \"C\"") {
            ctx.report(i, ErrorCategory::BuildStorageClass, 1, "extern declaration should not have initialization");
        }
    }
}

pub fn check_forward_decl(ctx: &mut LintContext) {
    let elided = ctx.lines.elided();
    let last = elided.len().saturating_sub(1);
    for i in 1..last {
        let line = elided[i].trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        if let Some(caps) = FORWARD_DECL_RE.captures(line) {
            let name = &caps[1];
            if name.starts_with("std::") || line.contains("namespace std") {
                ctx.report(i, ErrorCategory::BuildForwardDecl, 1,
                    &format!("Do not forward declare {} in namespace std", name));
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
    fn test_header_guard_present() {
        let src = "#ifndef TEST_H_\n#define TEST_H_\nint x;\n#endif  // TEST_H_";
        check_test!(check_header_guard, src, "test.h", 0);
    }

    #[test]
    fn test_header_guard_missing() {
        let src = "int x;\nint y;";
        check_test!(check_header_guard, src, "test.h", 1);
    }

    #[test]
    fn test_header_guard_wrong_style() {
        let src = "#ifndef WRONG_GUARD\n#define WRONG_GUARD\nint x;\n#endif";
        check_test!(check_header_guard, src, "test.h", 2);
    }

    #[test]
    fn test_header_guard_not_header() {
        let src = "int x;";
        check_test!(check_header_guard, src, "test.cc", 0);
    }

    #[test]
    fn test_include_order_ok() {
        let src = "#include <stdio.h>\n#include <vector>\n#include \"test.h\"";
        check_test!(check_include_order, src, "test.cc", 0);
    }

    #[test]
    fn test_include_c_style_cpp_header() {
        let src = "#include <string.h>";
        check_test!(check_include, src, "test.cc", 1);
    }

    #[test]
    fn test_using_namespace_std() {
        let src = "using namespace std;";
        check_test!(check_namespaces, src, "test.cc", 1);
    }

    #[test]
    fn test_using_namespace_std_header() {
        let src = "using namespace std;";
        check_test!(check_namespaces, src, "test.h", 1);
    }

    #[test]
    fn test_using_decl_in_header() {
        let src = "using std::string;";
        check_test!(check_namespaces_headers, src, "test.h", 1);
    }

    #[test]
    fn test_using_decl_in_source() {
        let src = "using std::string;";
        check_test!(check_namespaces_headers, src, "test.cc", 0);
    }

    #[test]
    fn test_literals_namespace() {
        let src = "using namespace std::string_literals;";
        check_test!(check_namespaces_literals, src, "test.cc", 1);
    }

    #[test]
    fn test_cpp11_nullptr() {
        let src = "int* p = nullptr;";
        check_test!(check_cpp11, src, "test.cc", 1);
    }

    #[test]
    fn test_cpp17_optional() {
        let src = "std::optional<int> x;";
        check_test!(check_cpp17, src, "test.cc", 1);
    }

    #[test]
    fn test_deprecated_auto_ptr() {
        let src = "std::auto_ptr<int> p;";
        check_test!(check_deprecated, src, "test.cc", 1);
    }

    #[test]
    fn test_endif_comment_missing() {
        let src = "#define FOO\n#ifdef FOO\nint x;\n#endif";
        check_test!(check_endif_comment, src, "test.cc", 1);
    }

    #[test]
    fn test_endif_comment_ok() {
        let src = "#define FOO\n#ifdef FOO\nint x;\n#endif  // FOO";
        check_test!(check_endif_comment, src, "test.cc", 0);
    }

    #[test]
    fn test_explicit_make_pair() {
        let src = "auto p = std::make_pair<int, int>(1, 2);";
        check_test!(check_explicit_make_pair, src, "test.cc", 1);
    }

    #[test]
    fn test_register_keyword() {
        let src = "register int x = 0;";
        check_test!(check_deprecated, src, "test.cc", 1);
    }

    #[test]
    fn test_extern_with_init() {
        let src = "extern int x = 0;";
        check_test!(check_storage_class, src, "test.cc", 1);
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
    fn test_forward_decl_std() {
        let src = "class std::vector;";
        check_test!(check_forward_decl, src, "test.cc", 1);
    }

    #[test]
    fn test_forward_decl_ok() {
        let src = "class Foo;";
        check_test!(check_forward_decl, src, "test.cc", 0);
    }

    #[test]
    fn test_include_subdir_bare_in_subdir() {
        let src = "#include \"foo.h\"";
        check_test!(check_include, src, "bar/test.cc", 1);
    }

    #[test]
    fn test_include_subdir_with_path() {
        let src = "#include \"bar/foo.h\"";
        check_test!(check_include, src, "bar/test.cc", 0);
    }

    #[test]
    fn test_include_subdir_no_subdir_file() {
        let src = "#include \"foo.h\"";
        check_test!(check_include, src, "test.cc", 0);
    }
}
