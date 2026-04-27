use cclinter::checker::magic_number::check_magic_number;
use cclinter::common::source::SourceFile;
use cclinter::config::MagicNumberConfig;
use std::path::PathBuf;

fn disable_color() {
    colored::control::set_override(false);
}

fn default_config() -> MagicNumberConfig {
    MagicNumberConfig {
        enabled: true,
        allowed: vec![0, 1, -1, 2],
    }
}

#[test]
fn test_detect_magic_number() {
    disable_color();
    let input = "int x = 42;\nint y = 0;\nint z = 100;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.iter().any(|d| d.message.contains("42")));
    assert!(diags.iter().any(|d| d.message.contains("100")));
}

#[test]
fn test_allowed_numbers_not_flagged() {
    disable_color();
    let input = "int x = 0;\nint y = 1;\nint z = -1;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_disabled_check() {
    disable_color();
    let input = "int x = 42;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let config = MagicNumberConfig {
        enabled: false,
        allowed: vec![],
    };
    let diags = check_magic_number(&src, &config);
    assert!(diags.is_empty());
}

#[test]
fn test_hex_not_flagged() {
    disable_color();
    let input = "int x = 0xFF;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_float_not_flagged() {
    disable_color();
    let input = "float x = 3.14;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_string_literal_not_flagged() {
    disable_color();
    let input = "const char* s = \"value 42\";\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_char_literal_not_flagged() {
    disable_color();
    let input = "char c = '\\x42';\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_preprocessor_skipped() {
    disable_color();
    let input = "#define MAX 100\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_inline_comment_skipped() {
    disable_color();
    let input = "int x = 0; // 100 here\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_negative_magic_number() {
    disable_color();
    let input = "int x = -42;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.iter().any(|d| d.message.contains("-42")));
}

#[test]
fn test_negative_one_allowed() {
    disable_color();
    let input = "int x = -1;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_empty_file() {
    disable_color();
    let src = SourceFile::from_string("", PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_custom_allowlist() {
    disable_color();
    let config = MagicNumberConfig {
        enabled: true,
        allowed: vec![0, 1, -1, 2, 42],
    };
    let input = "int x = 42;\nint y = 100;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &config);
    assert!(!diags.iter().any(|d| d.message.contains("42")));
    assert!(diags.iter().any(|d| d.message.contains("100")));
}

#[test]
fn test_identifier_with_digits_not_flagged() {
    disable_color();
    let input = "int var123 = 0;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_scientific_notation_not_flagged() {
    disable_color();
    let input = "double x = 1e10;\ndouble y = 2E+5;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_multiple_numbers_per_line() {
    disable_color();
    let input = "int x = 42, y = 100, z = 7;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.iter().any(|d| d.message.contains("42")));
    assert!(diags.iter().any(|d| d.message.contains("100")));
    assert!(diags.iter().any(|d| d.message.contains("7")));
}

#[test]
fn test_subtraction_not_negative_magic() {
    disable_color();
    let input = "int x = count-42;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.iter().any(|d| d.message.contains("42") && !d.message.contains("-42")));
}

#[test]
fn test_block_comment_line_skipped() {
    disable_color();
    let input = "/* int x = 42; */\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_type_suffix_ul_detected() {
    disable_color();
    let input = "unsigned long x = 42UL;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.iter().any(|d| d.message.contains("42")));
}

#[test]
fn test_type_suffix_ll_detected() {
    disable_color();
    let input = "long long x = 42LL;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.iter().any(|d| d.message.contains("42")));
}

#[test]
fn test_type_suffix_u_detected() {
    disable_color();
    let input = "unsigned x = 42u;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.iter().any(|d| d.message.contains("42")));
}

#[test]
fn test_inline_block_comment_not_flagged() {
    disable_color();
    let input = "int x = /* 42 */ 0;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_column_offset_with_indent() {
    disable_color();
    let input = "    int x = 42;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.iter().any(|d| d.col > 4));
}

#[test]
fn test_octal_flagged_as_decimal() {
    disable_color();
    let input = "int x = 0777;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.iter().any(|d| d.message.contains("777")));
}

#[test]
fn test_array_index_flagged() {
    disable_color();
    let input = "int x = arr[42];\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.iter().any(|d| d.message.contains("42")));
}

#[test]
fn test_dot_five_float_not_flagged() {
    disable_color();
    let input = "float x = .5;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_float_with_f_suffix_not_flagged() {
    disable_color();
    let input = "float x = 3.14f;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_enum_values_flagged() {
    disable_color();
    let input = "enum { A = 10, B = 20 };\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_number(&src, &default_config());
    assert!(diags.iter().any(|d| d.message.contains("10")));
    assert!(diags.iter().any(|d| d.message.contains("20")));
}
