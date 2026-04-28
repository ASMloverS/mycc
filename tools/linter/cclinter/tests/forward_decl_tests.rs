use cclinter::checker::forward_decl::check_forward_decl;
use cclinter::common::source::SourceFile;
use std::path::PathBuf;

fn disable_color() {
    colored::control::set_override(false);
}

#[test]
fn test_missing_forward_decl() {
    disable_color();
    let input = "void foo() {\n    bar();\n}\nvoid bar() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decl(&src);
    assert!(diags.iter().any(|d| d.message.contains("bar")));
}

#[test]
fn test_has_forward_decl() {
    disable_color();
    let input = "void bar();\nvoid foo() {\n    bar();\n}\nvoid bar() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decl(&src);
    assert!(!diags.iter().any(|d| d.message.contains("bar")));
}

#[test]
fn test_no_call_no_issue() {
    disable_color();
    let input = "void foo() {}\nvoid bar() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decl(&src);
    assert!(diags.is_empty());
}

#[test]
fn test_string_literal_no_false_positive() {
    disable_color();
    let input = "void foo() {\n    char *s = \"bar()\";\n}\nvoid bar() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decl(&src);
    assert!(!diags.iter().any(|d| d.message.contains("bar")));
}

#[test]
fn test_block_comment_no_false_positive() {
    disable_color();
    let input = "void foo() {\n    /* bar(); */\n}\nvoid bar() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decl(&src);
    assert!(!diags.iter().any(|d| d.message.contains("bar")));
}

#[test]
fn test_keyword_not_flagged() {
    disable_color();
    let input = "void foo() {\n    if (x) {}\n    for (;;) {}\n    while (1) {}\n    switch (c) {}\n    return;\n    sizeof(int);\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decl(&src);
    assert!(diags.is_empty());
}

#[test]
fn test_left_aligned_pointer() {
    disable_color();
    let input = "int* foo() { bar(); }\nint* bar() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decl(&src);
    assert!(diags.iter().any(|d| d.message.contains("bar")));
}

#[test]
fn test_allman_brace_style() {
    disable_color();
    let input = "void foo() { bar(); }\nvoid bar()\n{\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decl(&src);
    assert!(diags.iter().any(|d| d.message.contains("bar")));
}

#[test]
fn test_single_line_def_with_call() {
    disable_color();
    let input = "void foo() { bar(); }\nvoid bar() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decl(&src);
    assert!(diags.iter().any(|d| d.message.contains("bar")));
}

#[test]
fn test_static_qualifier() {
    disable_color();
    let input = "static void foo() { bar(); }\nstatic void bar() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decl(&src);
    assert!(diags.iter().any(|d| d.message.contains("bar")));
}

#[test]
fn test_deduplication() {
    disable_color();
    let input = "void foo() {\n    bar();\n    bar();\n}\nvoid bar() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decl(&src);
    let bar_count = diags.iter().filter(|d| d.message.contains("bar")).count();
    assert_eq!(bar_count, 1);
}

#[test]
fn test_multi_word_return_type() {
    disable_color();
    let input = "unsigned int foo() { bar(); }\nunsigned int bar() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decl(&src);
    assert!(diags.iter().any(|d| d.message.contains("bar")));
}
