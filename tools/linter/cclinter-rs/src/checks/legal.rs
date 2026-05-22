use crate::error::ErrorCategory;
use crate::lint_context::LintContext;

pub fn check_copyright(ctx: &mut LintContext) {
    let raw_lines = ctx.lines.raw_lines();
    let limit = std::cmp::min(11, raw_lines.len());
    let mut found_copyright = false;
    let mut found_year = false;
    let mut copyright_line = 0usize;

    for (i, raw_line) in raw_lines.iter().enumerate().take(limit).skip(1) {
        let line = raw_line.to_ascii_lowercase();
        if line.contains("copyright") {
            found_copyright = true;
            copyright_line = i;
            if contains_year(raw_line) {
                found_year = true;
                break;
            }
        }
    }

    if !found_copyright {
        ctx.report(
            0,
            ErrorCategory::LegalCopyright,
            80,
            "No copyright message found.  You must have a line:\
             \n  Copyright [year] <Copyright Owner>",
        );
    } else if !found_year {
        ctx.report(
            copyright_line,
            ErrorCategory::LegalCopyright,
            80,
            "No copyright year found.  You must have a line:\
             \n  Copyright [year] <Copyright Owner>",
        );
    }
}

fn contains_year(line: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    if chars.len() < 4 {
        return false;
    }
    for window in chars.windows(4) {
        if window.iter().all(|c| c.is_ascii_digit()) {
            let year: String = window.iter().collect();
            if let Ok(y) = year.parse::<u32>() {
                if (1900..=2100).contains(&y) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cleanse::CleansedLines;
    use crate::config::Config;
    use crate::filter::FilterSet;

    macro_rules! assert_copyright {
        ($source:expr, $expect_violation:expr, $msg_pat:expr) => {{
            let lines = CleansedLines::from_source($source);
            let filter = FilterSet::new();
            let config = Config::default();
            let mut ctx = LintContext::new("test.cc", &lines, &filter, &config);
            check_copyright(&mut ctx);
            if $expect_violation {
                assert_eq!(ctx.violations.len(), 1);
                assert_eq!(ctx.violations[0].category, ErrorCategory::LegalCopyright);
                assert!(ctx.violations[0].message.contains($msg_pat));
            } else {
                assert!(ctx.violations.is_empty());
            }
        }};
    }

    #[test]
    fn test_copyright_present() {
        assert_copyright!("// Copyright 2024 Acme Inc\nint x = 1;", false, "");
    }

    #[test]
    fn test_copyright_missing() {
        assert_copyright!("int x = 1;\nint y = 2;", true, "No copyright message found");
    }

    #[test]
    fn test_copyright_case_insensitive() {
        assert_copyright!("// copyright 2024 acme inc\nint x = 1;", false, "");
    }

    #[test]
    fn test_copyright_beyond_10_lines() {
        let mut lines: Vec<String> = (1..=10).map(|i| format!("// line {}", i)).collect();
        lines.push("// Copyright 2024 Acme Inc".to_string());
        lines.push("int x = 1;".to_string());
        let source = lines.join("\n");
        assert_copyright!(&source, true, "No copyright message found");
    }

    #[test]
    fn test_copyright_with_year() {
        assert_copyright!("// Copyright 2024 My Corp\nint x = 1;", false, "");
    }

    #[test]
    fn test_copyright_empty_file() {
        assert_copyright!("", true, "No copyright message found");
    }

    #[test]
    fn test_copyright_without_year() {
        assert_copyright!("// Copyright Acme Inc\nint x = 1;", true, "No copyright year found");
    }

    #[test]
    fn test_copyright_mixed_case() {
        assert_copyright!("// COPYRIGHT 2024 Acme Inc\nint x = 1;", false, "");
    }

    #[test]
    fn test_copyright_on_line_10() {
        let mut lines: Vec<String> = (1..=9).map(|i| format!("// line {}", i)).collect();
        lines.push("// Copyright 2024 Acme Inc".to_string());
        lines.push("int x = 1;".to_string());
        let source = lines.join("\n");
        assert_copyright!(&source, false, "");
    }

    #[test]
    fn test_contains_year_valid() {
        assert!(contains_year("Copyright 2024 Acme"));
        assert!(contains_year("Copyright 1999 Foo"));
        assert!(contains_year("Copyright 2100 Bar"));
    }

    #[test]
    fn test_contains_year_invalid() {
        assert!(!contains_year("Copyright Acme Inc"));
        assert!(!contains_year("123"));
        assert!(!contains_year("Copyright 1899 Old"));
        assert!(!contains_year("Copyright 2101 Future"));
    }
}
