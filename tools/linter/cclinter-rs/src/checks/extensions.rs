#![allow(clippy::needless_range_loop)]

use crate::error::ErrorCategory;
use crate::lint_context::LintContext;

use super::utils;

pub fn check_block_comment(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    let last = raw.len().saturating_sub(1);
    for i in 1..last {
        let line = &raw[i];
        if !utils::contains_outside_string(line, "/*") {
            continue;
        }
        if utils::contains_outside_string(line, "*/") {
            continue;
        }
        ctx.report(i, ErrorCategory::ExtensionsBlockComment, 1,
            "Block comment should use // style");
    }
}

pub fn check_utf8_bom(ctx: &mut LintContext) {
    let raw = ctx.lines.raw_lines();
    if raw.len() > 1 && raw[1].as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]) {
            ctx.report(1, ErrorCategory::ExtensionsUtf8Bom, 1,
                "UTF-8 BOM found; remove it");
        }
}

pub fn check_utf8_invalid(bytes: &[u8], ctx: &mut LintContext) {
    let mut line_num = 1usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            line_num += 1;
            i += 1;
            continue;
        }
        if bytes[i] < 0x80 {
            i += 1;
            continue;
        }
        let seq_len = if bytes[i] < 0xC0 {
            0
        } else if bytes[i] < 0xE0 {
            2
        } else if bytes[i] < 0xF0 {
            3
        } else if bytes[i] < 0xF8 {
            4
        } else {
            0
        };
        if seq_len == 0 {
            ctx.report(line_num, ErrorCategory::ExtensionsUtf8Invalid, 1,
                "Invalid UTF-8 byte sequence");
            i += 1;
            continue;
        }
        if i + seq_len > bytes.len() {
            ctx.report(line_num, ErrorCategory::ExtensionsUtf8Invalid, 1,
                "Invalid UTF-8 byte sequence (truncated)");
            break;
        }
        let valid = (1..seq_len).all(|j| bytes[i + j] >= 0x80 && bytes[i + j] < 0xC0);
        if !valid {
            ctx.report(line_num, ErrorCategory::ExtensionsUtf8Invalid, 1,
                "Invalid UTF-8 byte sequence");
            i += 1;
            continue;
        }
        if seq_len == 2 && bytes[i] < 0xC2 {
            ctx.report(line_num, ErrorCategory::ExtensionsUtf8Invalid, 1,
                "Invalid UTF-8 byte sequence (overlong)");
            i += seq_len;
            continue;
        }
        if seq_len >= 3 {
            let code_point = decode_code_point(&bytes[i..i + seq_len]);
            if (seq_len == 3 && code_point < 0x800) || (seq_len == 4 && code_point < 0x10000) {
                ctx.report(line_num, ErrorCategory::ExtensionsUtf8Invalid, 1,
                    "Invalid UTF-8 byte sequence (overlong)");
            }
            if (0xD800..=0xDFFF).contains(&code_point) {
                ctx.report(line_num, ErrorCategory::ExtensionsUtf8Invalid, 1,
                    "Invalid UTF-8 byte sequence (surrogate)");
            }
            if code_point > 0x10FFFF {
                ctx.report(line_num, ErrorCategory::ExtensionsUtf8Invalid, 1,
                    "Invalid UTF-8 byte sequence (beyond U+10FFFF)");
            }
        }
        i += seq_len;
    }
}

fn decode_code_point(bytes: &[u8]) -> u32 {
    let len = bytes.len();
    let mut cp = (bytes[0] & (0xFF >> (len + 1))) as u32;
    for &b in &bytes[1..] {
        cp = (cp << 6) | (b & 0x3F) as u32;
    }
    cp
}

