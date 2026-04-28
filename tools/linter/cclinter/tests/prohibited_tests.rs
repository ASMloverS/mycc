use cclinter::checker::prohibited::check_prohibited;
use cclinter::common::source::SourceFile;
use std::path::PathBuf;

fn disable_color() {
    colored::control::set_override(false);
}

#[test]
fn test_default_prohibited() {
    disable_color();
    let input = "char buf[10];\nstrcpy(buf, src);\nsprintf(buf, \"%d\", x);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_prohibited(&src, true, &[], &[]);
    assert!(diags.iter().any(|d| d.message.contains("strcpy")));
    assert!(diags.iter().any(|d| d.message.contains("sprintf")));
}

#[test]
fn test_extra_prohibited() {
    disable_color();
    let input = "malloc(10);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_prohibited(&src, true, &["malloc".into()], &[]);
    assert!(diags.iter().any(|d| d.message.contains("malloc")));
}

#[test]
fn test_remove_from_default() {
    disable_color();
    let input = "strcpy(buf, src);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_prohibited(&src, true, &[], &["strcpy".into()]);
    assert!(!diags.iter().any(|d| d.message.contains("strcpy")));
}

#[test]
fn test_use_default_false_no_builtin_diags() {
    disable_color();
    let input = "strcpy(buf, src);\nsprintf(buf, \"%d\", x);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_prohibited(&src, false, &[], &[]);
    assert!(diags.is_empty());
}

#[test]
fn test_string_literal_no_false_positive() {
    disable_color();
    let input = "char *s = \"strcpy is bad\";\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_prohibited(&src, true, &[], &[]);
    assert!(diags.is_empty());
}

#[test]
fn test_block_comment_no_false_positive() {
    disable_color();
    let input = "/* strcpy(dst, src); */\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_prohibited(&src, true, &[], &[]);
    assert!(diags.is_empty());
}

#[test]
fn test_word_boundary_no_suffix_match() {
    disable_color();
    let input = "strcpy_s(dst, src);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_prohibited(&src, true, &[], &[]);
    assert!(diags.is_empty());
}

#[test]
fn test_multiple_occurrences_same_line() {
    disable_color();
    let input = "strcpy(a, b); strcpy(c, d);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_prohibited(&src, true, &[], &[]);
    assert!(diags.iter().any(|d| d.message.contains("strcpy")));
}

#[test]
fn test_multiple_distinct_fns_same_line() {
    disable_color();
    let input = "strcpy(a, b); sprintf(buf, \"%d\", x);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_prohibited(&src, true, &[], &[]);
    assert!(diags.iter().any(|d| d.message.contains("strcpy")));
    assert!(diags.iter().any(|d| d.message.contains("sprintf")));
    assert_eq!(diags.len(), 2);
}