pub fn check_crlf(bytes: &[u8], ctx: &mut LintContext) {
    let mut line_num = 1usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
            ctx.report(line_num, ErrorCategory::ExtensionsCrlf, 1,
                "CRLF line ending found; use LF instead");
            i += 2;
            line_num += 1;
            continue;
        }
        if bytes[i] == b'\n' {
            line_num += 1;
        }
        i += 1;
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
    fn test_block_comment_multiline() {
        check_test!(check_block_comment, "int x; /* start\nmiddle\nend */", 1);
    }

    #[test]
    fn test_block_comment_single_line_inline() {
        check_test!(check_block_comment, "int x = /* block */ 1;", 0);
    }

    #[test]
    fn test_block_comment_none() {
        check_test!(check_block_comment, "int x = 1; // line comment", 0);
    }

    #[test]
    fn test_block_comment_in_string() {
        check_test!(check_block_comment, r#"auto s = "/* not a comment";"#, 0);
    }

    #[test]
    fn test_block_comment_standalone_single_line() {
        check_test!(check_block_comment, "/* standalone block comment */", 0);
    }

    #[test]
    fn test_block_comment_empty_file() {
        check_test!(check_block_comment, "", 0);
    }

    #[test]
    fn test_utf8_bom_present() {
        let lines = CleansedLines::from_source("\u{FEFF}int x = 1;");
        let filter = FilterSet::new();
        let config = Config::default();
        let mut ctx = LintContext::new("test.cc", &lines, &filter, &config);
        check_utf8_bom(&mut ctx);
        assert_eq!(ctx.violations.len(), 1);
    }

    #[test]
    fn test_utf8_bom_absent() {
        check_test!(check_utf8_bom, "int x = 1;", 0);
    }

    #[test]
    fn test_utf8_invalid_bytes() {
        let bytes: &[u8] = &[0x80, 0x81, b'\n', b'o', b'k'];
        let lines = CleansedLines::from_source("ok");
        let filter = FilterSet::new();
        let config = Config::default();
        let mut ctx = LintContext::new("test.cc", &lines, &filter, &config);
        check_utf8_invalid(bytes, &mut ctx);
        assert!(ctx.violations.len() >= 1);
    }

    #[test]
    fn test_utf8_invalid_valid_utf8() {
        let valid = "int x = 1;\n".as_bytes();
        let lines = CleansedLines::from_source("int x = 1;");
        let filter = FilterSet::new();
        let config = Config::default();
        let mut ctx = LintContext::new("test.cc", &lines, &filter, &config);
        check_utf8_invalid(valid, &mut ctx);
        assert_eq!(ctx.violations.len(), 0);
    }

    #[test]
    fn test_utf8_invalid_surrogate() {
        let bytes: &[u8] = &[0xED, 0xA0, 0x80, b'\n', b'o', b'k'];
        let lines = CleansedLines::from_source("ok");
        let filter = FilterSet::new();
        let config = Config::default();
        let mut ctx = LintContext::new("test.cc", &lines, &filter, &config);
        check_utf8_invalid(bytes, &mut ctx);
        assert!(ctx.violations.iter().any(|v| v.message.contains("surrogate")));
    }

    #[test]
    fn test_utf8_invalid_beyond_max() {
        let bytes: &[u8] = &[0xF4, 0x90, 0x80, 0x80, b'\n', b'o', b'k'];
        let lines = CleansedLines::from_source("ok");
        let filter = FilterSet::new();
        let config = Config::default();
        let mut ctx = LintContext::new("test.cc", &lines, &filter, &config);
        check_utf8_invalid(bytes, &mut ctx);
        assert!(ctx.violations.iter().any(|v| v.message.contains("U+10FFFF")));
    }

    #[test]
    fn test_crlf_present() {
        let bytes = b"int x = 1;\r\nint y = 2;\r\n";
        let lines = CleansedLines::from_source("int x = 1;\nint y = 2;\n");
        let filter = FilterSet::new();
        let config = Config::default();
        let mut ctx = LintContext::new("test.cc", &lines, &filter, &config);
        check_crlf(bytes, &mut ctx);
        assert_eq!(ctx.violations.len(), 2);
    }

    #[test]
    fn test_crlf_absent() {
        let bytes = b"int x = 1;\nint y = 2;\n";
        let lines = CleansedLines::from_source("int x = 1;\nint y = 2;\n");
        let filter = FilterSet::new();
        let config = Config::default();
        let mut ctx = LintContext::new("test.cc", &lines, &filter, &config);
        check_crlf(bytes, &mut ctx);
        assert_eq!(ctx.violations.len(), 0);
    }
}
